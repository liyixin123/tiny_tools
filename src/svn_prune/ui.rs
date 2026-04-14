// SVN 权限瘦身 Tab 的 UI 构建。
// 数据类型为 SvnPruneState（通过 app.rs 里 AppState::svn_prune lens 接入）。

use druid::widget::{
    Button, Checkbox, Either, Flex, Label, List, Scroll, SizedBox, TextBox,
};
use druid::{Color, Env, EventCtx, Widget, WidgetExt};

use crate::app::AppState;
use crate::common::message::MessageType;
use crate::svn_prune::actions;
use crate::svn_prune::authz::AuthzModel;
use crate::svn_prune::state::{PermissionRow, PruneStage, SvnPruneState, UserRow};

const BTN_W: f64 = 130.0;

pub fn build_prune_tab() -> impl Widget<AppState> {
    let msg_label = Label::dynamic(|data: &SvnPruneState, _: &Env| {
        if data.busy {
            data.busy_hint.clone()
        } else if data.message.is_empty() {
            "请点击【加载远程】开始".to_string()
        } else {
            data.message.clone()
        }
    })
    .with_text_size(24.0)
    .env_scope(|env, data: &SvnPruneState| {
        env.set(druid::theme::TEXT_COLOR, data.message_color.clone());
    })
    .lens(druid::lens::Map::new(
        |s: &AppState| s.svn_prune.clone(),
        |s: &mut AppState, inner| s.svn_prune = inner,
    ));

    // ── 顶部操作条 ──────────────────────────────────────────
    let btn_load = Button::<AppState>::new("加载远程")
        .fix_width(BTN_W)
        .on_click(|ctx: &mut EventCtx, data: &mut AppState, _| {
            data.svn_prune.set_busy("正在拉取远程 authz...");
            data.svn_prune.stage = PruneStage::Loading;
            let sink = ctx.get_external_handle();
            actions::load_remote(sink);
        });

    let btn_open_backup = Button::<AppState>::new("查看备份")
        .fix_width(BTN_W)
        .on_click(|_, _, _| {
            if let Some(dir) = subversion_edge_modify_tool::start_init::get_backups_dir() {
                let _ = crate::common::util::open_folder(dir.to_str().unwrap_or(""));
            }
        });

    let loaded_at_label = Label::dynamic(|data: &AppState, _: &Env| {
        if data.svn_prune.loaded_at.is_empty() {
            String::new()
        } else {
            format!("已加载: {}", data.svn_prune.loaded_at)
        }
    });

    let top_bar = Flex::row()
        .with_child(btn_load)
        .with_spacer(8.0)
        .with_child(btn_open_backup)
        .with_flex_spacer(1.0)
        .with_child(loaded_at_label);

    // ── 左栏：用户列表 ───────────────────────────────────────
    let search_box = TextBox::<String>::new()
        .with_placeholder("搜索用户名...")
        .expand_width()
        .lens(druid::lens::Map::new(
            |s: &AppState| s.svn_prune.search.clone(),
            |s: &mut AppState, v| {
                s.svn_prune.search = v;
                s.svn_prune.apply_search_filter();
            },
        ));

    let btn_select_all = Button::<AppState>::new("全选")
        .on_click(|_, data: &mut AppState, _| {
            for row in data.svn_prune.all_users.iter_mut() {
                if row.visible {
                    row.selected = true;
                }
            }
            rebuild_detail(data);
        });

    let btn_deselect_all = Button::<AppState>::new("全不选")
        .on_click(|_, data: &mut AppState, _| {
            for row in data.svn_prune.all_users.iter_mut() {
                row.selected = false;
            }
            data.svn_prune.detail_rows.clear();
        });

    let select_bar = Flex::row()
        .with_flex_child(btn_select_all, 1.0)
        .with_spacer(4.0)
        .with_flex_child(btn_deselect_all, 1.0);

    // 用户列表项：Checkbox + 名称；不可见行用空 SizedBox 折叠
    let user_list = List::new(|| {
        Either::new(
            |row: &UserRow, _| !row.visible,
            SizedBox::empty(),
            Flex::row()
                .with_child(Checkbox::new("").lens(UserRow::selected))
                .with_flex_child(
                    Label::dynamic(|row: &UserRow, _: &Env| row.name.clone()),
                    1.0,
                ),
        )
    })
    .lens(druid::lens::Map::new(
        |s: &AppState| s.svn_prune.all_users.clone(),
        |s: &mut AppState, v| {
            s.svn_prune.all_users = v;
            rebuild_detail(s);
        },
    ));

    let left_col = Flex::column()
        .with_child(search_box)
        .with_spacer(4.0)
        .with_child(select_bar)
        .with_spacer(4.0)
        .with_flex_child(Scroll::new(user_list).vertical(), 1.0);

    // ── 右栏：权限明细 ───────────────────────────────────────
    let selected_count_label = Label::dynamic(|data: &AppState, _: &Env| {
        let n = data.svn_prune.all_users.iter().filter(|r| r.selected).count();
        if n == 0 {
            "请在左侧勾选用户".to_string()
        } else {
            format!("已选 {} 位用户 — 以下权限将被删除（取消勾选可保留）", n)
        }
    });

    // 权限明细列表项
    let detail_list = List::new(|| {
        Flex::row()
            .with_child(Checkbox::new("").lens(PermissionRow::checked))
            .with_spacer(4.0)
            .with_flex_child(
                Label::dynamic(|row: &PermissionRow, _: &Env| {
                    format!("[{}]  {}", row.user, row.label)
                }),
                1.0,
            )
    })
    .lens(druid::lens::Map::new(
        |s: &AppState| s.svn_prune.detail_rows.clone(),
        |s: &mut AppState, v| s.svn_prune.detail_rows = v,
    ));

    let right_col = Flex::column()
        .with_child(selected_count_label)
        .with_spacer(4.0)
        .with_flex_child(Scroll::new(detail_list).vertical(), 1.0);

    // ── 左右分栏主体 ─────────────────────────────────────────
    let main_body = Flex::row()
        .with_flex_child(left_col, 3.0)
        .with_spacer(8.0)
        .with_flex_child(right_col, 7.0);

    // ── 预览影响文本面板 ─────────────────────────────────────
    let impact_panel = Scroll::new(
        Label::dynamic(|data: &AppState, _: &Env| {
            if data.svn_prune.impact_text.is_empty() {
                "点击「预览影响」查看将删除的内容".to_string()
            } else {
                data.svn_prune.impact_text.clone()
            }
        })
        .with_line_break_mode(druid::widget::LineBreaking::WordWrap),
    )
    .vertical();

    // ── 底部按钮条 ───────────────────────────────────────────
    let btn_preview = Button::<AppState>::new("预览影响")
        .fix_width(BTN_W)
        .disabled_if(|data: &AppState, _| {
            data.svn_prune.busy
                || data.svn_prune.stage == PruneStage::Idle
                || data.svn_prune.stage == PruneStage::Loading
                || data.svn_prune.selected_users().is_empty()
        })
        .on_click(|_, data: &mut AppState, _| {
            let reqs = build_prune_requests(data);
            if reqs.is_empty() {
                data.svn_prune
                    .set_message("没有勾选任何要删除的权限", MessageType::Info);
                return;
            }
            use crate::svn_prune::authz::AuthzModel;
            let mut model = AuthzModel::parse(&data.svn_prune.raw_authz);
            let report = model.apply_prune(&reqs);
            data.svn_prune.impact_text = report.summary_text();
            if report.is_empty() {
                data.svn_prune
                    .set_message("所选用户无任何权限需要删除", MessageType::Info);
            } else {
                data.svn_prune
                    .set_message("预览已生成，请查看下方详情", MessageType::Info);
            }
        });

    let btn_gen_preview = Button::<AppState>::new("生成预览")
        .fix_width(BTN_W)
        .disabled_if(|data: &AppState, _| {
            data.svn_prune.busy
                || data.svn_prune.stage == PruneStage::Idle
                || data.svn_prune.stage == PruneStage::Loading
                || data.svn_prune.impact_text.is_empty()
        })
        .on_click(|ctx: &mut EventCtx, data: &mut AppState, _| {
            let reqs = build_prune_requests(data);
            if reqs.is_empty() {
                data.svn_prune
                    .set_message("没有勾选任何要删除的权限", MessageType::Info);
                return;
            }
            data.svn_prune.set_busy("正在生成预览文件...");
            data.svn_prune.stage = PruneStage::GeneratingPreview;
            let raw = data.svn_prune.raw_authz.clone();
            let sink = ctx.get_external_handle();
            actions::gen_preview(raw, reqs, sink);
        });

    let btn_apply = Button::<AppState>::new("应用到服务器")
        .fix_width(BTN_W + 20.0)
        .disabled_if(|data: &AppState, _| {
            data.svn_prune.busy
                || data.svn_prune.impact_text.is_empty()
                || data.svn_prune.stage == PruneStage::Applying
        })
        .on_click(|_, data: &mut AppState, _| {
            let n_repos = data
                .svn_prune
                .detail_rows
                .iter()
                .filter(|r| r.checked && r.kind == crate::svn_prune::state::PermKind::Repo)
                .count();
            let n_groups = data
                .svn_prune
                .detail_rows
                .iter()
                .filter(|r| r.checked && r.kind == crate::svn_prune::state::PermKind::Group)
                .count();
            let users: Vec<String> = data
                .svn_prune
                .all_users
                .iter()
                .filter(|r| r.selected)
                .map(|r| r.name.clone())
                .collect();
            data.svn_prune.confirm_summary = format!(
                "即将从服务器删除以下操作（不可撤销）：\n\n\
                 用户: {}\n\
                 删除仓库权限: {} 条\n\
                 从用户组移除: {} 处\n\n\
                 程序将重新拉取最新 authz 后执行删除，\
                 并在本地 backups 目录生成 apply 备份。\n\
                 确认继续？",
                users.join(", "),
                n_repos,
                n_groups,
            );
            data.svn_prune.confirm_open = true;
        });

    let bottom_bar = Flex::row()
        .with_child(btn_preview)
        .with_spacer(8.0)
        .with_child(btn_gen_preview)
        .with_spacer(8.0)
        .with_flex_spacer(1.0)
        .with_child(btn_apply);

    // ── 二次确认弹窗（Either 覆盖层）────────────────────────
    let confirm_panel = Either::new(
        |data: &AppState, _| data.svn_prune.confirm_open,
        build_confirm_panel(),
        SizedBox::empty(),
    );

    // ── 总组合 ────────────────────────────────────────────────
    Flex::column()
        .with_child(msg_label)
        .with_spacer(4.0)
        .with_child(top_bar)
        .with_spacer(4.0)
        .with_flex_child(main_body, 5.0)
        .with_spacer(4.0)
        .with_flex_child(impact_panel, 2.0)
        .with_spacer(4.0)
        .with_child(bottom_bar)
        .with_child(confirm_panel)
}

