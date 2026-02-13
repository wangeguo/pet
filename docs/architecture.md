# Pet - 桌面宠物系统技术设计文档

## 1. 技术选型

| 领域 | 技术方案 | 选择理由 |
|------|---------|---------|
| 宠物剧场 | Bevy | 成熟的 Rust 游戏引擎，原生支持 GLB/glTF，内置 ECS 和动画系统 |
| 设置中心 | Iced | Rust 原生、声明式 UI，适合表单类配置界面 |
| 宠物管理器 | 待定 | 未来涉及 3D 预览，技术方案待定 |
| 系统托盘 | tray-icon | Rust 原生系统托盘支持 |
| AI 中枢 | rig.rs | Rust 原生 LLM 框架，支持多 provider 和 Tool 调用 |
| 宠物生成 | Meshy AI | 支持文字生成 3D 模型 |
| 实时通信 | Unix Domain Socket | 本机进程间低延迟双向通信 |
| 配置存储 | config crate | 支持多种格式（TOML/YAML/JSON 等） |
| 模型格式 | GLB | 通用 3D 模型格式，Bevy 原生支持 |

## 2. Replay 架构

宠物剧场采用类似游戏录像回放的 Replay 架构。宠物的行为
不是实时计算的，而是预先定义好的行为脚本，剧场进程负责
"回放"这些行为。

### 2.1 核心概念

```
┌──────────────────────────────────────────────────┐
│                   Replay 架构                     │
├──────────────────────────────────────────────────┤
│                                                  │
│  ┌──────────┐    ┌──────────┐    ┌──────────┐   │
│  │ 行为脚本 │    │ Replayer │    │ 渲染输出 │   │
│  │ (Script) │ ─► │ (剧场)   │ ─► │ (Screen) │   │
│  └──────────┘    └──────────┘    └──────────┘   │
│       │               │                         │
│       │               ▼                         │
│       │          ┌──────────┐                   │
│       │          │ 交互事件 │                   │
│       │          │ (Input)  │                   │
│       │          └──────────┘                   │
│       │               │                         │
│       │               ▼                         │
│       │          ┌──────────┐                   │
│       └────────► │ 触发新   │                   │
│                  │ 脚本     │                   │
│                  └──────────┘                   │
│                                                  │
└──────────────────────────────────────────────────┘
```

### 2.2 类比游戏录像

| 游戏录像 | 宠物剧场 |
|---------|---------|
| 录像文件 (.w3g/.dem) | 行为脚本 (BehaviorScript) |
| 游戏引擎 | Bevy 渲染引擎 |
| 回放器 | theater 进程 |
| 玩家操作记录 | 状态转换 + 动画序列 |
| 观看录像 | 宠物在桌面"表演" |

### 2.3 回放流程

```
1. 加载宠物模型和行为脚本库
2. 进入默认脚本 (idle)
3. 按时间轴执行关键帧动作
4. 脚本结束时，根据 next 字段切换脚本
5. 收到交互事件时，切换到对应脚本
6. 循环执行
```

### 2.4 优势

- **确定性**：相同脚本产生相同行为，便于调试
- **可扩展**：添加新行为只需编写新脚本
- **低耦合**：行为逻辑与渲染引擎分离
- **可存档**：脚本可序列化存储和分享

### 2.5 AI 驱动的脚本生成

在 Replay 架构的基础上，引入 AI 中枢（Brain）实现动态
脚本生成。剧场保持纯粹的回放器角色，脚本来源从"内置"
扩展为"内置 + AI 动态生成"。

```
┌─────────────────────────────────────────────┐
│           脚本来源                           │
├─────────────────────────────────────────────┤
│                                             │
│  ┌────────────┐        ┌────────────┐      │
│  │ 内置脚本库 │        │  AI 中枢   │      │
│  │ (builtin)  │        │  (Brain)   │      │
│  └─────┬──────┘        └─────┬──────┘      │
│        │                     │              │
│        │  降级回退    LLM 动态生成          │
│        │                     │              │
│        └────────┬────────────┘              │
│                 ▼                            │
│         ┌──────────────┐                    │
│         │ BehaviorScript│                   │
│         └──────┬───────┘                    │
│                ▼                             │
│         ┌──────────────┐                    │
│         │   Theater    │                    │
│         │  (Replayer)  │                    │
│         └──────────────┘                    │
│                                             │
└─────────────────────────────────────────────┘
```

