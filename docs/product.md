# Pet - Desktop Pet System Product Design Document

## 1. Product Overview

### 1.1 Product Positioning

Pet is an AI-driven intelligent desktop companion. Users can
generate personalized 3D pet models through AI, and pets reside
on the desktop as floating windows. The built-in AI Brain
understands user interaction intent and dynamically generates pet
behaviors, delivering a warm and intelligent companionship
experience.

### 1.2 Core Values

- **Companionship**: Pets reside on the desktop, providing
  continuous emotional companionship
- **Personalization**: Generate unique pet appearances through
  Meshy AI
- **Interactivity**: Supports drag, click, and other interaction
  methods
- **Intelligence**: Understands user intent through LLM and
  dynamically generates behavioral responses

### 1.3 Target Users

- Users who want personalized desktop decoration
- Users who enjoy virtual pets
- Users interested in AI-generated content

## 2. Feature Design

### 2.1 Feature Architecture

```
┌──────────────────────────────────────────────────────┐
│               Pet Intelligent Desktop Companion       │
├──────────────────────────────────────────────────────┤
│                                                      │
│  ┌──────────────────────────────────────────────┐   │
│  │          Pet Theater (Core Experience)        │   │
│  ├──────────────────────────────────────────────┤   │
│  │  3D Rendering   Behavior Replay   Drag       │   │
│  │  Animations     Floating Window   Always Top │   │
│  └──────────────────────────────────────────────┘   │
│                     ▲          ▲                     │
│       Script Push   │          │  Interaction Events │
│                     │          │                     │
│  ┌──────────────────┴──────────┴─────────────┐      │
│  │          AI Brain (Resident Process)       │      │
│  ├────────────────────────────────────────────┤      │
│  │  LLM Interaction   Dynamic Script Gen     │      │
│  │  Personality System  Tools/Agent           │      │
│  │  Context Management  Fallback             │      │
│  └────────────────────────────────────────────┘      │
│                                                      │
│  ┌─────────────┐ ┌────────────┐ ┌────────────┐     │
│  │ System Tray │ │ Pet Manager│ │  Settings  │     │
│  ├─────────────┤ ├────────────┤ ├────────────┤     │
│  │ Open Manager│ │ Create Pet │ │ General    │     │
│  │ Open Settings│ │ Pet List  │ │ Appearance │     │
│  │ Show/Hide  │ │ Model View │ │ AI         │     │
│  │ Quit       │ │ Skills Cfg │ │ Meshy AI   │     │
│  └─────────────┘ └────────────┘ └────────────┘     │
│                                                      │
│  ┌──────────────────────────────────────────────┐   │
│  │              Future Extensions                │   │
│  │  Voice Interaction  Screen Awareness          │   │
│  │  External Integration  Pet Ecosystem          │   │
│  │  Pet Marketplace                              │   │
│  └──────────────────────────────────────────────┘   │
│                                                      │
└──────────────────────────────────────────────────────┘
```

### 2.2 Core Features

#### 2.2.1 Pet Theater (Primary Feature)

The Pet Theater is the main interface that users see and interact
with daily. It uses a Replay architecture to play back pet
behaviors. The theater itself is a pure script player — all
behavior scripts come from AI Brain dynamic generation or the
built-in script library.

| Feature | Description |
|---------|-------------|
| Floating Window | Borderless transparent window, displayed at the bottom-right of the desktop by default |
| 3D Rendering | Renders GLB format 3D models |
| Behavior Replay | Receives and replays behavior scripts |
| Auto Animation Detection | Automatically detects built-in model animations and reports to AI Brain |
| Drag to Move | Supports mouse drag to move to any position |
| Always on Top | Always displayed above other windows |
| Interaction Events | Click and other interaction events are reported to AI Brain for processing |

#### 2.2.2 System Tray

The system tray provides quick access and runs persistently in
the background.

| Feature | Description |
|---------|-------------|
| Tray Icon | Displays the app icon, indicating the program is running |
| Open Manager | Opens the pet management interface |
| Open Settings | Opens the settings center |
| Show/Hide Pet | Quickly toggles pet visibility |
| Quit | Completely exits the application |

#### 2.2.3 Pet Manager

The manager is a separate process, opened on demand via the tray
menu. It may involve heavy rendering scenarios like 3D model
previews in the future — the technical approach is TBD
(Bevy / Web / other). It is decoupled from the settings center
to evolve independently.

| Feature | Description |
|---------|-------------|
| Create Pet | Enter a description, generate a 3D model via Meshy AI |
| Pet List | Display all created pets |
| Model Preview | Preview pet 3D models |
| Switch Pet | Select which pet to display |
| Delete Pet | Delete a pet and its model files |
| Pet Naming | Set/modify a pet's name |

**Future Expansion:**

- Pet Skills configuration (behavior capability customization)
- Pet appearance/variant management
- Pet ecosystems (League of Legends, Warcraft, Usagi,
  Ultraman...)
- Select from official lists or create custom ones

**Pet Creation Flow:**

