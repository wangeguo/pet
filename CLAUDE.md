# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build Commands

All operations use `just` (see `justfile`):

- `just build` - build all crates
- `just check` - run fmt + clippy + test (run after every change)
- `just clippy` - lint with `-D warnings`
- `just test` - run all tests
- `just fmt` - fix formatting (requires nightly)
- `just check-fmt` - check formatting (requires nightly)
- `just run` - run main app (builds first)
- `just run-theater` - run Bevy theater process
- `just run-tray` - run system tray process

Run a single test: `cargo test --package common -- paths::tests::env_override_with_valid_dir`

## Architecture

Multi-process desktop pet app. Bevy and Iced each own their event loop, so they run as separate OS processes coordinated by a parent.

### Process Model

**`app`** (main process) - Tokio-based orchestrator. `ProcessManager` spawns child
processes, watches `~/.config/pet/` with `notify` for config/state changes,
and shuts everything down on Ctrl-C or tray exit.

**`theater`** (Bevy 0.18) - 3D pet rendering in a transparent, always-on-top,
borderless window. Organized as ECS plugins (`PetPlugin`, `ReplayPlugin`,
`InteractionPlugin`) with systems in `systems/` for animation, movement,
scale, keyframe execution, and script transitions.

**`tray`** (tray-icon + winit) - System tray icon and menu. Communicates
with other processes by writing `state.toml`; the main process detects
changes via file watcher.

**`manager`** (Iced, placeholder) - Pet management GUI, launched on first run
or from tray menu. Currently a stub.

**`common`** - Shared library: data models (`Pet`, `WindowPosition`), config
loading/saving (`AppConfig`, `AppState` as TOML), `BehaviorScript` with RON
serialization, `AppPaths` for platform-aware directories, error types.

### Replay System

Pet behavior uses a replay architecture: `BehaviorScript` defines a sequence
of `Keyframe`s with timed `Action`s (play animation, move, scale, random
next script). The theater "replays" scripts rather than computing behavior
in real time. Built-in scripts: idle, walk, happy, sleep. Scripts are RON
files stored in `~/.local/share/pet/scripts/`.

### Inter-Process Communication

Processes share state through TOML files in `~/.config/pet/`:
- `config.toml` - pet list, active pet, window position, scale, API key
- `state.toml` - runtime flags (pet_visible, theater_running, manager_open)

The main process watches these files with `notify` and reacts to changes.

## Conventions

- Rust nightly toolchain (edition 2024, let-chains used throughout)
- `thiserror` for error types, `tracing` for logging
- Imports: std -> external -> crate modules (at file top, not inline)
- Conventional Commits: `<type>(<scope>): <description>` where scope is
  the crate name (theater, common, tray, manager, app)
- Behavior scripts use RON format, 3D models use GLB
- Assets dir resolved via 4-layer lookup (env var, exe-relative,
  system paths, dev fallback) - see `AppPaths::find_assets_dir`