**工作流：**

1. Theater 加载 GLB 模型，自动检测所有可用动画
2. Theater 通过 IPC 将可用动画列表通告 Brain
3. 用户交互事件（点击等）由 Theater 上报给 Brain
4. Brain 构建 prompt（系统人格 + 可用动画 + 上下文）
5. Brain 调用 LLM 生成 BehaviorScript（JSON 格式）
6. Brain 验证脚本合法性后推送给 Theater
7. Theater 接收并回放脚本

**降级策略**：当 LLM 不可用（无 API key、网络故障、
超时）时，Brain 回退到内置脚本库，保证宠物始终可用。

## 3. 进程架构

由于 Bevy 和 Iced 各自拥有独立的事件循环，采用多进程
架构：

```
┌───────────────────────────────────────────────────────┐
│                       主进程                           │
│  ┌─────────────────────────────────────────────────┐ │
│  │          进程管理器 + 消息路由器                 │ │
│  │  - 启动/停止/监控子进程                         │ │
│  │  - UDS 服务端，中心化消息路由                   │ │
│  │  - 配置文件变更监听                             │ │
│  └─────────────────────────────────────────────────┘ │
│      │        │        │        │         │          │
│  ┌───┴──┐ ┌──┴───┐ ┌──┴──┐ ┌──┴────┐ ┌──┴──────┐  │
│  │ tray │ │brain │ │thea-│ │mana-  │ │settings │  │
│  │(托盘)│ │(AI)  │ │ter  │ │ger    │ │(设置)   │  │
│  │ 常驻 │ │ 常驻 │ │(剧场)│ │(管理器)│ │  Iced   │  │
│  └──────┘ └──────┘ └─────┘ └───────┘ └─────────┘  │
│      │        │        │        │         │          │
│      └────────┴────────┴────────┴─────────┘          │
│                        │                              │
│              ┌─────────┴─────────┐                    │
│              │   config.toml     │                    │
│              │   (持久配置)      │                    │
│              └───────────────────┘                    │
└───────────────────────────────────────────────────────┘
```

### 3.1 进程职责

| 进程 | 技术栈 | 职责 | 生命周期 |
|------|-------|------|---------|
| 主进程 | tokio | 进程管理、消息路由 | 常驻 |
| 托盘进程 | tray-icon | 系统托盘、菜单交互 | 常驻 |
| AI 中枢 | rig.rs + tokio | LLM 交互、脚本生成 | 常驻 |
| 剧场进程 | Bevy | 脚本回放、3D 渲染、交互 | 宠物显示时运行 |
| 管理器进程 | 待定 | 宠物创建/管理/预览 | 按需启动 |
| 设置进程 | Iced | 配置管理 | 按需启动 |

### 3.2 进程间通信

采用双层 IPC 策略：

| 层 | 机制 | 用途 | 延迟 |
|---|------|------|------|
| 实时消息层 | Unix Domain Socket | 交互事件、脚本推送、状态同步 | < 1ms |
| 持久配置层 | config.toml + notify | 配置变更 | 50-500ms |

**实时消息层（UDS）**

星型拓扑，主进程作为中心路由：

```
tray ──UDS──> app <──UDS── theater
               |
             brain
             /   \
        manager  settings
```

每个子进程启动后连接到主进程的 UDS 服务端，注册自己的
`ProcessId`。主进程负责消息路由：将 theater 发来的
`PetClicked` 转发给 brain，将 brain 发来的
`ExecuteScript` 转发给 theater。

UDS 路径：`{config_dir}/pet.sock`

**消息协议**（定义在 `common` crate）：

```rust
/// 进程间消息信封
#[derive(Serialize, Deserialize)]
pub struct IpcEnvelope {
    pub source: ProcessId,
    pub target: ProcessId,
    pub payload: IpcMessage,
    pub timestamp: i64,
}

#[derive(Serialize, Deserialize)]
pub enum ProcessId {
    App, Tray, Theater, Brain, Manager, Settings,
}

#[derive(Serialize, Deserialize)]
pub enum IpcMessage {
    // Theater -> Brain
    PetClicked,
    PetDragCompleted { x: i32, y: i32 },
    AnimationsAvailable { animations: Vec<String> },

    // Brain -> Theater
    ExecuteScript { script: BehaviorScript },
    AiThinking { is_thinking: bool },

    // Tray/Manager -> Brain
    UserTextInput { text: String },

    // 通用
    ProcessReady,
    Shutdown,
    Ping,
    Pong,
}
```

