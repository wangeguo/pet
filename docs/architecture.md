# Pet - 桌面宠物系统技术设计文档

## 1. 技术选型

| 领域 | 技术方案 | 选择理由 |
|------|---------|---------|
| 宠物剧场 | Bevy | 成熟的 Rust 游戏引擎，原生支持 GLB/glTF，内置 ECS 和动画系统 |
| 管理界面 | Iced | Rust 原生、声明式 UI，适合构建管理工具界面 |
| 系统托盘 | tray-icon | Rust 原生系统托盘支持 |
| AI 集成 | Meshy AI | 支持文字生成 3D 模型 |
| 配置存储 | config crate | 支持多种格式（TOML/YAML/JSON 等） |
| 模型格式 | GLB | 通用 3D 模型格式，Bevy 原生支持 |

## 2. Replay 架构

宠物剧场采用类似游戏录像回放的 Replay 架构。宠物的行为不是实时计算的，而是预先定义好的行为脚本，剧场进程负责"回放"这些行为。

### 2.1 核心概念

```
┌─────────────────────────────────────────────────────────────────┐
│                        Replay 架构                              │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌─────────────┐      ┌─────────────┐      ┌─────────────┐     │
│  │ 行为脚本    │      │  Replayer   │      │  渲染输出   │     │
│  │ (Script)   │ ───► │  (剧场进程)  │ ───► │  (Screen)  │     │
│  └─────────────┘      └─────────────┘      └─────────────┘     │
│        │                    │                                   │
│        │                    ▼                                   │
│        │              ┌─────────────┐                          │
│        │              │ 交互事件    │                          │
│        │              │ (Input)    │                           │
│        │              └─────────────┘                          │
│        │                    │                                   │
│        │                    ▼                                   │
│        │              ┌─────────────┐                          │
│        └─────────────►│ 触发新脚本  │                          │
│                       └─────────────┘                          │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
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
5. 收到交互事件时，切换到对应脚本 (如点击 → happy)
6. 循环执行
```

### 2.4 优势

- **确定性**：相同脚本产生相同行为，便于调试
- **可扩展**：添加新行为只需编写新脚本
- **低耦合**：行为逻辑与渲染引擎分离
- **可存档**：脚本可序列化存储和分享

## 3. 进程架构

由于 Bevy 和 Iced 各自拥有独立的事件循环，采用多进程架构：

```
┌─────────────────────────────────────────────────────────────────┐
│                           主进程                                 │
│  ┌───────────────────────────────────────────────────────────┐ │
│  │                      进程管理器                            │ │
│  │  • 启动/停止子进程                                        │ │
│  │  • 监控子进程状态                                         │ │
│  │  • 进程间消息转发                                         │ │
│  │  • 应用生命周期管理                                        │ │
│  └───────────────────────────────────────────────────────────┘ │
│                              │                                  │
│         ┌────────────────────┼────────────────────┐            │
│         ▼                    ▼                    ▼             │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────┐      │
│  │ 托盘进程      │    │ 剧场进程     │    │ 管理器进程    │      │
│  │ (tray-icon)  │    │ (Bevy)      │    │ (Iced)       │      │
│  │              │    │              │    │              │      │
│  │ • 系统托盘   │    │ • 脚本回放   │    │ • 创建宠物   │      │
│  │ • 菜单交互   │    │ • 3D 渲染    │    │ • 列表管理   │      │
│  │ • 快捷操作   │    │ • 动画播放   │    │ • 设置配置   │      │
│  │              │    │ • 交互响应   │    │ • API 调用   │      │
│  └──────────────┘    └──────────────┘    └──────────────┘      │
│         │                    │                    │             │
│         └────────────────────┴────────────────────┘             │
│                              │                                  │
│                         配置文件                                 │
│                      (进程间状态同步)                            │
└─────────────────────────────────────────────────────────────────┘
```

### 3.1 进程职责

| 进程 | 技术栈 | 职责 | 生命周期 |
|------|-------|------|---------|
| 主进程 | tokio | 进程管理、消息转发 | 常驻 |
| 托盘进程 | tray-icon | 系统托盘、菜单交互 | 常驻 |
| 剧场进程 | Bevy | 脚本回放、3D 渲染、交互 | 宠物显示时运行 |
| 管理器进程 | Iced | UI 界面、API 调用 | 按需启动 |

### 3.2 进程间通信

通过配置文件进行进程间状态同步：

```
~/.config/pet/
├── config.toml         # 应用配置（宠物列表、API Key 等）
└── state.toml          # 运行状态（当前宠物、窗口位置等）
```

