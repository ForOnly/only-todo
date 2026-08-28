# Only Todo

基于 Tauri 2 + Rust + Vue 3 的本地优先桌面任务助手。

## 文档

- [doc/产品需求.md](doc/产品需求.md) — 产品规格（WHAT）
- [doc/技术设计.md](doc/技术设计.md) — 技术设计（HOW）
- [doc/项目进度.md](doc/项目进度.md) — 开发进度
- [doc/架构蓝图.md](doc/架构蓝图.md) — 可复用架构标准（新项目落地用）

## 运行环境

本项目使用 [mise](https://mise.jdx.dev/) 管理工具链，版本见 [mise.toml](mise.toml)。

| 工具 | 版本 |
|------|------|
| Node.js | 24 |
| Rust | stable |

系统依赖（Windows）：Visual Studio Build Tools（C++）、WebView2

### 首次 setup

```bash
mise trust
mise install
mise run install
```

### 常用命令

```bash
mise run dev           # 开发
mise run ci            # 本地复现 GitHub CI
mise run check         # Rust 编译检查
mise run clippy        # Rust clippy
mise run test          # Rust 单元测试
mise run lint          # ESLint
mise run build-fe      # 前端类型检查 + Vite 生产构建
mise run gen-types     # 从 Rust 导出 TS 类型
mise run gen-types-check  # 导出后检查 generated 无漂移
mise run format        # Prettier 格式化前端
mise run format-check  # Prettier 检查前端格式
mise run build         # 构建生产包（Tauri）
```

## 发布

Windows 安装包由 [`.github/workflows/release.yml`](.github/workflows/release.yml) 在 GitHub Actions 上构建，产物上传到 **已发布** 的 GitHub Release（含 `latest.json` 供应用内更新）。

### 触发方式

- **自动**：推送 `v*` tag（如 `v0.2.0`）
- **手动**：GitHub → Actions → Release → Run workflow

### 发版前

1. 合并到 `main`，确认 CI 通过
2. 同步三处版本号：`package.json`、`src-tauri/Cargo.toml`、`src-tauri/tauri.conf.json`（三者必须相同）
3. 打 tag：**必须**为 `v` + 上述版本号（如版本 `1.0.0` → tag `v1.0.0`）。CI 会校验 tag 与 `tauri.conf.json` 一致，不一致则构建失败
4. 提交后推送 tag：

```bash
git tag v1.0.0
git push origin v1.0.0
```

5. Actions 完成后，在 GitHub Releases 检查安装包与 `latest.json`（Release 名称、产物版本、`latest.json` 中的 `version` 均应与 tag 一致）

### 仓库设置

在 **Settings → Actions → General → Workflow permissions** 勾选 **Read and write permissions**，否则创建 Release 可能失败。

在 **Settings → Secrets and variables → Actions** 配置 Tauri 更新签名（与 `tauri.conf.json` 中公钥配对）：

| Secret | 说明 |
|--------|------|
| `TAURI_SIGNING_PRIVATE_KEY` | **同一次** `tauri signer generate` 产出的私钥**全文**（须含 `untrusted comment` 行；勿加引号、勿只贴 Base64 中间段） |
| `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` | 生成密钥时设置的密码（无密码则留空或不设；有密码必须与生成时一致） |

本地生成示例（私钥勿提交仓库）：

```bash
npm run tauri signer generate -- -w "$HOME/.tauri/only-todo.key"
```

将**同一次**生成的 `.pub` 内容写入 `src-tauri/tauri.conf.json` → `plugins.updater.pubkey`。私钥与公钥必须成对；换密钥时两边一起更新，否则 CI 签名或客户端校验会失败。

发版前可本地试签（确认 Secret 内容与密码正确）：

```powershell
$env:TAURI_SIGNING_PRIVATE_KEY = Get-Content $HOME\.tauri\only-todo.key -Raw
$env:TAURI_SIGNING_PRIVATE_KEY_PASSWORD = "<生成时的密码，无则空>"
npm run tauri -- signer sign .\some-temp-file.bin
```

`release.yml` 在完整构建前会对临时文件做同样的 dry-run；失败时不会进入 8 分钟级 `tauri-action` 编译。

未签名安装包在 Windows 上可能触发 SmartScreen 提示；Authenticode 代码签名可独立后续配置。

**应用内更新**：首个带 updater 的版本（如 `0.2.0`）之前的用户需手动安装一次；之后可在设置页或启动时检测更新，确认后自动下载、安装并重启。

CD 产物为 NSIS 安装包（`--bundles nsis`）及 updater 签名产物；本地完整 Tauri 构建仍可用 `mise run build`（含 MSI 等，需 WiX）。

### 约定

- 不要依赖系统全局 Node/Rust，进入项目目录后须先 `mise install`
- 脚本或未激活 mise 的 shell 中使用：`mise exec -- npm run tauri dev`
- Vue 文件跳转：安装推荐扩展 `Vue.volar`（不要装 Vetur），并把 TypeScript 选为工作区版本（`node_modules/typescript/lib`）
- Rust 文件跳转：安装推荐扩展 `rust-lang.rust-analyzer`；crate 在 `src-tauri/`。本地 `.vscode/settings.json`（不进 git）设置 `rust-analyzer.linkedProjects` 为 `["./src-tauri/Cargo.toml"]`，然后执行 **Rust Analyzer: Restart server**

## 项目结构

```
src/           Vue 3 前端（目录说明见 doc/技术设计.md §1.6）
src-tauri/     Rust 后端（分层模块）
doc/           需求与设计文档
mise.toml      工具链与任务入口
```