**持久配置层**

保留现有的 `config.toml` + `notify` 机制，用于低频
配置变更（宠物列表、API Key、设置项等）。

### 3.3 启动流程

```
app (主进程)
|
+-- 1. 读取配置
+-- 2. 启动 UDS 服务端
+-- 3. 启动常驻进程
|      ├── spawn: tray
|      └── spawn: brain
+-- 4. 如有活跃宠物，启动剧场
|      └── spawn: theater --pet-id=<uuid>
+-- 5. 首次运行时，启动管理器
|      └── spawn: manager
+-- 6. 进入事件循环
       - 监控子进程状态
       - 路由 IPC 消息
       - 监听配置文件变更
```

### 3.4 进程通信示例

```
用户点击宠物（AI 驱动模式）：
  theater -> UDS -> app -> UDS -> brain
  brain 调用 LLM，生成脚本
  brain -> UDS -> app -> UDS -> theater
  theater 回放脚本

用户在管理器切换宠物：
  manager -> 写入 config.toml ->
  app 检测变化 -> 重启 theater（brain 常驻不重启）
```

## 4. 项目结构

采用 Cargo workspace 组织 7 个 crate（app, common, tray,
theater, brain, manager, settings）：

```
pet/
├── Cargo.toml
├── docs/
│   ├── product.md
│   ├── architecture.md
│   └── roadmap.md
├── crates/
│   ├── app/                     # 主进程
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── main.rs          # 入口
│   │       ├── process.rs       # 子进程管理
│   │       └── router.rs        # IPC 消息路由
│   │
│   ├── tray/                    # 托盘进程
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── main.rs
│   │       └── menu.rs
│   │
│   ├── theater/                 # 剧场进程 (Bevy)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── main.rs
│   │       ├── app.rs
│   │       ├── plugins/
│   │       │   ├── mod.rs
│   │       │   ├── pet.rs       # 模型加载、渲染
│   │       │   ├── replay.rs    # 脚本回放
│   │       │   ├── interaction.rs # 拖拽、点击
│   │       │   └── ipc.rs       # IPC 收发
│   │       ├── components.rs
│   │       ├── resources.rs
│   │       ├── events.rs
│   │       └── systems/
│   │           ├── mod.rs
│   │           ├── keyframe.rs
│   │           ├── animation.rs
│   │           ├── movement.rs
│   │           ├── scale.rs
│   │           ├── bounce.rs
│   │           ├── rotation.rs
│   │           └── script_transition.rs
│   │
│   ├── brain/                   # AI 中枢进程
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── main.rs          # 入口、UDS 连接
│   │       ├── agent.rs         # rig.rs Agent 配置
│   │       ├── script_gen.rs    # LLM -> BehaviorScript
│   │       ├── personality.rs   # 宠物人格管理
│   │       ├── context.rs       # 上下文管理
│   │       ├── handlers.rs      # IPC 消息处理
│   │       └── tools/           # rig.rs Tool 实现
│   │           ├── mod.rs
│   │           ├── animation.rs # 查询可用动画
│   │           └── script.rs    # 生成脚本
│   │
│   ├── manager/                 # 管理器进程（技术方案待定）
│   │   ├── Cargo.toml
│   │   └── src/
│   │       └── main.rs
│   │
│   ├── settings/                # 设置进程 (Iced)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── main.rs
│   │       ├── app.rs
│   │       └── views/
│   │           ├── mod.rs
│   │           ├── general.rs   # General 分区
│   │           ├── appearance.rs # Appearance 分区
│   │           ├── ai.rs        # AI 分区
│   │           ├── meshy.rs     # Meshy AI 分区
│   │           └── about.rs     # About 分区
│   │
│   └── common/                  # 共享库
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs
│           ├── models.rs        # 数据模型
│           ├── config.rs        # 配置读写
│           ├── script.rs        # 行为脚本定义
│           ├── ipc.rs           # IPC 消息类型
│           ├── paths.rs         # 路径管理
│           ├── autostart.rs     # 开机自启
│           ├── error.rs         # 错误类型
│           └── storage.rs       # 存储服务
│
└── assets/
    ├── scripts/                 # 内置行为脚本
    │   ├── idle.ron
    │   ├── walk.ron
    │   ├── happy.ron
    │   ├── sleep.ron
    │   ├── bounce.ron
    │   └── spin.ron
    └── pets/                    # 宠物模型
```

