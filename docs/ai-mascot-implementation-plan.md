# SSMT4 AI 看板娘 — 完整实施清单

## 项目现状

- **技术栈**：Tauri v2 (Rust + Vue 3 + TypeScript)
- **前端**：Vue 3 + Vue Router + Pinia + Element Plus + Vue i18n + Three.js
- **后端**：30+ Tauri 命令（游戏启动、MOD 管理、模型/纹理提取、压缩、回收站、VS Check）
- **页面**：Home、GameLibrary、ModsManagement、GameBanana、NexusMods、WorkPage、MarkTexture、Settings

---

## 第一阶段：MCP 工具服务层（Rust 后端）

> **目标**：将现有 Tauri 命令包装为标准 MCP (Model Context Protocol) 接口，使 AI 可以自动发现和调用所有工具。

| # | 任务 | 文件 | 说明 |
|---|------|------|------|
| 1.1 | 新建 MCP 模块骨架 | `src-tauri/src/mcp/mod.rs` | 模块入口，声明子模块 |
| 1.2 | MCP 协议类型定义 | `src-tauri/src/mcp/types.rs` | JSON-RPC 2.0 消息结构：`Request`、`Response`、`Tool`、`ToolCall`、`CallToolResult` 等 |
| 1.3 | MCP Server 实现 | `src-tauri/src/mcp/server.rs` | 实现 `initialize`、`tools/list`、`tools/call`、`resources/list` 等标准 MCP 方法；基于 HTTP/SSE 或本地 WebSocket |
| 1.4 | 工具注册表 | `src-tauri/src/mcp/tools.rs` | 为 `lib.rs` 中所有 Tauri 命令编写 MCP Tool schema（名称、描述、参数 JSON Schema），实现 `ToolRegistry` |
| 1.5 | 工具执行分发器 | `src-tauri/src/mcp/tools.rs` | `tools/call` 请求 → 按名称匹配 → 反序列化参数 → 调用对应 Rust 函数 → 序列化结果返回 |
| 1.6 | 启动 MCP Server | `src-tauri/src/lib.rs` (setup) | 在 `setup()` 中启动 MCP Server，监听 `127.0.0.1` 随机端口，端口号写入 Store 供 AI Agent 读取 |
| 1.7 | 添加依赖 | `src-tauri/Cargo.toml` | 评估使用 `rmcp` crate 或手工实现轻量 JSON-RPC；可能需要 `axum` / `tiny_http` 作为 HTTP 层 |

### 第一阶段待映射的命令清单

以下命令需要逐一映射为 MCP Tool：

- `configure_zzmi_launch_settings` — 配置 ZZMI 启动参数
- `configure_wwmi_launch_settings` — 配置 WWMI 启动参数
- `execute_external_program` — 执行外部程序
- `launch_programs` — 启动程序组
- `file_md5` — 计算文件 MD5
- `watch_mods` / `unwatch_mods` — 监听/取消监听 MOD 目录
- `preview_mod_archive` — 预览 MOD 压缩包
- `mod_install_target_exists` — 检查 MOD 安装目标是否存在
- `install_mod_archive` — 安装 MOD 压缩包
- `gamebanana_download_and_install_mod` — 从 GameBanana 下载并安装 MOD
- `cancel_gamebanana_download_and_install_mod` — 取消 GameBanana 下载
- `nexusmods_download_and_install_mod` — 从 NexusMods 下载并安装 MOD
- `cancel_nexusmods_download_and_install_mod` — 取消 NexusMods 下载
- `export_mod_archive` — 导出 MOD 压缩包
- `scan_directory` — 扫描目录
- `get_mod_key_list` — 获取 MOD 键列表
- `mod_library_stream_scan` — MOD 库流式扫描
- `mod_library_scan_group` — MOD 库分组扫描
- `mod_library_refresh_group` — MOD 库刷新分组
- `mod_library_refresh_all` — MOD 库全量刷新
- `mod_library_all_mods` — 获取所有 MOD
- `watch_mod_library` / `unwatch_mod_library` — 监听 MOD 库
- `find_nested_ini_files` — 查找嵌套 INI 文件
- `extract_models_new` — 提取模型
- `full_extract` — 全量提取
- `analyze_draw_ib_submeshes` — 分析 DrawIB 子网格
- `regenerate_draw_ib_component_json` — 重新生成组件 JSON
- `update_vscheck` / `generate_vscheck` — VS 检查
- `extract_deduped_textures` — 提取去重纹理
- `extract_trianglelist_textures` — 提取三角形列表纹理
- `extract_zip_archive` — 解压 ZIP
- `create_rar_archive` — 创建 RAR
- `create_mod_archive` — 创建 MOD 压缩包
- `move_file_to_recycle_bin` / `move_dir_to_recycle_bin` — 回收站操作

