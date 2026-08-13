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
mise run check         # Rust 编译检查
mise run clippy        # Rust clippy
mise run lint          # ESLint
mise run gen-types     # 从 Rust 导出 TS 类型
mise run format        # Prettier 格式化前端
mise run format-check  # Prettier 检查前端格式
mise run build         # 构建生产包
```

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
