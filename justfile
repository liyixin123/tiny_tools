set shell := ["powershell.exe", "-NoProfile", "-Command"]

# 默认任务：显示帮助
default:
    just --list

# 构建 release 版本
build:
    cargo build --release

# 安装到系统（cargo install）
install:
    cargo install --path .

# 构建并复制到指定目录（可通过 dest 参数覆盖）
deploy dest="$env:LOCALAPPDATA\\Programs\\tiny_tools":
    cargo build --release
    New-Item -ItemType Directory -Force -Path "{{dest}}" | Out-Null
    Copy-Item -Force target\release\tiny_tools.exe "{{dest}}\tiny_tools.exe"
    Write-Host "已安装到 {{dest}}"

# 运行测试
test:
    cargo test

# 清理构建产物
clean:
    cargo clean