```
Tray Menu → Open Manager → Enter Description →
Submit Generation → Wait for Processing →
Preview & Confirm → Save and Display
```

#### 2.2.4 Pet Behavior System

The behavior system uses a Replay architecture with two driving
modes:

**AI-Driven Mode** (default, requires LLM configuration):

Interaction events are sent to the AI Brain, which understands
intent through LLM and dynamically generates behavior scripts.
The AI Brain is aware of the animation list supported by the pet
model and generates scripts that fully leverage the model's
animation capabilities.

**Fallback Mode** (when AI is unavailable):

Uses built-in behavior scripts, ensuring pets function normally
without AI configuration.

| State | Description | Trigger |
|-------|-------------|---------|
| Idle | Standby state, subtle breathing animation | Default state |
| Walking | Walking state, moves left and right | Random trigger |
| Happy | Happy state, jumping animation | User click |
| Sleeping | Sleep state, eyes closed, still | Long period of no interaction |
| Bounce | Bounce state, bouncing up and down | Random trigger |
| Spin | Spin state, rotating in place | Random trigger |

**State Transition Diagram (Fallback Mode):**

```
        ┌──────────────────────────────┐
        │                              │
        ▼                              │
    ┌───────┐  Random   ┌─────────┐   │
    │ Idle  │ ────────► │ Walking │ ──┘
    └───────┘           └─────────┘
        │ │ \
  Click │ │  └── Random ──► ┌────────┐ ──► Idle
        │ │                 │ Bounce │
        │ │                 └────────┘
        │ └── Random ──► ┌──────┐ ──► Idle
        │                │ Spin │
        │                └──────┘
        ▼
    ┌───────┐            ┌───────┐
    │ Happy │ ─────────► │ Idle  │
    └───────┘  Done      └───────┘

    Long period of no interaction
        │
        ▼
    ┌──────────┐  Any interaction  ┌──────┐
    │ Sleeping │ ────────────────►│ Idle │
    └──────────┘                  └──────┘
```

In AI-Driven Mode, state transitions are decided by the LLM,
unrestricted by the fixed rules above, enabling richer and more
creative behavior combinations.

#### 2.2.5 AI Brain

The AI Brain is the system's "brain", responsible for
understanding user intent and controlling pet behavior.

| Feature | Description |
|---------|-------------|
| Interaction Understanding | Receives interaction events from the theater and understands user intent |
| Dynamic Script Generation | Calls LLM to dynamically generate behavior scripts |
| Personality System | Generates differentiated responses based on configured pet personality |
| Context Management | Maintains interaction history and pet state context |
| Fallback | Automatically switches to built-in scripts when AI is unavailable |

**AI-Driven Flow:**

```
User Interaction → Theater Reports Event → AI Brain Receives →
Build Prompt → Call LLM → Generate Behavior Script →
Push to Theater → Replay Pet Action
```

**During LLM wait time**, the theater plays a "thinking"
transition animation and seamlessly switches when the response
arrives.

#### 2.2.6 Settings Center

The settings center is a separate process (Iced), opened on
demand via the tray menu. It is a pure form UI, separated from
the manager to stay lightweight.

| Section | Configuration Items |
|---------|-------------------|
| General | Auto-start on boot, language, update checking |
| Appearance | Pet scale, default position, window size, always on top, opacity |
| AI | LLM provider, API Key, model, endpoint, personality configuration |
| Meshy AI | Meshy API Key |
| About | Version number, build info, project link |

#### 2.2.7 Voice Interaction (Future)

Future support for voice input/output for more natural
interaction.

| Feature | Description |
|---------|-------------|
| Voice Input | Capture user voice via microphone, transcribe and send to AI Brain |
| Voice Response | AI Brain's text replies are synthesized as audio via TTS |
| Sound Effects | Pet behaviors accompanied by sound effects (e.g., duck "quack") |
| Wake Word | Activate voice interaction via wake word |

## 3. User Experience Flows

### 3.1 First-Time Use

```
Launch App → Display Tray Icon → Auto-Open Manager →
Guide User to Create First Pet → Pet Appears on Desktop
```

### 3.2 Daily Use

```
Auto-Start on Boot → Tray Icon Persists →
Pet Automatically Appears on Desktop →
User Interacts with Pet → AI Brain Generates Response →
Pet Reacts
        │
        └─ When management needed:
           Tray Menu → Open Manager
```

### 3.3 Typical Scenarios

| Scenario | User Action |
|----------|-------------|
| Daily Companionship | Pet runs on desktop automatically, occasional click interactions |
| AI Interaction | Click Pet → AI Thinks → Pet Makes Intelligent Response |
| Create New Pet | Tray → Manager → Create |
| Switch Pet | Tray → Manager → Select Another Pet |
| Temporarily Hide | Tray → Hide Pet |
| Move Position | Directly drag the pet floating window |
| Modify Settings | Tray → Settings Center |

## 4. Related Documents

- [Technical Architecture](architecture.md) - Technical
  architecture, data models, dependency list, etc.
- [Development Roadmap](roadmap.md) - Detailed development plan
  and task list
