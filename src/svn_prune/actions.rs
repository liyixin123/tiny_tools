// 异步动作：与远程 SVN Edge 交互，通过 ExtEventSink 把结果发回 UI 线程。
// 所有函数立即返回，实际工作在 tokio 后台线程执行。

use chrono::Local;
use druid::{ExtEventSink, Target};
use subversion_edge_modify_tool::html_parser::SubversionEdgeConfig;
use subversion_edge_modify_tool::start_init;

use crate::app::{PRUNE_APPLY_DONE, PRUNE_LOAD_DONE, PRUNE_PREVIEW_DONE};
use crate::common::runtime::rt;
use crate::svn_prune::authz::{AuthzModel, PruneRequest};

// ─────────────────────────────────────────────────────────────
// Phase 1：加载远程 authz
// ─────────────────────────────────────────────────────────────

/// 异步拉取远程 authz 原文，结果通过 PRUNE_LOAD_DONE 发回 UI。
pub fn load_remote(sink: ExtEventSink) {
    rt().spawn(async move {
        let result: Result<String, String> = try_load().await;
        let _ = sink.submit_command(PRUNE_LOAD_DONE, result, Target::Auto);
    });
}

async fn try_load() -> Result<String, String> {
    start_init::start_init();
    let mut client = SubversionEdgeConfig::new()
        .await
        .map_err(|e| e.to_string())?;
    // read_only=true：只拉 showAuthorization，不需要 token
    client
        .get_file_content_from_remote(true)
        .await
        .map_err(|e| e.to_string())
}

// ─────────────────────────────────────────────────────────────
// Phase 2：生成本地预览文件
// ─────────────────────────────────────────────────────────────

/// 基于当前 raw_authz 与勾选的 reqs 生成预览文件（不推服务器）。
/// 结果通过 PRUNE_PREVIEW_DONE 发回 UI：Ok((backup_path, new_path, impact_text))。
pub fn gen_preview(raw_authz: String, reqs: Vec<PruneRequest>, sink: ExtEventSink) {
    rt().spawn(async move {
        let result = try_gen_preview(raw_authz, reqs).await;
        let _ = sink.submit_command(PRUNE_PREVIEW_DONE, result, Target::Auto);
    });
}

async fn try_gen_preview(
    raw_authz: String,
    reqs: Vec<PruneRequest>,
) -> Result<(String, String, String), String> {
    start_init::start_init();
    let backup_dir = start_init::get_backups_dir()
        .ok_or_else(|| "无法获取备份目录".to_string())?;

    let ts = timestamp();
    let backup_path = backup_dir.join(format!("{}_prune_backup.txt", ts));
    let new_path = backup_dir.join(format!("{}_prune_new.txt", ts));

    // 先写原文备份
    std::fs::write(&backup_path, &raw_authz).map_err(|e| e.to_string())?;

    // 瘦身算法
    let mut model = AuthzModel::parse(&raw_authz);
    let report = model.apply_prune(&reqs);
    let new_content = model.serialize();

    // 写新内容预览
    std::fs::write(&new_path, &new_content).map_err(|e| e.to_string())?;

    Ok((
        backup_path.to_string_lossy().into_owned(),
        new_path.to_string_lossy().into_owned(),
        report.summary_text(),
    ))
}

// ─────────────────────────────────────────────────────────────
// Phase 3：应用到远程服务器
// ─────────────────────────────────────────────────────────────

/// 重新从服务器拉取最新 authz，执行瘦身，本地备份，再推回服务器。
/// 支持 DRY_RUN=1 环境变量保护（只写本地，不推服务器）。
/// 结果通过 PRUNE_APPLY_DONE 发回 UI：Ok(成功消息) / Err(错误消息)。
pub fn apply_remote(reqs: Vec<PruneRequest>, sink: ExtEventSink) {
    rt().spawn(async move {
        let result = try_apply_remote(reqs).await;
        let _ = sink.submit_command(PRUNE_APPLY_DONE, result, Target::Auto);
    });
}