// ── 二次确认弹窗内容 ─────────────────────────────────────────

fn build_confirm_panel() -> impl Widget<AppState> {
    let summary = Label::dynamic(|data: &AppState, _: &Env| {
        data.svn_prune.confirm_summary.clone()
    })
    .with_line_break_mode(druid::widget::LineBreaking::WordWrap);

    let btn_cancel = Button::<AppState>::new("取消")
        .on_click(|_, data: &mut AppState, _| {
            data.svn_prune.confirm_open = false;
        });

    let btn_confirm = Button::<AppState>::new("确认应用到服务器")
        .on_click(|ctx: &mut EventCtx, data: &mut AppState, _| {
            data.svn_prune.confirm_open = false;
            let reqs = build_prune_requests(data);
            if reqs.is_empty() {
                data.svn_prune
                    .set_message("没有要删除的权限，操作取消", MessageType::Info);
                return;
            }
            data.svn_prune.set_busy("正在应用到服务器，请稍候...");
            data.svn_prune.stage = PruneStage::Applying;
            let sink = ctx.get_external_handle();
            actions::apply_remote(reqs, sink);
        });

    Flex::column()
        .with_child(
            Label::new("⚠ 请确认操作")
                .with_text_color(Color::RED)
                .with_text_size(20.0),
        )
        .with_spacer(8.0)
        .with_child(summary)
        .with_spacer(12.0)
        .with_child(
            Flex::row()
                .with_flex_spacer(1.0)
                .with_child(btn_cancel)
                .with_spacer(8.0)
                .with_child(btn_confirm),
        )
        .padding(16.0)
        .background(Color::grey8(40))
        .border(Color::RED, 2.0)
}