---

## 第二阶段：AI 代理后端（Rust）

> **目标**：实现 LLM 调用能力，支持 Function Calling，让 AI 能自主决定调用哪个 MCP 工具。

| # | 任务 | 文件 | 说明 |
|---|------|------|------|
| 2.1 | 新建 AI 模块 | `src-tauri/src/ai/mod.rs` | 模块入口 |
| 2.2 | LLM Provider 抽象层 | `src-tauri/src/ai/providers/mod.rs` | 定义 `LlmProvider` trait：`chat_stream(messages, tools) → Stream<Delta>` |
| 2.3 | OpenAI 兼容 Provider | `src-tauri/src/ai/providers/openai.rs` | 基于 `reqwest` + SSE 实现；支持自定义 endpoint（兼容 Ollama、LM Studio 等） |
| 2.4 | Function Calling 循环 | `src-tauri/src/ai/agent.rs` | AI 返回 tool_call → 通过 MCP 本地调用执行 → 结果回传 AI → 循环直到 AI 生成文本回复 |
| 2.5 | 对话历史管理 | `src-tauri/src/ai/session.rs` | 内存中维护多轮对话上下文、token 估算、自动截断 |
| 2.6 | Tauri 命令封装 | `src-tauri/src/commands/ai_chat.rs` | `send_chat_message`（流式 SSE）、`get_chat_history`、`clear_chat`、`get_available_tools` |
| 2.7 | API Key 配置 | 通过 `tauri-plugin-store` | 持久化：`ai.apiKey`、`ai.model`、`ai.endpoint`、`ai.mascotEnabled` |
| 2.8 | 注册命令 | `src-tauri/src/lib.rs` | 将 `commands/ai_chat.rs` 中的命令注册到 `invoke_handler` |

---

## 第三阶段：AI 对话页面（Vue 前端）

> **目标**：提供一个独立的对话页面，用户可以通过输入框与 AI 交互，AI 可以自动调用工具。

| # | 任务 | 文件 | 说明 |
|---|------|------|------|
| 3.1 | 新建 Pinia Store | `src/store/AiChatStore.ts` | 管理对话历史、流式消息、工具调用状态 |
| 3.2 | 新建对话页面 | `src/views/AiChat/AiChat.vue` | 页面主体布局：消息列表 + 输入框 |
| 3.3 | 消息气泡组件 | `src/components/AiMessageBubble.vue` | 区分用户/AI/工具调用三种类型；AI 消息支持 Markdown 渲染 |
| 3.4 | 工具调用卡片组件 | `src/components/AiToolCallCard.vue` | 可折叠卡片，显示工具名、参数、执行结果 |
| 3.5 | 输入框组件 | `src/components/AiChatInput.vue` | 支持 Enter 发送、Shift+Enter 换行、发送/停止按钮、附件（可选） |
| 3.6 | 流式渲染 Hook | `src/common/useStreamChat.ts` | 封装 Tauri `invoke` + 流式读取逻辑 |
| 3.7 | Markdown 渲染 | `src/components/AiMarkdown.vue` | 基于 `marked` 或轻量实现，支持代码高亮 |
| 3.8 | 注册路由 | `src/router/index.ts` | 添加 `/ai-chat` 路由，设置 `meta: { title: 'AI Chat', requiresGame: false }` |
| 3.9 | 导航入口 | `src/components/TitleBar.vue` 或侧边栏 | 添加"AI 助手"按钮 |
| 3.10 | 空状态引导 | `AiChat.vue` 内 | 首次进入显示欢迎语和示例提问（如"帮我安装一个 Mod"、"分析当前游戏的纹理"） |
| 3.11 | i18n 国际化 | `src/i18n/` | 添加 `aiChat` 命名空间下的中英文翻译 key |
| 3.12 | 设置页面集成 | `src/views/Settings/Settings.vue` | 添加 AI 配置卡片：API Key、Model、Endpoint、看板娘开关 |

---

## 第四阶段：桌面看板娘形象（Vue 前端 + Tauri）

> **目标**：实现一个可交互的桌面宠物角色，能根据 AI 状态切换动画。

