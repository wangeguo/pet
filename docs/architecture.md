# Pet - Desktop Pet System Technical Design Document

## 1. Technology Stack

| Area | Technology | Rationale |
|------|-----------|-----------|
| Pet Theater | Bevy | Mature Rust game engine with native GLB/glTF support, built-in ECS and animation system |
| Settings Center | Iced | Rust-native, declarative UI, ideal for form-based configuration interfaces |
| Pet Manager | TBD | May involve 3D preview in the future, technical approach pending |
| System Tray | tray-icon | Rust-native system tray support |
| AI Brain | rig.rs | Rust-native LLM framework with multi-provider and Tool calling support |
| Pet Generation | Meshy AI | Supports text-to-3D model generation |
| Real-time Communication | Unix Domain Socket | Low-latency bidirectional IPC for local processes |
| Configuration Storage | config crate | Supports multiple formats (TOML/YAML/JSON, etc.) |
| Model Format | GLB | Universal 3D model format with native Bevy support |

## 2. Replay Architecture

The Pet Theater uses a Replay architecture similar to game
replay systems. Pet behaviors are not computed in real-time but
defined as pre-authored behavior scripts that the theater
process "replays."

### 2.1 Core Concepts

```
┌──────────────────────────────────────────────────┐
│                Replay Architecture                │
├──────────────────────────────────────────────────┤
│                                                  │
│  ┌──────────┐    ┌──────────┐    ┌──────────┐   │
│  │ Behavior │    │ Replayer │    │ Rendered │   │
│  │ Script   │ ─► │ (Theater)│ ─► │ Output   │   │
│  └──────────┘    └──────────┘    └──────────┘   │
│       │               │                         │
│       │               ▼                         │
│       │          ┌──────────┐                   │
│       │          │Interaction│                   │
│       │          │ Events   │                   │
│       │          └──────────┘                   │
│       │               │                         │
│       │               ▼                         │
│       │          ┌──────────┐                   │
│       └────────► │ Trigger  │                   │
│                  │ New Script│                   │
│                  └──────────┘                   │
│                                                  │
└──────────────────────────────────────────────────┘
```

### 2.2 Game Replay Analogy

| Game Replay | Pet Theater |
|-------------|-------------|
| Replay file (.w3g/.dem) | Behavior script (BehaviorScript) |
| Game engine | Bevy rendering engine |
| Replay player | Theater process |
| Player action records | State transitions + animation sequences |
| Watching replay | Pet "performing" on desktop |

### 2.3 Replay Flow

```
1. Load pet model and behavior script library
2. Enter default script (idle)
3. Execute keyframe actions along the timeline
4. When script ends, switch based on the `next` field
5. When interaction events are received, switch to the
   corresponding script
6. Loop
```

### 2.4 Advantages

- **Determinism**: Same script produces same behavior, easy to
  debug
- **Extensibility**: Adding new behaviors only requires writing
  new scripts
- **Low Coupling**: Behavior logic is separated from the
  rendering engine
- **Archivability**: Scripts can be serialized, stored, and
  shared

### 2.5 AI-Driven Script Generation

Building on the Replay architecture, the AI Brain is introduced
for dynamic script generation. The theater maintains its pure
replayer role, with script sources expanding from "built-in
only" to "built-in + AI dynamically generated."

```
┌─────────────────────────────────────────────┐
│              Script Sources                  │
├─────────────────────────────────────────────┤
│                                             │
│  ┌────────────┐        ┌────────────┐      │
│  │  Built-in  │        │  AI Brain  │      │
│  │  Scripts   │        │            │      │
│  └─────┬──────┘        └─────┬──────┘      │
│        │                     │              │
│        │  Fallback    LLM Dynamic Gen       │
│        │                     │              │
│        └────────┬────────────┘              │
│                 ▼                            │
│         ┌──────────────┐                    │
│         │BehaviorScript│                    │
│         └──────┬───────┘                    │
│                ▼                             │
│         ┌──────────────┐                    │
│         │   Theater    │                    │
│         │  (Replayer)  │                    │
│         └──────────────┘                    │
│                                             │
└─────────────────────────────────────────────┘
```