async fn try_apply_remote(reqs: Vec<PruneRequest>) -> Result<String, String> {
    start_init::start_init();
    let dry_run = std::env::var("DRY_RUN").is_ok();

    // 1. 重新拉取最新 authz 并获取 SYNCHRONIZER_TOKEN（read_only=false）
    //    post_content 依赖 get_file_content_from_remote(false) 填充的 token/fileId
    let mut client = SubversionEdgeConfig::new()
        .await
        .map_err(|e| e.to_string())?;
    let latest_authz = client
        .get_file_content_from_remote(false)
        .await
        .map_err(|e| e.to_string())?;

    // 2. 执行瘦身算法
    let mut model = AuthzModel::parse(&latest_authz);
    let report = model.apply_prune(&reqs);

    if report.is_empty() {
        return Err("服务器上当前内容与预期不符，没有找到需要删除的权限，操作已取消。\n请重新加载后再试。".to_string());
    }

    let new_content = model.serialize();

    // 3. 本地备份（备份最新拉取内容 + 瘦身后内容）
    let backup_dir = start_init::get_backups_dir()
        .ok_or_else(|| "无法获取备份目录".to_string())?;
    let ts = timestamp();
    let backup_path = backup_dir.join(format!("{}_prune_apply_backup.txt", ts));
    let new_path = backup_dir.join(format!("{}_prune_apply_new.txt", ts));
    std::fs::write(&backup_path, &latest_authz).map_err(|e| e.to_string())?;
    std::fs::write(&new_path, &new_content).map_err(|e| e.to_string())?;

    if dry_run {
        return Ok(format!(
            "[DRY-RUN] 未推送服务器。本地备份已生成：\n备份: {}\n新文件: {}",
            backup_path.display(),
            new_path.display()
        ));
    }

    // 4. 推送到服务器
    client
        .post_content(&new_content)
        .await
        .map_err(|e| e.to_string())?;

    // 5. 简单验证：再次拉取，检查目标用户是否已从 authz 消失
    //    用 read_only=true 拉，不再取 token
    let verify_result = async {
        let mut vc = SubversionEdgeConfig::new().await?;
        vc.get_file_content_from_remote(true).await
    }
    .await;

    match verify_result {
        Ok(verify_content) => {
            let verify_model = AuthzModel::parse(&verify_content);
            // 简单检查：所有被删用户不应再出现在直接授权行里
            let deleted_users: Vec<&str> = reqs.iter().map(|r| r.user.as_str()).collect();
            let mut still_found: Vec<String> = Vec::new();
            for user in &deleted_users {
                let detail = verify_model.lookup(user);
                if !detail.repos.is_empty() {
                    // 还有残余仓库权限（可能是重新加入，也可能推送不完整）
                    for repo in &detail.repos {
                        still_found.push(format!("{} → {}", user, repo.repo_name));
                    }
                }
            }
            if still_found.is_empty() {
                Ok(format!(
                    "权限已成功应用到服务器！删除了 {} 条仓库授权、修改了 {} 个用户组。",
                    report.repo_removals.len(),
                    report.group_removals.len()
                ))
            } else {
                Ok(format!(
                    "已推送，但验证时发现以下条目仍存在（可能推送部分失败，请核对备份）：\n{}",
                    still_found.join("\n")
                ))
            }
        }
        Err(_) => {
            // 验证拉取失败，不影响推送本身，只给提示
            Ok(format!(
                "已推送到服务器（验证拉取失败，建议手动确认）。删除了 {} 条仓库授权、修改了 {} 个用户组。",
                report.repo_removals.len(),
                report.group_removals.len()
            ))
        }
    }
}

// ─────────────────────────────────────────────────────────────
// 工具函数
// ─────────────────────────────────────────────────────────────

/// 格式化当前本地时间为文件名前缀，与底层 crate 格式一致：YYYY-MM-DD-HH-MM-SS
fn timestamp() -> String {
    Local::now().format("%Y-%m-%d-%H-%M-%S").to_string()
}
