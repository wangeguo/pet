# Pet

AI-driven intelligent desktop companion. Generate personalized
3D pet models through AI, and pets reside on your desktop as
floating windows. The built-in AI Brain understands user
interaction intent and dynamically generates pet behaviors,
delivering a warm and intelligent companionship experience.

## Features

- **3D Desktop Pet** — GLB/glTF models rendered via Bevy in a
  transparent, borderless, always-on-top window
- **Replay-Based Behavior** — deterministic behavior scripts
  drive pet actions (idle, walk, happy, sleep, bounce, spin)
- **Drag & Click Interaction** — drag your pet anywhere on the
  desktop, click to trigger reactions
- **AI Brain** (planned) — LLM-powered behavior generation
  that understands user intent and dynamically creates behavior
  scripts
- **AI Pet Generation** (planned) — create unique 3D pet
  models from text descriptions via Meshy AI
- **System Tray** — quick access to show/hide pet, open
  manager, settings, and exit
- **Cross-Platform** — macOS, Linux, and Windows support

## Architecture

Multi-process architecture with replay-based behavior system:

```
                    ┌─────────┐
                    │   app   │  Main process:
                    │ (router)│  process management & IPC routing
                    └────┬────┘
           ┌─────────┬───┼───┬──────────┐
           │         │       │          │
       ┌───┴──┐ ┌────┴──┐ ┌─┴──────┐ ┌─┴───────┐
       │ tray │ │theater│ │ brain  │ │ manager │
       │      │ │(Bevy) │ │(rig.rs)│ │         │
       └──────┘ └───────┘ └────────┘ └─────────┘
```

| Crate | Role |
|-------|------|
| `app` | Main process — spawns and monitors child processes |
| `common` | Shared library — config, models, scripts, paths, errors |
| `theater` | Bevy-based 3D renderer and behavior replayer |
| `tray` | System tray icon and menu |
| `brain` | AI agent — LLM interaction and script generation (planned) |
| `manager` | Pet creation and management UI (planned) |

See [docs/architecture.md](docs/architecture.md) for full
details.

## Prerequisites

- [Rust](https://rustup.rs/) (nightly toolchain, edition 2024)
- [just](https://github.com/casey/just) command runner
- Platform-specific dependencies (Linux only):

  ```bash
  just install-deps
  ```

## Quick Start

```bash
# Build all crates
just build

# Run the application
just run
```

## Development

All development commands are available via `just`:

```bash
just              # List all commands
just check        # Format + lint + test
just build        # Build all crates
just clippy       # Lint with -D warnings
just test         # Run all tests
just fmt          # Fix formatting (nightly)
just check-fmt    # Check formatting (nightly)
```

Run individual processes:

```bash
just run          # Main app (starts all processes)
just run-theater  # Bevy theater only
just run-tray     # System tray only
```

Run a single test:

```bash
cargo test --package common -- paths::tests::env_override_with_valid_dir
```

## Project Status

The project is in active development. Current progress:

- [x] **Phase 1** — Multi-process framework and IPC
- [x] **Phase 2** — Bevy theater with 3D rendering and
  interaction
- [x] **Phase 3** — Replay behavior system with scripted
  actions
- [ ] **Phase 4** — UDS-based real-time IPC
- [ ] **Phase 5** — Settings center (Iced)
- [ ] **Phase 6** — Pet manager with Meshy AI integration
- [ ] **Phase 7** — AI Brain with LLM-driven behaviors
- [ ] **Phase 8** — Polish and cross-platform refinement
- [ ] **Phase 9** — Voice interaction (future)

See [docs/roadmap.md](docs/roadmap.md) for the full roadmap.

## Documentation

- [Product Design](docs/product.md) — vision, features, and
  user experience
- [Architecture](docs/architecture.md) — technical design,
  data models, and IPC protocol
- [Roadmap](docs/roadmap.md) — development phases and
  milestones

## License

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

<http://www.apache.org/licenses/LICENSE-2.0>

Unless required by applicable law or agreed to in writing,
software distributed under the License is distributed on an
"AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND,
either express or implied. See the License for the specific
language governing permissions and limitations under the
License.
