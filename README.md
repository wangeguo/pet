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
  drive pet actions
- **Drag & Click Interaction** — drag your pet anywhere on the
  desktop, click to trigger reactions
- **AI Brain** — LLM-powered behavior generation that
  understands user intent and dynamically creates scripts
- **AI Pet Generation** — create unique 3D pet models from
  text descriptions via Meshy AI
- **System Tray** — quick access to show/hide pet, open
  manager, settings, and exit
- **Cross-Platform** — macOS, Linux, and Windows support

## Architecture

Multi-process architecture with replay-based behavior system:

```
                  app (main process)
                   │  process mgmt & IPC routing
     ┌──────┬──────┼───────┬─────────┬──────────┐
     │      │      │       │         │          │
   tray  theater  brain  manager  settings  common
```

| Crate | Role |
|-------|------|
| `app` | Main process — spawns and monitors child processes |
| `common` | Shared library — config, models, scripts, paths, errors |
| `theater` | Bevy-based 3D renderer and behavior replayer |
| `tray` | System tray icon and menu |
| `brain` | AI agent — LLM interaction and script generation |
| `manager` | Pet creation and management UI |
| `settings` | Configuration management UI (Iced) |

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
just run           # Main app (starts all processes)
just run-theater   # Bevy theater only
just run-tray      # System tray only
just run-manager   # Pet manager only
just run-settings  # Settings only
```

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

    http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