- 管理器修改配置后写入文件
- 剧场进程监听配置文件变化并重新加载
- 主进程根据配置启动/停止剧场进程

### 3.3 启动流程

```
app (主进程)
│
├── 1. 读取配置
├── 2. 启动托盘进程
│      └── spawn: tray
├── 3. 如有活跃宠物，启动剧场进程
│      └── spawn: theater --pet-id=<uuid>
├── 4. 首次运行时，启动管理器
│      └── spawn: manager
└── 5. 进入事件循环
       • 监控子进程状态
       • 接收子进程消息并转发
       • 处理子进程退出/重启
```

### 3.4 进程通信示例

```
用户点击托盘菜单「打开管理器」：
  托盘进程 → 主进程 → 启动管理器进程

用户在管理器切换宠物：
  管理器进程 → 写入配置 → 主进程检测变化 → 重启剧场进程
```

## 4. 项目结构

采用 Cargo workspace 组织多个 crate，crate 名称不加项目前缀以便后期更名：

```
pet/
├── Cargo.toml                   # workspace 配置
├── docs/
│   ├── product.md
│   ├── architecture.md
│   └── roadmap.md
├── crates/
│   ├── app/                     # 主进程
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── main.rs          # 入口
│   │       └── process.rs       # 子进程管理
│   │
│   ├── tray/                    # 托盘进程
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── main.rs          # 入口
│   │       └── menu.rs          # 托盘菜单
│   │
│   ├── theater/                 # 剧场进程 (Bevy) - Replayer
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── main.rs          # 入口
│   │       ├── app.rs           # Bevy App 配置
│   │       ├── plugins/
│   │       │   ├── mod.rs
│   │       │   ├── pet.rs       # 宠物插件 (模型加载、渲染)
│   │       │   ├── replay.rs    # 回放插件 (脚本执行)
│   │       │   ├── animation.rs # 动画插件
│   │       │   └── interaction.rs # 交互插件 (拖拽、点击)
│   │       ├── script.rs        # 行为脚本定义
│   │       ├── components.rs    # ECS 组件定义
│   │       ├── resources.rs     # ECS 资源定义
│   │       └── systems.rs       # ECS 系统
│   │
│   ├── manager/                 # 管理器进程 (Iced)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── main.rs          # 入口
│   │       ├── app.rs           # Iced Application
│   │       ├── views/
│   │       │   ├── mod.rs
│   │       │   ├── create.rs    # 创建宠物视图
│   │       │   ├── list.rs      # 宠物列表视图
│   │       │   └── settings.rs  # 设置视图
│   │       └── messages.rs      # UI 消息定义
│   │
│   └── common/                  # 共享库
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs
│           ├── models.rs        # 数据模型定义
│           ├── config.rs        # 配置读写
│           ├── script.rs        # 行为脚本定义 (共享)
│           ├── ipc.rs           # 进程间通信
│           └── services/
│               ├── mod.rs
│               ├── meshy.rs     # Meshy AI 集成
│               └── storage.rs   # 存储服务
│
└── assets/
    ├── icon.png                 # 托盘图标
    ├── scripts/                 # 内置行为脚本
    │   ├── idle.ron
    │   ├── walk.ron
    │   ├── happy.ron
    │   └── sleep.ron
    └── pets/                    # 宠物模型存储目录
```

## 5. 数据模型

### 5.1 宠物数据

```rust
#[derive(Serialize, Deserialize)]
struct Pet {
    id: Uuid,            // 唯一标识
    name: String,        // 宠物名称
    description: String, // 生成时的描述文字
    model_path: PathBuf, // 模型文件路径
    created_at: DateTime<Utc>, // 创建时间
}
```

### 5.2 配置数据

```rust
#[derive(Serialize, Deserialize)]
struct AppConfig {
    pets: Vec<Pet>,                // 宠物列表
    active_pet: Option<Uuid>,      // 当前显示的宠物
    pet_position: (i32, i32),      // 宠物窗口位置
    pet_scale: f32,                // 宠物缩放比例
    auto_start: bool,              // 开机自启
    meshy_api_key: Option<String>, // API 密钥
}
```

### 5.3 行为脚本

