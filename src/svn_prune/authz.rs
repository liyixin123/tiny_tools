// authz 文件的解析、结构化访问、瘦身算法与序列化。
// 纯逻辑，零 druid 依赖，便于单元测试。

use std::collections::{BTreeSet, HashMap, HashSet};

// ─────────────────────────────────────────────────────────────
// 公开类型
// ─────────────────────────────────────────────────────────────

/// 某用户在一个仓库段的授权记录。
#[derive(Debug, Clone, PartialEq)]
pub struct RepoAuth {
    /// 仓库段头，形如 `[softwarerepo:/a/b]`
    pub repo_name: String,
    /// 权限字符串，"rw" 或 "r"
    pub permission: String,
}

/// 某用户的所有权限：直接仓库授权 + 所属用户组。
#[derive(Debug, Default, Clone)]
pub struct UserDetail {
    pub repos: Vec<RepoAuth>,
    /// 所属组名（不含 @，如 "sw_app_3"）
    pub groups: Vec<String>,
}

/// 一次瘦身请求：指定用户 + 要删除的仓库授权集合 + 要从中移除成员的用户组集合。
/// 两个 Vec 为空表示不操作对应维度。
#[derive(Debug, Clone)]
pub struct PruneRequest {
    pub user: String,
    pub repos_to_remove: Vec<String>,
    pub groups_to_remove: Vec<String>,
}

/// apply_prune 产生的影响摘要，供 UI 显示预览。
#[derive(Debug, Default, Clone)]
pub struct ImpactReport {
    /// (用户, 仓库名, 权限)
    pub repo_removals: Vec<(String, String, String)>,
    /// (用户, 组名)
    pub group_removals: Vec<(String, String)>,
}

impl ImpactReport {
    pub fn is_empty(&self) -> bool {
        self.repo_removals.is_empty() && self.group_removals.is_empty()
    }

    /// 生成用户可读的摘要文本。
    pub fn summary_text(&self) -> String {
        if self.is_empty() {
            return "无需变更".to_string();
        }

        // 按用户聚合
        let mut by_user: HashMap<&str, (Vec<(&str, &str)>, Vec<&str>)> = HashMap::new();
        for (user, repo, perm) in &self.repo_removals {
            by_user
                .entry(user.as_str())
                .or_default()
                .0
                .push((repo.as_str(), perm.as_str()));
        }
        for (user, group) in &self.group_removals {
            by_user
                .entry(user.as_str())
                .or_default()
                .1
                .push(group.as_str());
        }

        let mut lines = Vec::new();
        let mut users: Vec<&str> = by_user.keys().copied().collect();
        users.sort_unstable();
        for user in users {
            let (repos, groups) = &by_user[user];
            lines.push(format!("【{}】", user));
            for (repo, perm) in repos {
                lines.push(format!("  删除仓库权限: {} ({})", repo, perm));
            }
            for group in groups {
                lines.push(format!("  从用户组移除: {}", group));
            }
        }
        lines.join("\n")
    }
}

// ─────────────────────────────────────────────────────────────
// 内部结构
// ─────────────────────────────────────────────────────────────

struct GroupEntry {
    line: usize,
    group_name: String,
    members: Vec<String>,
    /// 等号左侧含等号的前缀，如 "sw_app_3=" 或 "sw_app_3 = "
    prefix: String,
    /// 成员间分隔符，通常 "," 或 ", "
    sep: String,
}

struct AuthEntry {
    line: usize,
    principal: String,
    permission: String,
}

enum Section {
    Groups {
        #[allow(dead_code)]
        header_line: usize,
        entries: Vec<GroupEntry>,
    },
    Repo {
        #[allow(dead_code)]
        header_line: usize,
        /// 含方括号的段头，如 "[softwarerepo:/a/b]"
        name: String,
        entries: Vec<AuthEntry>,
    },
    /// [test] 等未知段，原样保留
    Other {
        #[allow(dead_code)]
        header_line: usize,
    },
}

// ─────────────────────────────────────────────────────────────
// 主结构
// ─────────────────────────────────────────────────────────────

