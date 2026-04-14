use druid::{AppDelegate, Command, Data, DelegateCtx, Env, Handled, Lens, Selector, Target};

use crate::svn_prune::state::SvnPruneState;

// ─────────────────────────────────────────────────────────────
// Selectors（异步动作回调信号）
// ─────────────────────────────────────────────────────────────

/// load_remote 完成：Ok(authz 原文) / Err(错误信息)
pub const PRUNE_LOAD_DONE: Selector<Result<String, String>> =
    Selector::new("prune.load.done");

/// gen_preview 完成：Ok((备份路径, 新文件路径, 影响摘要)) / Err(错误信息)
pub const PRUNE_PREVIEW_DONE: Selector<Result<(String, String, String), String>> =
    Selector::new("prune.preview.done");

/// apply_remote 完成：Ok(成功信息) / Err(错误信息)
pub const PRUNE_APPLY_DONE: Selector<Result<String, String>> =
    Selector::new("prune.apply.done");

// ─────────────────────────────────────────────────────────────
// 顶层应用状态
// ─────────────────────────────────────────────────────────────

#[derive(Clone, Data, Lens)]
pub struct AppState {
    /// 原有"SVN 权限开通"Tab 的状态（保持不变）
    pub svn_add: crate::SvnAddState,
    /// 新"SVN 权限瘦身"Tab 的状态
    pub svn_prune: SvnPruneState,
}

impl AppState {
    pub fn new() -> Self {
        AppState {
            svn_add: crate::SvnAddState::new(),
            svn_prune: SvnPruneState::new(),
        }
    }
}

// ─────────────────────────────────────────────────────────────
// AppDelegate：接收异步 Selector，写回 AppState
// ─────────────────────────────────────────────────────────────

pub struct PruneDelegate;

impl AppDelegate<AppState> for PruneDelegate {
    fn command(
        &mut self,
        _ctx: &mut DelegateCtx,
        _target: Target,
        cmd: &Command,
        data: &mut AppState,
        _env: &Env,
    ) -> Handled {
        use crate::common::message::MessageType;

        if let Some(result) = cmd.get(PRUNE_LOAD_DONE) {
            match result {
                Ok(raw) => {
                    use crate::svn_prune::authz::AuthzModel;
                    use crate::svn_prune::state::UserRow;
                    use druid::im::Vector;

                    let model = AuthzModel::parse(raw);
                    let users = model.real_users();

                    let mut all_users = Vector::new();
                    for name in users {
                        all_users.push_back(UserRow {
                            name,
                            selected: false,
                            visible: true,
                        });
                    }

                    let now = chrono_now();
                    data.svn_prune.raw_authz = raw.clone();
                    data.svn_prune.loaded_at = now;
                    data.svn_prune.all_users = all_users;
                    data.svn_prune.detail_rows = druid::im::Vector::new();
                    data.svn_prune.impact_text = String::new();
                    data.svn_prune.stage = crate::svn_prune::state::PruneStage::Loaded;
                    data.svn_prune.set_message(
                        format!(
                            "加载成功，共 {} 位用户",
                            data.svn_prune.all_users.len()
                        ),
                        MessageType::Info,
                    );
                }
                Err(e) => {
                    data.svn_prune
                        .set_message(format!("加载失败：{}", e), MessageType::Error);
                }
            }
            return Handled::Yes;
        }

        if let Some(result) = cmd.get(PRUNE_PREVIEW_DONE) {
            match result {
                Ok((backup_path, new_path, impact)) => {
                    let mut paths = druid::im::Vector::new();
                    paths.push_back(backup_path.clone());
                    paths.push_back(new_path.clone());
                    data.svn_prune.preview_paths = paths;
                    data.svn_prune.impact_text = impact.clone();
                    data.svn_prune.stage = crate::svn_prune::state::PruneStage::Loaded;
                    data.svn_prune.set_message("预览文件已生成，请查看备份目录确认", MessageType::Info);
                }
                Err(e) => {
                    data.svn_prune.stage = crate::svn_prune::state::PruneStage::Loaded;
                    data.svn_prune
                        .set_message(format!("生成预览失败：{}", e), MessageType::Error);
                }
            }
            return Handled::Yes;
        }

        if let Some(result) = cmd.get(PRUNE_APPLY_DONE) {
            match result {
                Ok(msg) => {
                    data.svn_prune.stage = crate::svn_prune::state::PruneStage::Done;
                    data.svn_prune.set_message(msg.clone(), MessageType::Info);
                }
                Err(e) => {
                    data.svn_prune.stage = crate::svn_prune::state::PruneStage::Loaded;
                    data.svn_prune
                        .set_message(format!("应用失败：{}", e), MessageType::Error);
                }
            }
            return Handled::Yes;
        }

        Handled::No
    }
}

fn chrono_now() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    // 简单格式化为 HH:MM:SS（仅用于展示）
    let s = secs % 60;
    let m = (secs / 60) % 60;
    let h = (secs / 3600) % 24;
    format!("{:02}:{:02}:{:02}", h, m, s)
}
