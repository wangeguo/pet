# Pet Desktop Pet System Development Roadmap

Based on the [Product Document](product.md) and
[Architecture Document](architecture.md).

## Phase 1: Foundation Framework ✅

**Goal**: Build the project infrastructure and multi-process
communication mechanism

- [x] 1.1 Project initialization - Create Cargo workspace
- [x] 1.2 Create crate structure - app, common, tray,
  theater, manager
- [x] 1.3 Shared library `common` - Data models, config
  read/write, storage
- [x] 1.4 Config storage system - TOML config file read/write
- [x] 1.5 Main process framework - Process manager
- [x] 1.6 Inter-process communication - Config file-based
  state sync
- [x] 1.7 System tray process - tray-icon tray and menu

## Phase 2: Theater Process (Bevy) ✅

**Goal**: Implement pet 3D rendering and basic interaction

- [x] 2.1 Bevy App configuration
- [x] 2.2 Transparent borderless window
- [x] 2.3 Always-on-top display
- [x] 2.4 GLB model loading
- [x] 2.5 Camera setup
- [x] 2.6 Lighting setup
- [x] 2.7 Window dragging
- [x] 2.8 Click interaction

## Phase 3: Replay System ✅

**Goal**: Implement behavior script-driven pet action replay
system

- [x] 3.1 Script data structures - BehaviorScript, Keyframe,
  Action
- [x] 3.2 Script loader - RON format parsing
- [x] 3.3 Replay plugin - Script replay state management
- [x] 3.4 Keyframe system - Execute actions along the timeline
- [x] 3.5 Animation playback - PlayAnimation action
- [x] 3.6 Movement action - MoveTo
- [x] 3.7 Scale action - Scale
- [x] 3.8 Random selection - Random
- [x] 3.9 Script transition - Automatic transition via `next`
  field
- [x] 3.10 Built-in scripts - idle, walk, happy, sleep
- [x] 3.11 Interaction trigger - Click triggers script
  transition
- [x] 3.12 Bounce action - Bounce
- [x] 3.13 Spin action - Spin

## Phase 4: IPC Infrastructure ✅

**Goal**: Establish UDS bidirectional real-time communication
layer as the foundation for Brain

- [x] 4.1 IPC message types - Define IpcEnvelope, ProcessId,
  IpcMessage in common
- [x] 4.2 UDS server - UDS listener in the app process
- [x] 4.3 UDS client - Child process connection and message
  send/receive
- [x] 4.4 Message routing - MessageRouter in app, forwarding
  messages by target
- [x] 4.5 Theater IPC plugin - Bevy plugin bridging UDS and
  ECS events
- [x] 4.6 Integration tests - theater <-> app message
  round-trip verification

## Phase 5: Settings Center

**Goal**: Implement centralized configuration management
interface (Iced)

- [ ] 5.1 Refactor AppConfig - Grouped structure
  (GeneralSettings / AppearanceSettings /
  AiSettings / MeshySettings)
- [ ] 5.2 Config migration - Old flat structure -> new grouped
  structure, backward compatible
- [ ] 5.3 Create settings crate - Iced Application base
  framework
- [ ] 5.4 Settings: General - Auto-start, language
- [ ] 5.5 Settings: Appearance - Scale, position, always on
  top, opacity
- [ ] 5.6 Settings: AI - Provider, API key, model, endpoint,
  personality configuration
- [ ] 5.7 Settings: Meshy AI - API Key
- [ ] 5.8 Settings: About - Version, build info
- [ ] 5.9 Auto-start migration - Move from tray menu to
  Settings
- [ ] 5.10 Tray menu update - Add "Settings" entry

## Phase 6: Pet Manager

**Goal**: Implement pet creation and management interface
(technical approach TBD)

- [ ] 6.1 Determine technical approach - Evaluate Bevy / Web /
  other
- [ ] 6.2 Manager base framework
- [ ] 6.3 Create pet interface - Description input + Meshy AI
  integration
- [ ] 6.4 Generation progress display -
  Pending/InProgress/Succeeded
- [ ] 6.5 Pet list - Display/switch/delete pets
- [ ] 6.6 Model preview

## Phase 7: AI Brain

**Goal**: Implement LLM-driven intelligent behavior generation

- [ ] 7.1 brain crate basics - Entry point, UDS connection,
  resident event loop
- [ ] 7.2 rig.rs integration - Agent construction,
  multi-provider support (OpenAI / Anthropic / Ollama / Custom)
- [ ] 7.3 PetContext - Available animation list, interaction
  history, pet state management
- [ ] 7.4 PersonalityManager - Generate system prompts based
  on configuration
- [ ] 7.5 ScriptGenerator - Parse LLM output into
  BehaviorScript
- [ ] 7.6 Script validation - Ensure referenced animations and
  action types are valid
- [ ] 7.7 Fallback strategy - Fall back to built-in scripts
  when AI is unavailable
- [ ] 7.8 Process management - Add brain to app
  ProcessManager (resident process, starts with tray)
- [ ] 7.9 Theater refactoring - Report interaction events to
  Brain, receive remote script execution
- [ ] 7.10 AnimationMap notification - Theater sends model
  animation list to Brain via IPC
- [ ] 7.11 End-to-end integration - Click -> brain -> LLM ->
  script -> theater replay

## Phase 8: Experience Polish

**Goal**: Improve stability and refine AI interaction experience

- [ ] 8.1 First-run guide - Auto-open manager to guide pet
  creation
- [ ] 8.2 Error handling - Error handling and user
  notifications across modules
- [ ] 8.3 Logging system - Log recording and debug support
- [ ] 8.4 Thinking animation - Play transition animation
  during LLM wait
- [ ] 8.5 Conversation memory - Sliding window management of
  AI interaction history
- [ ] 8.6 Sound effects - Behavior sound effects (e.g., duck
  "quack")
- [ ] 8.7 Text bubbles - Display AI reply text in Theater
- [ ] 8.8 Window position memory - Save and restore pet window
  position
- [ ] 8.9 Graceful shutdown - Cleanup on exit
- [ ] 8.10 Cross-platform support - macOS / Windows / Linux
  (X11/Wayland)
- [ ] 8.11 Performance optimization - Rendering performance,
  LLM call caching

## Phase 9: Voice Interaction (Future)

**Goal**: Implement voice input and output

- [ ] 9.1 STT integration - whisper.cpp or cloud API
- [ ] 9.2 TTS integration - Text-to-speech
- [ ] 9.3 Microphone management - Device selection + VAD
  (Voice Activity Detection)
- [ ] 9.4 Wake word detection
- [ ] 9.5 Voice Settings UI - Voice-related settings interface
- [ ] 9.6 Audio output - Voice and sound effect playback

## Phase 10: Advanced Features (Future)

**Goal**: Advanced integration and extensibility

- [ ] 10.1 Screen awareness - Read active window info,
  context-aware interaction
- [ ] 10.2 MCP integration - Brain as MCP host, calling
  external tools
- [ ] 10.3 External system integration - Brain as orchestrator
  interfacing with external agent systems (e.g., OpenClaw),
  or building a similar multi-agent collaboration platform