**Workflow:**

1. Theater loads the GLB model and auto-detects all available
   animations
2. Theater reports the available animation list to Brain via IPC
3. User interaction events (clicks, etc.) are reported from
   Theater to Brain
4. Brain builds a prompt (system personality + available
   animations + context)
5. Brain calls LLM to generate a BehaviorScript (JSON format)
6. Brain validates the script and pushes it to Theater
7. Theater receives and replays the script

**Fallback Strategy**: When LLM is unavailable (no API key,
network failure, timeout), Brain falls back to the built-in
script library, ensuring the pet is always functional.

## 3. Process Architecture

Since Bevy and Iced each have their own independent event loops,
a multi-process architecture is used:

```
┌───────────────────────────────────────────────────────┐
│                    Main Process                        │
│  ┌─────────────────────────────────────────────────┐ │
│  │       Process Manager + Message Router          │ │
│  │  - Start/stop/monitor child processes           │ │
│  │  - UDS server, centralized message routing      │ │
│  │  - Config file change listener                  │ │
│  └─────────────────────────────────────────────────┘ │
│      │        │        │        │         │          │
│  ┌───┴──┐ ┌──┴───┐ ┌──┴──┐ ┌──┴────┐ ┌──┴──────┐  │
│  │ tray │ │brain │ │thea-│ │mana-  │ │settings │  │
│  │      │ │ (AI) │ │ter  │ │ger    │ │  Iced   │  │
│  │always│ │always│ │     │ │       │ │         │  │
│  └──────┘ └──────┘ └─────┘ └───────┘ └─────────┘  │
│      │        │        │        │         │          │
│      └────────┴────────┴────────┴─────────┘          │
│                        │                              │
│              ┌─────────┴─────────┐                    │
│              │   config.toml     │                    │
│              │ (persistent cfg)  │                    │
│              └───────────────────┘                    │
└───────────────────────────────────────────────────────┘
```

### 3.1 Process Responsibilities

| Process | Tech Stack | Responsibility | Lifecycle |
|---------|-----------|----------------|-----------|
| Main Process | tokio | Process management, message routing | Resident |
| Tray Process | tray-icon | System tray, menu interaction | Resident |
| AI Brain | rig.rs + tokio | LLM interaction, script generation | Resident |
| Theater Process | Bevy | Script replay, 3D rendering, interaction | Runs when pet is visible |
| Manager Process | TBD | Pet creation/management/preview | On-demand |
| Settings Process | Iced | Configuration management | On-demand |

### 3.2 Inter-Process Communication

A two-layer IPC strategy is used:

| Layer | Mechanism | Purpose | Latency |
|-------|-----------|---------|---------|
| Real-time Messaging | Unix Domain Socket | Interaction events, script push, state sync | < 1ms |
| Persistent Config | config.toml + notify | Configuration changes | 50-500ms |

**Real-time Messaging Layer (UDS)**

Star topology with the main process as the central router:

```
tray ──UDS──> app <──UDS── theater
               |
             brain
             /   \
        manager  settings
```

Each child process connects to the main process's UDS server
upon startup and registers its `ProcessId`. The main process
handles message routing: forwarding `PetClicked` from theater
to brain, and `ExecuteScript` from brain to theater.

UDS path: `{config_dir}/pet.sock`

**Message Protocol** (defined in `common` crate):

```rust
/// IPC message envelope
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

    // General
    ProcessReady,
    Shutdown,
    Ping,
    Pong,
}
```

**Persistent Config Layer**

Retains the existing `config.toml` + `notify` mechanism for
low-frequency configuration changes (pet list, API Key, settings
items, etc.).

### 3.3 Startup Flow

