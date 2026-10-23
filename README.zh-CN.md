# gpui-mcp

[English](README.md) | [简体中文](README.zh-CN.md)

[![CI](https://github.com/xxxxzb/gpui-kit-mcp/actions/workflows/ci.yml/badge.svg?branch=gpui-kit)](https://github.com/xxxxzb/gpui-kit-mcp/actions/workflows/ci.yml)
[![License: Apache-2.0](https://img.shields.io/badge/License-Apache--2.0-blue.svg)](LICENSE)
[![Rust 1.96+](https://img.shields.io/badge/rust-1.96%2B-orange.svg)](Cargo.toml)
[![MCP](https://img.shields.io/badge/MCP-compatible-6f42c1.svg)](https://modelcontextprotocol.io)
![Platforms](https://img.shields.io/badge/platform-macOS%20%7C%20Linux%20%7C%20Windows-lightgrey.svg)
[![GitHub stars](https://img.shields.io/github/stars/xxxxzb/gpui-kit-mcp?style=social)](https://github.com/xxxxzb/gpui-kit-mcp/stargazers)

> 这是 `gpui-kit-mcp` fork，跟踪 GPUI Kit 0.7.3（`gpui-pre` 0.3.9）与
> GPUI Kit 0.8.0 的 `gpui-fast` 渲染后端（`gpui-fast` 0.1.10）。
> 分叉差异和上游同步说明见 [FORK.md](FORK.md)。

让 MCP agent 在 [GPUI](https://github.com/zed-industries/zed/tree/main/crates/gpui)
应用里“看得见、点得到”。

Agent 把运行中的界面读成语义树，然后点击、输入、悬停、拖拽、滚动，并在真实状态上等待。
它还可以截图和做像素 diff、录屏、测量帧耗时，以及在应用运行时编辑 HTML 编写的界面。
支持 Windows 11、macOS 和 Linux。

## 安装

安装 MCP 服务，并加入你的 MCP 客户端：

```console
cargo install --git https://github.com/xxxxzb/gpui-kit-mcp --locked gpui-mcp-server
```

```json
{ "mcpServers": { "gpui": { "command": "gpui-mcp" } } }
```

给应用加上桥接和它使用的 GPUI 构建：

```toml
[dependencies]
gpui = "=0.2.2"
gpui_platform = { git = "https://github.com/zed-industries/zed", rev = "16c9aa7ea6d897a8044d9501cde1b295256722f2", features = ["font-kit", "wayland", "x11"] }
gpui-mcp = { git = "https://github.com/xxxxzb/gpui-kit-mcp", branch = "gpui-kit" }

[patch.crates-io]
gpui = { git = "https://github.com/xxxxzb/gpui-kit-mcp", branch = "gpui-kit" }

[patch."https://github.com/zed-industries/zed"]
gpui = { git = "https://github.com/xxxxzb/gpui-kit-mcp", branch = "gpui-kit" }
```

两个 patch 都需要。它们让应用、`gpui_platform` 和桥接使用同一份 GPUI 构建。
等 [vendor 补丁清单](vendor/gpui/PATCHES.md) 里的改动进入上游后，这两个 patch 就可以去掉。

开窗时安装桥接：

```rust,ignore
use gpui_mcp::{AppId, BridgeConfig, BridgeHandle};

let bridge = BridgeHandle::install(
    window,
    cx,
    BridgeConfig::new(AppId::new("my-app")?, "My App"),
)?;
```

集成就这些。客户端会自己发现正在运行的应用。
完整窗口示例见 [demo](examples/demo/src/main.rs)。

### 多应用与多代理

每个 `gpui-mcp` 服务进程各自保存选中目标。要让多个代理驱动不同的应用实例，
给每个实例配一个独立的服务条目：

```json
{
  "mcpServers": {
    "gpui-a": { "command": "gpui-mcp", "args": ["--endpoint", "<descriptor-a>.json"] },
    "gpui-b": { "command": "gpui-mcp", "args": ["--endpoint", "<descriptor-b>.json"] }
  }
}
```

描述符文件位于运行根目录下的 `run/`：macOS 是
`~/Library/Application Support/gpui-mcp/run`，Linux 是
`$XDG_RUNTIME_DIR/gpui-mcp/run`，Windows 是 `%LOCALAPPDATA%\gpui-mcp\run`。
`BridgeHandle::endpoint_path()` 会返回运行中应用的描述符路径。应用退出时该文件
会被删除，重启后文件名会变，所以应用重启后需要更新这个条目。

只配一个服务条目、又有多个应用存活时，先调用 `list_apps`，再调用
`select_app`。未选中目标的 app-scoped 调用会失败并返回存活目标列表。

## 让界面适合 agent 使用

语义树由实际渲染的元素和无障碍数据构建。大多数问题来自下面几条。

1. **让 `BridgeHandle` 保持存活。** 把它存进根视图；drop 掉它，桥接就停止。
2. **给元素加 `.id(...)`。** 只有带 id 的元素才会成为节点。没有 id 的元素里的文字
   会并入最近祖先的 label，无法单独点击、等待或断言。可点击元素在 GPUI 里本来就需要 id。
3. **同级 id 保持唯一。** id 冲突时，节点会得到带路径的长身份；完全重复的节点会被丢弃，
   并产生 `DuplicateId` 诊断。找子节点请走 `parent`，不要依赖 id 的形状。
4. **给没有文字的控件命名。** 图标按钮的 label 否则就是它的字形，比如 `⚙`。
   用 `.aria_label("Settings")`；角色推断不出来时用 `.role(...)`，比如 tabs。
5. **显式声明禁用和只读。** `enabled` 只来自 `.aria_disabled(true)`。
   一个变灰但没有点击处理的控件仍然会报 `enabled: true`。只读输入框用
   `.aria_read_only(true)`。
6. **脱敏敏感字段。** 在带 id 的元素上加 `.frame_redacted(true)`，它的文字和值以及
   由它们派生出的 label 都不会进入桥接。
7. **检查诊断。** `get_ui_tree` 返回的 `diagnostics` 会报告被省略、重复和孤立的节点。

```rust,ignore
div()
    .id("settings")
    .aria_label("Settings")
    .on_click(cx.listener(|this, _, _, cx| this.open_settings(cx)))
    .child(svg().path("icons/settings.svg").size_4())
```

如果界面是 HTML 编写、并且希望 agent 实时编辑，见
[visual builder 指南](docs/visual-builder.md) 和
[showcase](examples/runtime-showcase)。HTML 渲染器支持标准 CSS 动画：
`transition`、`@keyframes`、`@starting-style` 和 view transitions（见
[Motion](docs/visual-builder.md#motion)）。

## Agent 能做什么

| 领域 | 工具 |
| --- | --- |
| 发现 | `list_apps`、`select_app`、`get_ui_tree`、`find_elements`、`get_element` |
| 操作 | `click_element`、`type_text`、`set_text`、`set_value`、`keyboard`、`hover_element`、`drag_element`、`scroll`、`pointer_*` |
| 验证 | `wait_for_element`、`wait_for_state`、`get_element_state`、`save_ui_snapshot`、`diff_current_ui` |
| 窗口 | `resize_window` 按逻辑像素设置内容区尺寸，并返回实际达到的尺寸 |
| 像素 | `screenshot`、`screenshot_element`、`compare_screenshots`、`start_video_recording` |
| 标注 | `annotate_elements`、`remove_annotations`、`list_annotations`、`highlight_elements`、`clear_highlights` |
| 与应用对话 | `read_messages`、`wait_for_messages`、`send_message`、`gpui://messages` 资源 |
| 性能 | `mark_frames`、`get_frame_report`、`record_performance` |
| 实时编辑 | `get_live_document`、`preview_live_document` |

优先用元素工具，而不是坐标。所有坐标都是相对于窗口的逻辑像素。

支持 MCP tasks 扩展（SEP-2663）的客户端，会把可能阻塞数秒的工具
（`wait_for_messages`、`wait_for_element`、`wait_for_state`、`record_performance`）
当作 task，用 `tasks/get` 轮询，长等待不会卡住客户端。其他客户端用普通调用拿到同样的结果。

## 标注

标注用一个轮廓或填充，加上可选标签，标出一个语义元素或固定矩形。
节点标注每帧重新解析，所以它会跟随元素经过布局变化、滚动和缩放；元素不存在、隐藏或
滚出容器时，标注会隐藏。每个标注有 id（复用它来更新）、可选 group、可选生命周期
（从第一次绘制它的帧开始计时）。Agent 用 `annotate_elements`；
`highlight_elements` 是它的薄封装，把元素画进 `highlights` 组。

应用和 agent 共享同一套标注。应用可以添加自己的标注、观察 agent 的标注，也可以自己绘制：

```rust,ignore
use gpui_mcp::{AnnotationSpec, AnnotationStyle};

let automation = bridge.automation();
automation.annotate(
    AnnotationSpec::node("save")
        .with_label("Unsaved changes")
        .with_style(AnnotationStyle::OutlineFill)
        .with_color("#FFB020FF")
        .with_ttl_ms(5_000),
    window,
)?;

// Agent、应用或过期都会触发；每次变化后调用。
bridge.on_annotations(|event, _window, cx| {
    // event.annotations: 完整集合；event.changed_by: Agent、App 或 Bridge
})?;

// 也可以自己绘制；bounds 仍然每帧解析。
automation.paint_annotations(false, window);
let drawn = automation.annotations(); // 每个都带 `resolved` bounds
```

## 从应用和 agent 对话

桥接里没有模型。应用内的聊天框通过一个有界消息日志，和用户自己的 MCP agent
（Claude Code、Codex 或任何其他客户端）对话：消息有单调递增的 id 和时间戳，
两边按 id 读取，每一方最多保留 64 条对方尚未读取的消息。

```rust,ignore
use gpui_mcp::NewMessage;

// 在 GPUI 线程上按顺序、每条一次地接收 agent 的消息。
// 注册这个回调也会告诉 agent：应用会读消息。
bridge.on_message(|message, _window, cx| {
    // 展示 message.text；message.reply_to 指向它回答的应用消息
})?;

// 发布用户输入的内容。
bridge.post_message(NewMessage::text("把工具栏做紧凑一点"))?;
```

Agent 用 `read_messages` 读取应用消息（下次把返回的 `latest_id` 作为 `since` 传入），
用 `send_message` 回复，并在 `reply_to` 里写明回答的是哪条消息。等待用户的下一条消息：

- 任何客户端都可以循环调用 `wait_for_messages`；每次最多阻塞 30 秒，没有消息时返回
  空页和 `timed_out: true`；
- 支持 MCP tasks 的客户端会把这次等待当作 task 执行；
- 支持资源订阅的客户端可以订阅 `gpui://messages`（用 `resources/subscribe` 或
  `subscriptions/listen`），应用每发一条消息就收到 `notifications/resources/updated`。

聊天循环可以这样提示 agent：“用你见过的最新 id 调 `wait_for_messages`，处理应用发来的
每条消息，用 `send_message` 回答，然后继续等待。”

## GPUI Kit、`gpui-pre`、`gpui-fast` 和 `gpui-ce`

除了 Zed 自己的 GPUI，桥接还支持 crates.io 上的三个 GPUI 发行版：

| GPUI crate | 使用方 | 支持版本 | 桥接 feature |
| --- | --- | --- | --- |
| `gpui-pre` | GPUI Kit、`gpui-component` | 0.3.5、0.3.6、0.3.7、0.3.9 | `gpui-pre` |
| `gpui-fast` | GPUI Kit 0.8 的保留模式渲染器 | 0.1.10 | `gpui-fast` |
| `gpui-ce` | 社区 fork 及其应用 | 0.2.2 | `gpui-ce` |

每个版本都需要一小组 GPUI 补丁，本仓库提供。下面的配方会从本仓库拉取打好补丁的副本。

### GPUI Kit 0.7.3（`gpui-pre` 0.3.9）

这个 fork 带有 0.3.9 补丁系列。把依赖指向本仓库（开发时也可以用本地路径）；
上游 `themixednuts/gpui-mcp` 只到 0.3.7。

在工作区的 `Cargo.toml` 里加：

```toml
[dependencies]
gpui-kit = "=0.7.3"
gpui-mcp = { git = "https://github.com/xxxxzb/gpui-kit-mcp", branch = "gpui-kit", default-features = false, features = ["gpui-pre"] }

[patch.crates-io]
gpui-pre = { git = "https://github.com/xxxxzb/gpui-kit-mcp", branch = "gpui-kit" }
```

调用 `gpui_kit::init(cx)`，用 `gpui_kit::open_window` 开窗，然后照常安装桥接。
桥接直接接受 Kit 的 `Window` 和 `App`。想看实际效果，运行
[Kit demo](examples/gpui-kit)：

```console
cargo run --manifest-path examples/gpui-kit/Cargo.toml
```

### GPUI Kit 0.8.0（`gpui-fast` 0.1.10）

GPUI Kit 0.8 可以选择通过 `gpui-fast`（GPUI 的保留模式 fork）渲染。桥接
同样支持这个后端，机制与上面一致：本仓库 vendor 了带 automation 补丁
系列的 `gpui-fast` 副本，桥接链接它而不是 `gpui-pre`。

Kit 的两个渲染后端是编译期二选一，桥接的 backend feature 必须与 Kit
实际渲染用的那个一致。在你的 workspace `Cargo.toml` 加：

```toml
[dependencies]
gpui-kit = { version = "=0.8.0", features = ["gpui-fast"] }
gpui-mcp = { git = "https://github.com/xxxxzb/gpui-kit-mcp", branch = "gpui-kit", default-features = false, features = ["gpui-fast"] }

[patch.crates-io]
gpui-fast = { git = "https://github.com/xxxxzb/gpui-kit-mcp", branch = "gpui-kit" }
```

Kit 的 `gpui-pre` 依赖仍在图中，但开着 `gpui-fast` feature 时不链接任何
东西；桥接直接接受 Kit 的 `Window` 和 `App`，与 0.7.3 配方完全一样。

有一处视图追踪的差异是真实行为：即时模式每帧重建未缓存的视图，
`gpui-fast` 把它保留下来，所以 fast 应用的 frame report 在一次交互里
上报的渲染次数更少。语义树、来源信息、输入、截图等其余观测能力一致。
想看实际效果，运行 [Kit fast demo](examples/gpui-kit-fast)：

```console
cargo run --manifest-path examples/gpui-kit-fast/Cargo.toml
```

### `gpui-ce` 0.2.2

在工作区的 `Cargo.toml` 里加：

```toml
[dependencies]
gpui-ce = "=0.2.2"
gpui_ce_platform = "=0.1.0"
gpui-mcp = { git = "https://github.com/xxxxzb/gpui-kit-mcp", branch = "gpui-kit", default-features = false, features = ["gpui-ce"] }

[patch.crates-io]
gpui-ce = { git = "https://github.com/xxxxzb/gpui-kit-mcp", branch = "gpui-kit" }
```

`gpui-ce` 以 `gpui` 的名字导入，所以开窗时照常安装桥接。

### 其他版本，或你自己的 GPUI 补丁

上面的 `[patch]` 配方总是给你本仓库 vendored 的版本。如果你的应用在更早的支持版本上，
或者你自己带 GPUI 补丁，就保留自己的补丁副本：

1. 在本仓库的 checkout 里，把打好补丁的副本写进你的应用。
   `gpui-ce` 用 `--crate gpui-ce`，`gpui-fast` 用 `--crate gpui-fast`。

   ```console
   cargo xtask vendor --crate gpui-pre --version 0.3.5 --output /path/to/your-app/vendor/gpui-pre
   ```

2. 如果有自己的补丁，应用到这份副本上。
3. 让工作区指向它：

   ```toml
   [patch.crates-io]
   gpui-pre = { path = "vendor/gpui-pre" }
   ```

补丁在 [`vendor/patches/`](vendor/patches)，每个 crate、每个版本一个目录。
`gpui-pre` 与 `gpui-ce` 系列每个版本有三个补丁，`gpui-fast` 系列只带
`automation.patch`：

- `automation.patch`：桥接需要的全部改动。
- `font-fallback.patch`：一个无关的字体修复。加 `--without font-fallback` 可以跳过。
- `grid.patch`：暴露 CSS grid 轨道列表（`grid_template_columns`、`grid_template_rows`、
  `grid_auto_rows`、`grid_auto_flow`、`grid_column`、`grid_row`，支持 `px`、`%`、`fr`、
  `auto`、`min-content`、`max-content`、`minmax()`、`fit-content()` 和
  `repeat(n | auto-fill | auto-fit, …)`）。桥接不需要它，但 `gpui-mcp-html` 需要。
  加 `--without grid` 可以跳过。

CI 会针对每个支持版本构建并测试桥接，所以这些补丁会保持可用。

### 注意事项

- 每个应用只用一个 backend。GPUI Kit 或 `gpui-ce` 应用里的每个 `gpui-mcp` 依赖都需要
  `default-features = false`。关闭默认 feature 的 Zed 应用要加 `features = ["zed"]`。
- 发布无障碍信息的 Kit 组件可以用全部工具。自绘组件只会显示它标注的内容。
- GPUI Kit 0.7.x 可能把禁用控件报成 `enabled: true`，并漏掉 `read_only`；组件不发布时，
  应用要用 `.aria_disabled(true)` 和 `.aria_read_only(true)` 补上。
- `gpui-mcp-html` 也支持 GPUI Kit：用
  `default-features = false, features = ["gpui-pre", "json", "ron"]` 添加。它的 CSS
  过渡和动画跑在 GPUI Kit 的 `gpui_base::motion` 上，解析到与你 gpui-pre 匹配的 Kit
  版本（0.3.5 和 0.3.6 用 0.6.4+，0.3.9 用 0.7.3）。它的 `grid-template-*` 支持需要
  下面说的 `grid.patch`，`[patch]` 配方已经包含。

## 测量帧耗时

注入输入和真实输入的开销相同。服务端会等这次输入产生的帧，自己不会额外加帧。

调用 `mark_frames`，执行交互，再调用 `get_frame_report`。它会给出标记之后每一帧的：

- 完整的 `Window::draw` 耗时（`draw_ms`），拆成应用占比（`app_draw_ms`）和桥接占比
  （`bridge_ms`）；
- 每项的 p50、p95 和最大值；
- 每个渲染的视图及原因，以及回放缓存而不是重绘的视图。

渲染原因有：

- `notified`：视图或它内部的视图调用了 `cx.notify()`
- `ancestor_rendered`：它周围的缓存视图渲染了
- `refresh`：窗口被刷新
- `first_draw`：视图还没有缓存
- `layout_changed`：bounds、content mask 或文字样式变了
- `uncached`：它没有用 `.cached(...)` 嵌入

在 `Entity::cached` 区域里悬停，应该看到该区域 `notified`，兄弟视图回放。其他情况说明
缓存边界有泄漏。`get_frame_stats` 对同样的帧取平均；`record_performance` 报告固定区间。
进程内可以用 `Automation::mark_frames` 和 `Automation::frame_report`。

`bridge_ms` 是桥接给一次绘制增加的工作：完成无障碍树、构建 observed frame、绘制标注。
客户端读取语义树时，转换在 UI 线程之外完成。

## 平台说明

- **Linux：** 精确窗口截图目前需要 X11。
- **Windows：** 截图会取一小段合成器样本并返回最新的那个，因为 Windows Graphics Capture
  可能先返回旧帧。一秒的截止时间覆盖等待合成器，不包括 readback，所以即使很大的窗口也能
  从 debug 构建里截到。

## 安全

只在开发、测试或其他可信环境里启用自动化。见 [SECURITY.md](SECURITY.md)。

## 许可证

Apache-2.0。这个 fork 基于
[`themixednuts/gpui-mcp`](https://github.com/themixednuts/gpui-mcp)；归属和修改列表见
[NOTICE](NOTICE)。