```rust
/// 行为脚本 - 定义宠物的一段完整行为
#[derive(Serialize, Deserialize)]
struct BehaviorScript {
    id: String,                    // 脚本标识 (如 "idle", "walk", "happy")
    duration: Option<Duration>,    // 持续时间 (None 表示循环)
    keyframes: Vec<Keyframe>,      // 关键帧序列
    next: Option<String>,          // 结束后转换到的下一个脚本
    interruptible: bool,           // 是否可被交互打断
}

/// 关键帧 - 某一时刻的状态
#[derive(Serialize, Deserialize)]
struct Keyframe {
    time: f32,                     // 时间点 (秒)
    action: Action,                // 动作
}

/// 动作类型
#[derive(Serialize, Deserialize)]
enum Action {
    PlayAnimation(String),         // 播放模型动画
    MoveTo(Vec2),                  // 移动到位置
    Scale(f32),                    // 缩放
    SetExpression(String),         // 设置表情
    Wait(f32),                     // 等待
    Random(Vec<String>),           // 随机选择下一个脚本
}
```

### 5.4 Bevy 组件

```rust
// 宠物标记组件
#[derive(Component)]
struct PetMarker;

// 当前回放状态
#[derive(Component)]
struct ReplayState {
    script_id: String,
    current_time: f32,
    keyframe_index: usize,
}

// 交互状态组件
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
| 宠物模型 | `~/.local/share/pet/models/` |
| 行为脚本 | `~/.local/share/pet/scripts/` |
| 日志文件 | `~/.local/share/pet/logs/` |

## 6. Meshy AI 集成

### 6.1 工作流程

```
┌─────────┐    ┌─────────┐    ┌─────────┐    ┌─────────┐
│ 提交任务 │ ─► │ 获取ID  │ ─► │ 轮询状态 │ ─► │ 下载模型 │
└─────────┘    └─────────┘    └─────────┘    └─────────┘
     │              │              │              │
     ▼              ▼              ▼              ▼
  POST /v2/    返回 task_id   GET /v2/     GET model_url
  text-to-3d                  text-to-3d/   下载 GLB 文件
                              {task_id}
```

### 6.2 API 调用

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

## 7. 依赖清单

```toml
# Cargo.toml (workspace)
[workspace]
resolver = "2"
members = ["crates/*"]

# crates/common/Cargo.toml (共享库)
[dependencies]
serde = { version = "1", features = ["derive"] }
config = "0.14"
uuid = { version = "1", features = ["v4", "serde"] }
directories = "5"
chrono = { version = "0.4", features = ["serde"] }
thiserror = "2"

# crates/app/Cargo.toml (主进程)
[dependencies]
common = { path = "../common" }
tokio = { version = "1", features = ["full", "process"] }
notify = "7"  # 配置文件变更监听

# crates/tray/Cargo.toml (托盘进程)
[dependencies]
common = { path = "../common" }
tray-icon = "0.19"
winit = "0.30"  # 事件循环

# crates/theater/Cargo.toml (剧场进程)
[dependencies]
common = { path = "../common" }
bevy = { version = "0.15", default-features = false, features = [
    "bevy_asset",
    "bevy_winit",
    "bevy_render",
    "bevy_pbr",
    "bevy_gltf",
    "bevy_animation",
    "bevy_state",
    "x11",
    "wayland",
] }

# crates/manager/Cargo.toml (管理器进程)
[dependencies]
common = { path = "../common" }
iced = { version = "0.13", features = ["tokio"] }
tokio = { version = "1", features = ["full"] }
reqwest = { version = "0.12", features = ["json"] }
```

## 8. 行为脚本示例

```ron
// assets/scripts/idle.ron
BehaviorScript(
    id: "idle",
    duration: None,  // 循环
    keyframes: [
        Keyframe(time: 0.0, action: PlayAnimation("breathe")),
        Keyframe(time: 3.0, action: Random(["idle", "idle", "walk"])),  // 30% 概率切换到 walk
    ],
    next: None,
    interruptible: true,
)

// assets/scripts/happy.ron
BehaviorScript(
    id: "happy",
    duration: Some(2.0),
    keyframes: [
        Keyframe(time: 0.0, action: PlayAnimation("jump")),
        Keyframe(time: 0.5, action: Scale(1.2)),
        Keyframe(time: 1.0, action: Scale(1.0)),
        Keyframe(time: 1.5, action: PlayAnimation("spin")),
    ],
    next: Some("idle"),
    interruptible: false,
)
```

## 9. 参考资料

- [Bevy 引擎文档](https://bevyengine.org/learn/)
- [Bevy Cheat Book](https://bevy-cheatbook.github.io/)
- [Iced 框架文档](https://docs.rs/iced)
- [Meshy AI API 文档](https://docs.meshy.ai)
- [tray-icon 文档](https://docs.rs/tray-icon)