```
app (main process)
|
+-- 1. Read configuration
+-- 2. Start UDS server
+-- 3. Start resident processes
|      ├── spawn: tray
|      └── spawn: brain
+-- 4. If there is an active pet, start theater
|      └── spawn: theater --pet-id=<uuid>
+-- 5. On first run, start manager
|      └── spawn: manager
+-- 6. Enter event loop
       - Monitor child process status
       - Route IPC messages
       - Listen for config file changes
```

### 3.4 Process Communication Examples

```
User clicks pet (AI-Driven Mode):
  theater -> UDS -> app -> UDS -> brain
  brain calls LLM, generates script
  brain -> UDS -> app -> UDS -> theater
  theater replays script

User switches pet in manager:
  manager -> writes config.toml ->
  app detects change -> restarts theater
  (brain stays resident)
```

## 4. Project Structure

Organized as a Cargo workspace with 7 crates (app, common, tray,
theater, brain, manager, settings):

```
pet/
├── Cargo.toml
├── docs/
│   ├── product.md
│   ├── architecture.md
│   └── roadmap.md
├── crates/
│   ├── app/                     # Main process
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── main.rs          # Entry point
│   │       ├── process.rs       # Child process management
│   │       └── router.rs        # IPC message routing
│   │
│   ├── tray/                    # Tray process
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── main.rs
│   │       └── menu.rs
│   │
│   ├── theater/                 # Theater process (Bevy)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── main.rs
│   │       ├── app.rs
│   │       ├── plugins/
│   │       │   ├── mod.rs
│   │       │   ├── pet.rs       # Model loading, rendering
│   │       │   ├── replay.rs    # Script replay
│   │       │   ├── interaction.rs # Drag, click
│   │       │   └── ipc.rs       # IPC send/receive
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
│   ├── brain/                   # AI Brain process
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── main.rs          # Entry, UDS connection
│   │       ├── agent.rs         # rig.rs Agent config
│   │       ├── script_gen.rs    # LLM -> BehaviorScript
│   │       ├── personality.rs   # Pet personality management
│   │       ├── context.rs       # Context management
│   │       ├── handlers.rs      # IPC message handlers
│   │       └── tools/           # rig.rs Tool implementations
│   │           ├── mod.rs
│   │           ├── animation.rs # Query available animations
│   │           └── script.rs    # Generate scripts
│   │
│   ├── manager/                 # Manager process (TBD)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       └── main.rs
│   │
│   ├── settings/                # Settings process (Iced)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── main.rs
│   │       ├── app.rs
│   │       └── views/
│   │           ├── mod.rs
│   │           ├── general.rs   # General section
│   │           ├── appearance.rs # Appearance section
│   │           ├── ai.rs        # AI section
│   │           ├── meshy.rs     # Meshy AI section
│   │           └── about.rs     # About section
│   │
│   └── common/                  # Shared library
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs
│           ├── models.rs        # Data models
│           ├── config.rs        # Config read/write
│           ├── script.rs        # Behavior script definitions
│           ├── ipc.rs           # IPC message types
│           ├── paths.rs         # Path management
│           ├── autostart.rs     # Auto-start on boot
│           ├── error.rs         # Error types
│           └── storage.rs       # Storage service
│
└── assets/
    ├── scripts/                 # Built-in behavior scripts
    │   ├── idle.ron
    │   ├── walk.ron
    │   ├── happy.ron
    │   ├── sleep.ron
    │   ├── bounce.ron
    │   └── spin.ron
    └── pets/                    # Pet models
```

## 5. Data Models

### 5.1 Pet Data

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

### 5.2 Configuration Data

Configuration uses a grouped structure for easy section-based
management in the Settings interface:

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
    endpoint: Option<String>,   // Custom endpoint
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
    name: String,               // Pet name
    traits: Vec<String>,        // ["cheerful", "curious"]
    custom_prompt: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct MeshySettings {
    api_key: Option<String>,
}
```

### 5.3 Behavior Script

```rust
/// Behavior script - defines a complete behavior sequence
#[derive(Serialize, Deserialize)]
struct BehaviorScript {
    id: String,
    duration: Option<Duration>,
    keyframes: Vec<Keyframe>,
    next: Option<String>,
    interruptible: bool,
}