pub struct AuthzModel {
    raw_lines: Vec<String>,
    line_ending: &'static str,
    has_bom: bool,
    ends_with_newline: bool,
    sections: Vec<Section>,
    /// 待删除的行索引集合（用于 apply_prune）
    tombstones: BTreeSet<usize>,
    /// 待改写的行（用于 groups 成员删除）
    overrides: HashMap<usize, String>,
}

impl AuthzModel {
    // ─── 解析 ───────────────────────────────────────────────

    pub fn parse(content: &str) -> Self {
        // 1. BOM 检测
        let has_bom = content.starts_with('\u{FEFF}');
        let content = if has_bom { &content[3..] } else { content };

        // 2. 换行风格检测
        let line_ending: &'static str = if content.contains("\r\n") { "\r\n" } else { "\n" };
        let ends_with_newline = content.ends_with('\n');

        // 3. 按行切分（lines() 自动处理 \r\n / \n，去掉行尾换行符）
        let raw_lines: Vec<String> = content.lines().map(String::from).collect();

        // 4. 扫描段落
        let sections = Self::scan_sections(&raw_lines);

        AuthzModel {
            raw_lines,
            line_ending,
            has_bom,
            ends_with_newline,
            sections,
            tombstones: BTreeSet::new(),
            overrides: HashMap::new(),
        }
    }

    fn scan_sections(raw_lines: &[String]) -> Vec<Section> {
        // 临时 builder
        enum Builder {
            Groups {
                header_line: usize,
                entries: Vec<GroupEntry>,
            },
            Repo {
                header_line: usize,
                name: String,
                entries: Vec<AuthEntry>,
            },
            Other {
                header_line: usize,
            },
        }

        fn flush(sections: &mut Vec<Section>, b: Builder) {
            match b {
                Builder::Groups { header_line, entries } => {
                    sections.push(Section::Groups { header_line, entries });
                }
                Builder::Repo { header_line, name, entries } => {
                    sections.push(Section::Repo { header_line, name, entries });
                }
                Builder::Other { header_line } => {
                    sections.push(Section::Other { header_line });
                }
            }
        }

        let mut sections = Vec::new();
        let mut current: Option<Builder> = None;

        for (i, line) in raw_lines.iter().enumerate() {
            let trimmed = line.trim();

            if is_section_header(trimmed) {
                if let Some(b) = current.take() {
                    flush(&mut sections, b);
                }
                let inner = &trimmed[1..trimmed.len() - 1];
                current = Some(if inner == "groups" {
                    Builder::Groups { header_line: i, entries: vec![] }
                } else if inner.contains(':') {
                    Builder::Repo {
                        header_line: i,
                        name: trimmed.to_string(),
                        entries: vec![],
                    }
                } else {
                    Builder::Other { header_line: i }
                });
                continue;
            }

            // 跳过注释与空行
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            match current {
                Some(Builder::Groups { ref mut entries, .. }) => {
                    if let Some(entry) = parse_group_entry(i, trimmed) {
                        entries.push(entry);
                    }
                }
                Some(Builder::Repo { ref mut entries, .. }) => {
                    if let Some(entry) = parse_auth_entry(i, trimmed) {
                        entries.push(entry);
                    }
                }
                _ => {}
            }
        }

        if let Some(b) = current {
            flush(&mut sections, b);
        }
        sections
    }

    // ─── 查询 ───────────────────────────────────────────────

    /// 返回 authz 中所有"真人账号"（排除 @组名）。
    /// 来源：[groups] 成员列表 + 所有仓库段的直接授权行。
    pub fn real_users(&self) -> BTreeSet<String> {
        let mut users = BTreeSet::new();
        for section in &self.sections {
            match section {
                Section::Groups { entries, .. } => {
                    for e in entries {
                        for m in &e.members {
                            if !m.starts_with('@') {
                                users.insert(m.clone());
                            }
                        }
                    }
                }
                Section::Repo { entries, .. } => {
                    for e in entries {
                        if !e.principal.starts_with('@') {
                            users.insert(e.principal.clone());
                        }
                    }
                }
                Section::Other { .. } => {}
            }
        }
        users
    }