## 5. 数据模型

### 5.1 宠物数据

```rust
#[derive(Serialize, Deserialize)]
struct Pet {
    id: Uuid,
    name: String,
    description: String,
    model_path: PathBuf,
    created_at: DateTime<Utc>,
}
```

### 5.2 配置数据

配置采用分组结构，便于 Settings 界面按分区管理：

```rust
#[derive(Serialize, Deserialize)]
struct AppConfig {
    #[serde(default)]
    general: GeneralSettings,
    #[serde(default)]
    appearance: AppearanceSettings,
    #[serde(default)]
    ai: AiSettings,
    #[serde(default)]
    meshy: MeshySettings,
    pets: Vec<Pet>,
    active_pet: Option<Uuid>,
}

#[derive(Serialize, Deserialize)]
struct GeneralSettings {
    auto_start: bool,
    language: String,           // "en", "zh"
}

#[derive(Serialize, Deserialize)]
struct AppearanceSettings {
    pet_scale: f32,
    pet_position: WindowPosition,
    always_on_top: bool,
    opacity: f32,               // 0.0 - 1.0
}

#[derive(Serialize, Deserialize)]
struct AiSettings {
    enabled: bool,
    provider: AiProvider,
    api_key: Option<String>,
    model: String,
    endpoint: Option<String>,   // 自定义端点
    personality: PersonalityConfig,
    temperature: f32,
}

#[derive(Serialize, Deserialize)]
enum AiProvider {
    OpenAi,
    Anthropic,
    Ollama,
    Custom,
}

#[derive(Serialize, Deserialize)]
struct PersonalityConfig {
    name: String,               // 宠物名字
    traits: Vec<String>,        // ["cheerful", "curious"]
    custom_prompt: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct MeshySettings {
    api_key: Option<String>,
}
```

### 5.3 行为脚本

```rust
/// 行为脚本 - 定义宠物的一段完整行为
#[derive(Serialize, Deserialize)]
struct BehaviorScript {
    id: String,
    duration: Option<Duration>,
    keyframes: Vec<Keyframe>,
    next: Option<String>,
    interruptible: bool,
}

/// 关键帧 - 某一时刻的状态
#[derive(Serialize, Deserialize)]
struct Keyframe {
    time: f32,
    action: Action,
}

/// 动作类型
#[derive(Serialize, Deserialize)]
enum Action {
    PlayAnimation(String),
    MoveTo(Vec2),
    Scale(f32),
    Spin(f32),
    Bounce(f32),
    Wait(f32),
    Random(Vec<String>),
    SetExpression(String),
}
```

### 5.4 Bevy 组件

```rust
#[derive(Component)]
struct PetMarker;

#[derive(Component)]
struct ReplayState {
    script_id: String,
    current_time: f32,
    keyframe_index: usize,
}

#[derive(Component)]
struct Draggable {
    dragging: bool,
    offset: Vec2,
}
```

### 5.5 存储位置

| 数据类型 | 存储位置 |
|---------|---------|
| 配置文件 | `~/.config/pet/config.toml` |
| 运行状态 | `~/.config/pet/state.toml` |
| UDS 套接字 | `~/.config/pet/pet.sock` |
| 宠物模型 | `~/.local/share/pet/models/` |
| 行为脚本 | `~/.local/share/pet/scripts/` |
| 日志文件 | `~/.local/share/pet/logs/` |

## 6. Brain 架构设计

### 6.1 核心职责

Brain 作为 AI 中枢，负责将用户交互转化为宠物行为：

```
交互事件 → prompt 构建 → LLM 调用 →
脚本解析验证 → 推送 theater
```

### 6.2 rig.rs 集成

使用 rig.rs 构建 LLM Agent，支持多 provider：

```rust
pub struct PetBrain {
    agent: Agent<impl CompletionModel>,
    context: PetContext,
}

impl PetBrain {
    pub async fn handle_interaction(
        &mut self,
        event: InteractionEvent,
    ) -> Result<BehaviorScript> {
        let prompt = self.context.build_prompt(event);
        let response = self.agent.prompt(&prompt).await?;
        let script = parse_script_response(&response)?;
        validate_script(
            &script,
            &self.context.available_animations,
        )?;
        Ok(script)
    }
}
```