| # | 任务 | 文件 | 说明 |
|---|------|------|------|
| 4.1 | 角色资源准备 | `src/assets/mascot/` | 选择/制作看板娘角色：Live2D 模型、Spine 动画，或 CSS 精灵图序列帧 |
| 4.2 | 渲染技术选型 | 评估 | **方案 A**：独立 Tauri WebView 透明窗口（可拖拽到桌面任意位置）；**方案 B**：主窗口内 Canvas/WebGL 渲染 Live2D（跟随页面）；**方案 C**：CSS 精灵图 + 帧动画（最简单） |
| 4.3 | 看板娘组件 | `src/components/MascotPet.vue` | 核心组件：角色渲染 + 动画状态机 + 交互 |
| 4.4 | 动画状态机 | `MascotPet.vue` 内部 | 状态：`idle`（待机呼吸/眨眼）、`listening`（倾听倾斜）、`thinking`（思考动画）、`speaking`（说话嘴型）、`working`（忙碌）、`happy`、`surprised` |
| 4.5 | 对话联动 | `MascotPet.vue` + `AiChatStore` | 监听 Store 状态自动切换动画 |
| 4.6 | 交互功能 | `MascotPet.vue` | 点击 → 打开/关闭聊天面板；拖拽 → 移动位置；右键菜单 → 设置/隐藏 |
| 4.7 | 透明窗口方案 | `src-tauri/tauri.conf.json`（方案 A 时） | 新建 `mascot` 窗口配置：`decorations: false`、`transparent: true`、`alwaysOnTop: true`、`skipTaskbar: true` |
| 4.8 | 语音合成 TTS（可选） | Web Speech API 或 Tauri 插件 | 让看板娘"说话"，配合嘴型动画 |

---

## 第五阶段：集成与联调

> **目标**：全链路打通，确保用户体验流畅。

| # | 任务 | 说明 |
|---|------|------|
| 5.1 | 前后端联调 | 用户输入 → Tauri invoke → AI Agent → MCP 工具调用 → 结果返回 → 流式渲染 |
| 5.2 | 看板娘状态联动测试 | 验证所有动画状态切换正确、无闪烁 |
| 5.3 | 错误处理 | 网络断开、API Key 无效、工具调用超时/失败 → 友好提示 |
| 5.4 | CSP 安全策略 | `tauri.conf.json` 中确认 AI API endpoint 在 `connect-src` 白名单 |
| 5.5 | 性能检查 | 确保流式渲染不导致页面卡顿、看板娘动画帧率稳定 |
| 5.6 | 看板娘窗口通信（方案 A） | Tauri 事件系统 (`emit`/`listen`) 在主窗口和看板娘窗口间同步状态 |
| 5.7 | 打包验证 | `bun run tauri build` 确认新模块正常编译、资源正确打包 |

---

## 关键技术与选型建议

| 层面 | 推荐方案 | 理由 |
|------|---------|------|
| MCP 协议实现 | 手工实现轻量 JSON-RPC over HTTP | 依赖少、可控；`rmcp` crate 也可考虑 |
| LLM 接入 | OpenAI 兼容 API（`reqwest` + SSE streaming） | 项目已有 `reqwest` 依赖；兼容 Ollama、Groq 等 |
| 看板娘渲染 | **方案 A**（独立透明窗口）优先，**方案 C**（CSS 精灵图）作为快速原型 | 独立窗口体验最好，可拖拽到桌面任意位置 |
| Live2D 渲染 | `pixi-live2d-display` 或 Cubism SDK for Web | 如使用 Live2D 模型则需要 |
| 状态管理 | Pinia `AiChatStore` | 项目已使用 Pinia，保持一致 |
| Markdown 渲染 | `marked` + `highlight.js` | 轻量成熟 |

---

## 建议执行顺序

```
Week 1-2： 第一阶段 1.1 → 1.7（MCP 工具层骨架 + 核心工具映射）
Week 2-3： 第二阶段 2.1 → 2.8（AI Agent + Tauri 命令 + Function Calling 联调）
Week 3-4： 第三阶段 3.1 → 3.12（对话页面完整 UI）
Week 4-5： 第四阶段 4.1 → 4.8（看板娘基础形象 + 动画状态机）
Week 5-6： 第五阶段 5.1 → 5.7（全链路联调 + 错误处理 + 打包）
```

---

## 附录：MCP 协议简介

MCP (Model Context Protocol) 是一种 AI 与工具之间的标准通信协议，核心概念：

- **Tool**：一个可被 AI 调用的函数，有名称、描述和 JSON Schema 参数
- **`tools/list`**：返回所有可用工具列表
- **`tools/call`**：执行指定工具并返回结果
- **传输层**：stdio（子进程）或 HTTP + SSE（网络）

在 SSMT4 中，MCP Server 将内嵌在 Tauri 应用中，AI Agent（在同一进程内）通过本地方法调用直接使用 MCP 工具注册表，无需走网络。