    /// 查询某用户的所有权限详情（仓库授权 + 所属组）。
    pub fn lookup(&self, user: &str) -> UserDetail {
        let mut detail = UserDetail::default();
        for section in &self.sections {
            match section {
                Section::Repo { name, entries, .. } => {
                    for e in entries {
                        if e.principal == user {
                            detail.repos.push(RepoAuth {
                                repo_name: name.clone(),
                                permission: e.permission.clone(),
                            });
                        }
                    }
                }
                Section::Groups { entries, .. } => {
                    for e in entries {
                        if e.members.iter().any(|m| m == user) {
                            detail.groups.push(e.group_name.clone());
                        }
                    }
                }
                Section::Other { .. } => {}
            }
        }
        detail
    }

    // ─── 瘦身 ───────────────────────────────────────────────

    /// 执行批量瘦身请求，填充 tombstones / overrides，返回影响摘要。
    /// 不修改 raw_lines，调用 serialize() 才产出新内容。
    pub fn apply_prune(&mut self, reqs: &[PruneRequest]) -> ImpactReport {
        let mut report = ImpactReport::default();

        // 先收集变更意图，避免同时可变/不可变借用 self
        let mut to_tombstone: Vec<usize> = Vec::new();
        let mut to_override: Vec<(usize, String)> = Vec::new();

        for req in reqs {
            let repos_set: HashSet<&str> = req.repos_to_remove.iter().map(String::as_str).collect();
            let groups_set: HashSet<&str> = req.groups_to_remove.iter().map(String::as_str).collect();

            for section in &self.sections {
                match section {
                    Section::Repo { name, entries, .. } if repos_set.contains(name.as_str()) => {
                        for e in entries {
                            if e.principal == req.user {
                                to_tombstone.push(e.line);
                                report.repo_removals.push((
                                    req.user.clone(),
                                    name.clone(),
                                    e.permission.clone(),
                                ));
                            }
                        }
                    }
                    Section::Groups { entries, .. } => {
                        for e in entries {
                            if groups_set.contains(e.group_name.as_str()) {
                                let new_members: Vec<&str> = e
                                    .members
                                    .iter()
                                    .filter(|m| m.as_str() != req.user)
                                    .map(String::as_str)
                                    .collect();
                                if new_members.len() < e.members.len() {
                                    let new_line =
                                        format!("{}{}", e.prefix, new_members.join(&e.sep));
                                    to_override.push((e.line, new_line));
                                    report.group_removals.push((
                                        req.user.clone(),
                                        e.group_name.clone(),
                                    ));
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        for idx in to_tombstone {
            self.tombstones.insert(idx);
        }
        for (idx, content) in to_override {
            self.overrides.insert(idx, content);
        }

        report
    }

    // ─── 序列化 ─────────────────────────────────────────────

    /// 将当前模型（含 tombstones/overrides）序列化为 authz 文本。
    pub fn serialize(&self) -> String {
        let lines: Vec<&str> = self
            .raw_lines
            .iter()
            .enumerate()
            .filter(|(i, _)| !self.tombstones.contains(i))
            .map(|(i, line)| {
                if let Some(new_content) = self.overrides.get(&i) {
                    new_content.as_str()
                } else {
                    line.as_str()
                }
            })
            .collect();

        let mut result = lines.join(self.line_ending);

        if self.ends_with_newline {
            result.push_str(self.line_ending);
        }

        if self.has_bom {
            format!("\u{FEFF}{}", result)
        } else {
            result
        }
    }
}

// ─────────────────────────────────────────────────────────────
// 内部辅助函数
// ─────────────────────────────────────────────────────────────

fn is_section_header(line: &str) -> bool {
    line.starts_with('[') && line.ends_with(']') && line.len() > 2
}

/// 解析 groups 段的一行，如 `sw_app_3=gaowei,makun,liyixin`
fn parse_group_entry(line_idx: usize, line: &str) -> Option<GroupEntry> {
    let eq_pos = line.find('=')?;
    let key = line[..eq_pos].trim().to_string();
    let val = &line[eq_pos + 1..];

    // 保留等号后的原始空格作为前缀
    let val_ltrimmed = val.trim_start();
    let leading = val.len() - val_ltrimmed.len();
    let prefix = format!("{}={}", key, &val[..leading]);

    // 检测成员间分隔符（保留空格风格）
    let sep = if val_ltrimmed.contains(", ") { ", " } else { "," }.to_string();

    let members: Vec<String> = val_ltrimmed
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    Some(GroupEntry { line: line_idx, group_name: key, members, prefix, sep })
}

/// 解析仓库段的一行，如 `liyixin = rw` 或 `@sw_manager = rw`
fn parse_auth_entry(line_idx: usize, line: &str) -> Option<AuthEntry> {
    let eq_pos = line.find('=')?;
    let principal = line[..eq_pos].trim().to_string();
    let permission = line[eq_pos + 1..].trim().to_string();
    if principal.is_empty() || permission.is_empty() {
        return None;
    }
    Some(AuthEntry { line: line_idx, principal, permission })
}

// ─────────────────────────────────────────────────────────────
// 单元测试
// ─────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    /// 读取 subversion_edge_modify_tool 的测试数据
    fn simple_txt() -> String {
        std::fs::read_to_string(
            "../subversion_edge_modify_tool/test_data/simple.txt",
        )
        .expect("找不到 simple.txt，请确认路径")
    }

    #[test]
    fn test_parse_real_users_excludes_group_refs() {
        let content = simple_txt();
        let model = AuthzModel::parse(&content);
        let users = model.real_users();

        // 真人账号应存在
        assert!(users.contains("liyixin"), "liyixin 应在列表中");
        assert!(users.contains("makun"), "makun 应在列表中");
        assert!(users.contains("swjenkins"), "swjenkins 应在列表中");

        // @组名 不应出现
        assert!(!users.contains("@sw_manager"), "@sw_manager 不应在用户列表");
        assert!(!users.contains("@sw_app_1"), "@sw_app_1 不应在用户列表");
    }

    #[test]
    fn test_lookup_repos_and_groups() {
        let content = simple_txt();
        let model = AuthzModel::parse(&content);
        let detail = model.lookup("liyixin");

        // liyixin 在 sw_app_3 组
        assert!(detail.groups.contains(&"sw_app_3".to_string()), "liyixin 应属于 sw_app_3");
    }

    #[test]
    fn test_remove_user_from_repo() {
        let content = simple_txt();
        let mut model = AuthzModel::parse(&content);

        let req = PruneRequest {
            user: "chenyang".to_string(),
            repos_to_remove: vec![
                "[softwarerepo:/products/auto/guide/guide_works_rackposition/rpc_demo]"
                    .to_string(),
            ],
            groups_to_remove: vec![],
        };

        let report = model.apply_prune(&[req]);
        let new_content = model.serialize();

        // 应被删除
        assert!(!new_content.contains("chenyang = r"), "chenyang 的仓库权限行应已删除");
        // 其他用户保留
        assert!(new_content.contains("makun = r"), "makun 的权限不应被动");

        // 影响报告
        assert_eq!(report.repo_removals.len(), 1);
        assert!(report.group_removals.is_empty());
    }

    #[test]
    fn test_remove_member_from_group_preserves_format() {
        let content = simple_txt();
        let mut model = AuthzModel::parse(&content);

        let req = PruneRequest {
            user: "liyixin".to_string(),
            repos_to_remove: vec![],
            groups_to_remove: vec!["sw_app_3".to_string()],
        };

        let report = model.apply_prune(&[req]);
        let new_content = model.serialize();

        // liyixin 应从 sw_app_3 成员列表消失
        let line = new_content
            .lines()
            .find(|l| l.starts_with("sw_app_3="))
            .expect("sw_app_3 行应存在");
        assert!(!line.contains("liyixin"), "liyixin 应从 sw_app_3 移除");
        // 其他成员保留
        assert!(line.contains("gaowei"), "gaowei 应保留");
        assert!(line.contains("makun"), "makun 应保留");

        // 影响报告
        assert_eq!(report.group_removals.len(), 1);
        assert_eq!(report.group_removals[0], ("liyixin".to_string(), "sw_app_3".to_string()));
    }

    #[test]
    fn test_empty_section_kept_after_all_users_removed() {
        // 构造一个只有一个用户的仓库段，删除后段头仍保留
        let content = "[softwarerepo:/only]\ntest_user = rw\n\n[softwarerepo:/other]\nother_user = r\n";
        let mut model = AuthzModel::parse(content);
        let req = PruneRequest {
            user: "test_user".to_string(),
            repos_to_remove: vec!["[softwarerepo:/only]".to_string()],
            groups_to_remove: vec![],
        };
        model.apply_prune(&[req]);
        let out = model.serialize();

        // 段头保留
        assert!(out.contains("[softwarerepo:/only]"), "空段头应保留");
        // 权限行消失
        assert!(!out.contains("test_user"), "test_user 权限行应已删除");
        // 另一段不受影响
        assert!(out.contains("other_user = r"), "other_user 不应受影响");
    }

    #[test]
    fn test_parse_preserves_comments() {
        let content = simple_txt();
        let mut model = AuthzModel::parse(&content);

        let req = PruneRequest {
            user: "liyixin".to_string(),
            repos_to_remove: vec![],
            groups_to_remove: vec!["sw_app_3".to_string()],
        };
        model.apply_prune(&[req]);
        let out = model.serialize();

        assert!(out.contains("# 软件部团队"), "中文注释应保留");
        assert!(out.contains("# 5c pro"), "5c pro 注释应保留");
    }

    #[test]
    fn test_crlf_roundtrip() {
        let lf_content = simple_txt();
        let crlf_content = lf_content.replace('\n', "\r\n");

        let model = AuthzModel::parse(&crlf_content);
        let out = model.serialize();

        assert!(out.contains("\r\n"), "CRLF 应被保留");
        assert!(!out.starts_with('\n'), "不应开头多出 LF");
    }

    #[test]
    fn test_bom_roundtrip() {
        let content = simple_txt();
        let bom_content = format!("\u{FEFF}{}", content);

        let model = AuthzModel::parse(&bom_content);
        let out = model.serialize();

        assert!(out.starts_with('\u{FEFF}'), "BOM 应被保留");
    }

    #[test]
    fn test_duplicate_user_line_all_removed() {
        let content =
            "[softwarerepo:/dup]\ndup_user = rw\ndup_user = r\nother = r\n";
        let mut model = AuthzModel::parse(content);
        let req = PruneRequest {
            user: "dup_user".to_string(),
            repos_to_remove: vec!["[softwarerepo:/dup]".to_string()],
            groups_to_remove: vec![],
        };
        model.apply_prune(&[req]);
        let out = model.serialize();

        assert!(!out.contains("dup_user"), "重复的用户授权行应全部删除");
        assert!(out.contains("other = r"), "其他用户不受影响");
    }

    #[test]
    fn test_impact_report_counts() {
        let content = simple_txt();
        let mut model = AuthzModel::parse(&content);

        let reqs = vec![
            PruneRequest {
                user: "liyixin".to_string(),
                repos_to_remove: vec![],
                groups_to_remove: vec!["sw_app_3".to_string()],
            },
            PruneRequest {
                user: "chenyang".to_string(),
                repos_to_remove: vec![
                    "[softwarerepo:/products/auto/guide/guide_works_rackposition/rpc_demo]"
                        .to_string(),
                ],
                groups_to_remove: vec![],
            },
        ];

        let report = model.apply_prune(&reqs);
        assert_eq!(report.repo_removals.len(), 1, "应有 1 条仓库删除记录");
        assert_eq!(report.group_removals.len(), 1, "应有 1 条组成员删除记录");
    }
}