### 6.3 上下文管理

`PetContext` 维护 Brain 决策所需的全部上下文：

- 宠物模型支持的动画列表（从 Theater 获取）
- 可用的 Action 类型列表
- 最近交互历史
- 宠物当前状态（正在执行的脚本等）
- 宠物性格配置

### 6.4 LLM 延迟处理

LLM 调用通常需要 1-3 秒，等待期间：

1. Brain 立即发送 `AiThinking { is_thinking: true }`
2. Theater 播放"思考中"过渡动画
3. LLM 响应到达后发送脚本，无缝切换

### 6.5 降级策略

AI 不可用（无 API key / 网络故障 / 超时）时：

1. Brain 回退到内置脚本映射（如点击 → happy 脚本）
2. 行为与 AI 关闭时完全一致，用户无感知
3. 恢复后自动切换回 AI 模式

## 7. Meshy AI 集成

### 7.1 工作流程

```
┌─────────┐  ┌─────────┐  ┌──────────┐  ┌──────────┐
│ 提交任务│ ►│ 获取 ID │ ►│ 轮询状态 │ ►│ 下载模型 │
└─────────┘  └─────────┘  └──────────┘  └──────────┘
     │            │             │             │
     ▼            ▼             ▼             ▼
  POST /v2/   返回        GET /v2/      GET model_url
  text-to-3d  task_id     text-to-3d/    下载 GLB
                          {task_id}
```

### 7.2 API 调用

**创建任务：**
```
POST https://api.meshy.ai/v2/text-to-3d
{
    "mode": "preview",
    "prompt": "用户输入的描述",
    "art_style": "realistic"
}
```

**查询状态：**
```
GET https://api.meshy.ai/v2/text-to-3d/{task_id}
```

**状态值：**
- `PENDING` - 等待处理
- `IN_PROGRESS` - 处理中
- `SUCCEEDED` - 完成
- `FAILED` - 失败

## 8. 依赖清单

```toml
# Cargo.toml (workspace)
[workspace]
resolver = "2"
members = ["crates/*"]

# crates/common
[dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"
config = "0.14"
uuid = { version = "1", features = ["v4", "serde"] }
thiserror = "2"

# crates/app
[dependencies]
common = { path = "../common" }
tokio = { version = "1", features = ["full", "process"] }
notify = "7"

# crates/tray
[dependencies]
common = { path = "../common" }
tray-icon = "0.21"

# crates/theater
[dependencies]
common = { path = "../common" }
bevy = "0.18"

# crates/brain
[dependencies]
common = { path = "../common" }
rig-core = "0.11"
tokio = { version = "1", features = ["full"] }
serde.workspace = true
serde_json.workspace = true
tracing.workspace = true

# crates/settings
[dependencies]
common = { path = "../common" }
iced = { version = "0.14", features = ["tokio"] }

# crates/manager (技术方案待定)
[dependencies]
common = { path = "../common" }
reqwest = { version = "0.12", features = ["json"] }
```

## 9. 行为脚本示例

```ron
// assets/scripts/idle.ron
BehaviorScript(
    id: "idle",
    duration: None,
    keyframes: [
        Keyframe(
            time: 0.0,
            action: PlayAnimation("breathe"),
        ),
        Keyframe(
            time: 3.0,
            action: Random(["idle", "idle", "walk"]),
        ),
    ],
    next: None,
    interruptible: true,
)

// assets/scripts/happy.ron
BehaviorScript(
    id: "happy",
    duration: Some(2.0),
    keyframes: [
        Keyframe(
            time: 0.0,
            action: PlayAnimation("jump"),
        ),
        Keyframe(time: 0.5, action: Scale(1.2)),
        Keyframe(time: 1.0, action: Scale(1.0)),
        Keyframe(
            time: 1.5,
            action: PlayAnimation("spin"),
        ),
    ],
    next: Some("idle"),
    interruptible: false,
)
```

## 10. 参考资料

- [Bevy 引擎文档](https://bevyengine.org/learn/)
- [Bevy Cheat Book](https://bevy-cheatbook.github.io/)
- [Iced 框架文档](https://docs.rs/iced)
- [rig.rs 文档](https://docs.rig.rs/)
- [Meshy AI API 文档](https://docs.meshy.ai)
- [tray-icon 文档](https://docs.rs/tray-icon)