// ── 辅助函数 ──────────────────────────────────────────────────

/// 根据右侧明细勾选状态构造 PruneRequest 列表。
fn build_prune_requests(data: &AppState) -> Vec<crate::svn_prune::authz::PruneRequest> {
    use crate::svn_prune::state::PermKind;
    use std::collections::HashMap;

    let mut by_user: HashMap<String, (Vec<String>, Vec<String>)> = HashMap::new();

    for row in data.svn_prune.detail_rows.iter() {
        if !row.checked {
            continue;
        }
        let entry = by_user.entry(row.user.clone()).or_default();
        match row.kind {
            PermKind::Repo => entry.0.push(row.key.clone()),
            PermKind::Group => entry.1.push(row.key.clone()),
        }
    }

    by_user
        .into_iter()
        .map(|(user, (repos, groups))| crate::svn_prune::authz::PruneRequest {
            user,
            repos_to_remove: repos,
            groups_to_remove: groups,
        })
        .collect()
}

/// 从 all_users 的当前选中状态重建右侧 detail_rows（通过 authz 查询）。
fn rebuild_detail(data: &mut AppState) {
    let raw = data.svn_prune.raw_authz.clone();
    if raw.is_empty() {
        return;
    }
    let model = AuthzModel::parse(&raw);
    data.svn_prune.rebuild_detail_rows(&model);
}