/// Keyframe - state at a specific moment
#[derive(Serialize, Deserialize)]
struct Keyframe {
    time: f32,
    action: Action,
}

/// Action types
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

### 5.4 Bevy Components

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

### 5.5 Storage Locations

| Data Type | Storage Location |
|-----------|-----------------|
| Config file | `~/.config/pet/config.toml` |
| Runtime state | `~/.config/pet/state.toml` |
| UDS socket | `~/.config/pet/pet.sock` |
| Pet models | `~/.local/share/pet/models/` |
| Behavior scripts | `~/.local/share/pet/scripts/` |
| Log files | `~/.local/share/pet/logs/` |

## 6. Brain Architecture Design

### 6.1 Core Responsibilities

The Brain serves as the AI hub, transforming user interactions
into pet behaviors:

```
Interaction Event → Prompt Construction → LLM Call →
Script Parsing & Validation → Push to Theater
```

### 6.2 rig.rs Integration

Uses rig.rs to build an LLM Agent with multi-provider support:

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

### 6.3 Context Management

`PetContext` maintains all context needed for Brain
decision-making:

- Animation list supported by the pet model (obtained from
  Theater)
- Available Action type list
- Recent interaction history
- Current pet state (currently executing script, etc.)
- Pet personality configuration

### 6.4 LLM Latency Handling

LLM calls typically take 1-3 seconds. During the wait:

1. Brain immediately sends
   `AiThinking { is_thinking: true }`
2. Theater plays a "thinking" transition animation
3. When LLM response arrives, the script is sent and seamlessly
   transitioned

### 6.5 Fallback Strategy

When AI is unavailable (no API key / network failure / timeout):

1. Brain falls back to built-in script mapping (e.g., click →
   happy script)
2. Behavior is identical to when AI is disabled — transparent to
   the user
3. Automatically switches back to AI mode when restored

## 7. Meshy AI Integration

### 7.1 Workflow

```
┌─────────┐  ┌─────────┐  ┌──────────┐  ┌──────────┐
│ Submit  │ ►│ Get ID  │ ►│  Poll    │ ►│ Download │
│ Task    │  │         │  │  Status  │  │  Model   │
└─────────┘  └─────────┘  └──────────┘  └──────────┘
     │            │             │             │
     ▼            ▼             ▼             ▼
  POST /v2/   Returns      GET /v2/      GET model_url
  text-to-3d  task_id      text-to-3d/   Download GLB
                           {task_id}
```

### 7.2 API Calls

**Create Task:**
```
POST https://api.meshy.ai/v2/text-to-3d
{
    "mode": "preview",
    "prompt": "user-provided description",
    "art_style": "realistic"
}
```

**Query Status:**
```
GET https://api.meshy.ai/v2/text-to-3d/{task_id}
```

**Status Values:**
- `PENDING` - Waiting to be processed
- `IN_PROGRESS` - Processing
- `SUCCEEDED` - Completed
- `FAILED` - Failed

## 8. Dependency List

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

# crates/manager (technical approach TBD)
[dependencies]
common = { path = "../common" }
reqwest = { version = "0.12", features = ["json"] }
```

## 9. Behavior Script Examples

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

## 10. References

- [Bevy Engine Documentation](https://bevyengine.org/learn/)
- [Bevy Cheat Book](https://bevy-cheatbook.github.io/)
- [Iced Framework Documentation](https://docs.rs/iced)
- [rig.rs Documentation](https://docs.rig.rs/)
- [Meshy AI API Documentation](https://docs.meshy.ai)
- [tray-icon Documentation](https://docs.rs/tray-icon)
