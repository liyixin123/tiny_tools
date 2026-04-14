use druid::im::Vector;
use druid::{Color, Data, Lens};

use crate::common::message::MessageType;

#[derive(Clone, Copy, Data, PartialEq, Debug)]
pub enum PruneStage {
    Idle,
    Loading,
    Loaded,
    GeneratingPreview,
    Applying,
    Done,
}

#[derive(Clone, Copy, Data, PartialEq, Debug)]
pub enum PermKind {
    Repo,
    Group,
}

/// 左侧用户列表的一行。
#[derive(Clone, Data, Lens, Debug)]
pub struct UserRow {
    pub name: String,
    pub selected: bool,
    /// false 时被搜索过滤掉，UI 层用 Either 隐藏（不占高度）
    pub visible: bool,
}

/// 右侧权限明细的一行（所有选中用户的权限聚合展示）。
#[derive(Clone, Data, Lens, Debug)]
pub struct PermissionRow {
    /// 归属用户
    pub user: String,
    /// 显示文本，如 "[softwarerepo:/a/b] rw" 或 "sw_app_3 (组)"
    pub label: String,
    /// 精确匹配键（仓库段头或组名）
    pub key: String,
    pub kind: PermKind,
    /// UI 勾选框：true 表示"将要删除此项"（默认全选）
    pub checked: bool,
}

#[derive(Clone, Data, Lens)]
pub struct SvnPruneState {
    pub stage: PruneStage,
    /// 从远程拉取的 authz 原文
    pub raw_authz: String,
    /// 加载时间显示字符串
    pub loaded_at: String,
    /// 搜索框内容
    pub search: String,
    /// 真人账号全集（按字母排序，含过滤标志）
    pub all_users: Vector<UserRow>,
    /// 所有选中用户的权限明细（聚合，含 user 字段区分归属）
    pub detail_rows: Vector<PermissionRow>,
    /// "预览影响" 后填充的多行文本
    pub impact_text: String,
    /// 生成的预览文件绝对路径
    pub preview_paths: Vector<String>,
    /// 顶部消息条
    pub message: String,
    pub message_color: Color,
    /// 操作进行中（控制按钮 disabled）
    pub busy: bool,
    /// busy 时显示的提示文字
    pub busy_hint: String,
    /// 二次确认弹窗是否打开
    pub confirm_open: bool,
    /// 弹窗内的摘要文字
    pub confirm_summary: String,
}

impl SvnPruneState {
    pub fn new() -> Self {
        SvnPruneState {
            stage: PruneStage::Idle,
            raw_authz: String::new(),
            loaded_at: String::new(),
            search: String::new(),
            all_users: Vector::new(),
            detail_rows: Vector::new(),
            impact_text: String::new(),
            preview_paths: Vector::new(),
            message: String::new(),
            message_color: Color::GREEN,
            busy: false,
            busy_hint: String::new(),
            confirm_open: false,
            confirm_summary: String::new(),
        }
    }

    pub fn set_message(&mut self, msg: impl Into<String>, mt: MessageType) {
        self.message = msg.into();
        self.message_color = mt.color();
        self.busy = false;
        self.busy_hint = String::new();
    }

    pub fn set_busy(&mut self, hint: impl Into<String>) {
        self.busy = true;
        self.busy_hint = hint.into();
    }

    /// 根据搜索框内容更新 all_users 的 visible 标志。
    pub fn apply_search_filter(&mut self) {
        let q = self.search.to_lowercase();
        for row in self.all_users.iter_mut() {
            row.visible = q.is_empty() || row.name.to_lowercase().contains(&q);
        }
    }

    /// 返回当前已选中的用户名列表。
    pub fn selected_users(&self) -> Vec<String> {
        self.all_users
            .iter()
            .filter(|r| r.selected)
            .map(|r| r.name.clone())
            .collect()
    }

    /// 重新根据已选用户的 detail_rows 刷新右侧明细（从 authz 解析结果推算）。
    pub fn rebuild_detail_rows(&mut self, authz: &crate::svn_prune::authz::AuthzModel) {
        let mut rows = Vector::new();
        for row in self.all_users.iter().filter(|r| r.selected) {
            let detail = authz.lookup(&row.name);
            for repo in &detail.repos {
                rows.push_back(PermissionRow {
                    user: row.name.clone(),
                    label: format!("{} ({})", repo.repo_name, repo.permission),
                    key: repo.repo_name.clone(),
                    kind: PermKind::Repo,
                    checked: true,
                });
            }
            for group in &detail.groups {
                rows.push_back(PermissionRow {
                    user: row.name.clone(),
                    label: format!("{} (组)", group),
                    key: group.clone(),
                    kind: PermKind::Group,
                    checked: true,
                });
            }
        }
        self.detail_rows = rows;
    }
}
