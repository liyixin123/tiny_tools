# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## 项目简介

这是一个 Windows 桌面小工具，基于 [druid](https://github.com/linebender/druid) GUI 框架构建。主要功能是将 SVN 申请表格中带 IP 的完整地址转换为简短的权限路径格式，并可直接写入本地或远程 SVN 服务器的权限文件。

本地依赖：`subversion_edge_modify_tool`（路径：`../subversion_edge_modify_tool`），负责实际读写 SVN authz 文件，需提前在本地准备好。

## 常用命令

```bash
# 构建 release 版本
cargo build --release
# 或
just build

# 运行所有测试
cargo test
# 或
just test

# 运行单个测试
cargo test test_convert_address

# 安装到当前用户的 Programs 目录（Windows）
just deploy

# 清理构建产物
cargo clean
```

## 架构说明

所有逻辑集中在 `src/` 下三个文件：

| 文件 | 职责 |
|---|---|
| `src/main.rs` | 应用入口、GUI 构建、核心业务逻辑 |
| `src/tests.rs` | 单元测试 |
| `src/pic_uploader.rs` | 图片上传功能（暂未接入主界面） |

### 核心数据结构

`SVNAddress`（实现 `druid::Data` + `druid::Lens`）是整个应用的状态：

- `old: String` — 用户粘贴的原始申请文本
- `new_addrs: Vector<TextBoxData>` — 转换后的路径列表（可点击复制）
- `names: String` — 提取出的 SVN 账号名称
- `read_write: bool` — 读写权限标志
- `backup_path: String` — 备份目录路径
- `message / message_color` — 状态提示信息

### 地址转换流程

`SVNAddress::update()` 驱动整个转换流程：

1. `extract_name()` — 用正则从申请文本中提取 "SVN账号名称" 后一行的账号
2. `extract_substrings_containing_base_url()` — 按空白字符分割，过滤含 `BASE_URL` 的片段
3. `extract_permissions()` — 从末尾往前找第一个含「只读/读写」的行
4. `convert_address()` → `replace_str()` — 将完整 URL 转换为 `[softwarerepo:/path/to/repo]` 格式，同时去掉括号注释和末尾斜杠

### 关键常量

- `BASE_URL`：硬编码为 `http://172.17.102.22:18080/svn/softwarerepo`
- `SEPARATOR`：地址末尾分隔符（`/` 和空格）
- `REGEXES`：用于剔除地址尾部括号注释的正则列表

### 权限写入

`generate_permissions()` 将转换结果组装成 `subversion_edge_modify_tool::permissions::Permissions` 结构体，再由：
- `apply_to_local()` — 写入本地 authz 文件（含备份）
- `apply_to_remote()` — 通过 HTTP 写入远程 SVN Edge 服务器

两者均通过 `tokio::runtime::Runtime::new().unwrap().block_on(...)` 在点击事件中同步调用异步函数。

## 注意事项

- 目标平台为 Windows（`#![windows_subsystem = "windows"]` 隐藏控制台窗口）
- 远程修改需要在程序安装目录下配置 `.env` 文件，包含 `USERNAME` 和 `PASSWORD`
- druid 依赖来自 git，不是 crates.io 版本
