# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## 项目简介

这是一个 Windows 桌面小工具，基于 [druid](https://github.com/linebender/druid) GUI 框架构建，有两个主要功能 Tab：
- **SVN权限开通**：将申请表格中的完整 SVN 地址转换为权限路径，写入本地或远程 authz 文件
- **SVN权限瘦身**：从远程拉取 authz，批量删除指定用户的仓库权限及用户组成员资格

本地依赖：`subversion_edge_modify_tool`（路径：`../subversion_edge_modify_tool`），负责实际读写 SVN authz 文件，需提前在本地准备好。

## 常用命令

```bash
# 构建 release 版本
cargo build --release
# 或
just build

# 运行所有测试
cargo test

# 运行单个测试（模块::测试名）
cargo test test_convert_address
cargo test svn_prune::authz::tests::test_remove_user_from_repo

# 安装到当前用户的 Programs 目录（Windows）
just deploy

# 清理构建产物
cargo clean
```

## 架构说明

```
src/
├── main.rs              # 入口、窗口、字体、Tabs 装配；SVN开通页 UI 及业务逻辑
├── app.rs               # 顶层 AppState、AppDelegate(PruneDelegate)、异步 Selector 定义
├── tests.rs             # main.rs 相关单元测试
├── pic_uploader.rs      # 图片上传（暂未接入主界面）
├── common/
│   ├── message.rs       # MessageType { Info, Error }，统一 set_message 颜色
│   ├── runtime.rs       # OnceLock<tokio::Runtime> 单例，全局共用
│   └── util.rs          # open_folder()（Windows: explorer）
└── svn_prune/
    ├── authz.rs         # authz 解析器（纯逻辑，可单测）
    ├── state.rs         # SvnPruneState / UserRow / PermissionRow / PruneStage
    ├── actions.rs       # 异步动作：load_remote / gen_preview / apply_remote
    └── ui.rs            # 权限瘦身 Tab UI
```

### 顶层状态

`AppState { svn_add: SVNAddress, svn_prune: SvnPruneState }` 是整个应用的根状态，实现 `druid::Data + Lens`。  
`SVNAddress`（通过 `pub(crate) type SvnAddState = SVNAddress` 别名供 app.rs 引用）是开通页状态。

### 异步模式（权限瘦身 Tab）

所有网络操作不阻塞 UI，统一用 `ExtEventSink + Selector` 回调：

```
按钮 on_click
  → data.svn_prune.set_busy(...)
  → actions::xxx(ctx.get_external_handle())     // 立即返回
      └─ common::runtime::rt().spawn(async {
             let result = await 网络调用;
             sink.submit_command(PRUNE_xxx_DONE, result, Target::Auto);
         })
          ↓
PruneDelegate::command(PRUNE_xxx_DONE)           // UI 线程回调
  → 回写 state，busy=false，set_message
```

三个 Selector 定义在 `app.rs`：`PRUNE_LOAD_DONE` / `PRUNE_PREVIEW_DONE` / `PRUNE_APPLY_DONE`。

### authz 解析器（`svn_prune/authz.rs`）

采用**最小改动策略**：保存 `raw_lines: Vec<String>` 原文，删除操作只记录行索引（`tombstones`）或改写内容（`overrides`），序列化时过滤/替换，保留注释、空行、CRLF、BOM。

关键方法：
- `AuthzModel::parse(content)` — 检测 BOM/CRLF，扫描各 section
- `real_users()` — 排除 `@group` 引用，返回真人账号集合
- `lookup(user)` — 查询某用户的仓库授权 + 所属用户组
- `apply_prune(reqs)` — 填充 tombstones/overrides，返回 ImpactReport
- `serialize()` — 输出改写后的 authz 文本

### 开通页地址转换流程（`main.rs`）

`SVNAddress::update()` 驱动：
1. `extract_name()` — 正则提取 "SVN账号名称" 后一行
2. `extract_substrings_containing_base_url()` — 过滤含 `BASE_URL` 的片段
3. `extract_permissions()` — 从末尾找第一个含「只读/读写」的行
4. `convert_address()` → `replace_str()` — 转换为 `[softwarerepo:/path]` 格式

`BASE_URL` 硬编码为 `http://172.17.102.22:18080/svn/softwarerepo`。

### 备份目录

`subversion_edge_modify_tool::start_init::get_backups_dir()` 返回统一备份路径（`~/<数据目录>/svn_user_auth/backups/`），两个 Tab 共用。

## 注意事项

- 目标平台为 Windows（`#![windows_subsystem = "windows"]`）
- 远程操作需在程序安装目录配置 `.env`，包含 `USERNAME` 和 `PASSWORD`
- 瘦身功能支持 `DRY_RUN=1` 环境变量（只写本地，不推服务器）
- druid 依赖来自 git，不是 crates.io 版本；`im::Vector<T>` 用于 druid Data 兼容的列表状态
- `post_content` 需先调用 `get_file_content_from_remote(false)` 填充 SYNCHRONIZER_TOKEN，`apply_remote` 中已保证顺序
