# AGENTS.md

## Rule management

- Keep rules concise and actionable
- When user gives similar guidance 3+ times, propose adding as rule
- Only write to AGENTS.md after explicit user approval

## Language

- Code, comments, and data: English
- User-facing messages: may be localized
- Development discussions: Chinese

## Architecture

Multi-process desktop pet app with replay-based behavior system.
See `docs/architecture.md` for details.

Crates: `app`, `theater`, `tray`, `manager`, `common`.

## Toolchain

- Rust nightly (edition 2024, let-chains used throughout)

## Build commands

Use `justfile` for all operations (`just` to list):
- `just check` - fmt + clippy + test (run after changes)
- `just build` - build all crates
- `just clippy` - lint with `-D warnings`
- `just test` - run all tests
- `just fmt` - fix formatting (requires nightly)
- `just check-fmt` - check formatting (requires nightly)
- `just run` - run main app (builds first)
- `just run-theater` - run Bevy theater
- `just run-tray` - run tray process

Run a single test:
`cargo test --package common -- paths::tests::env_override_with_valid_dir`

Fix all linting errors before completing tasks.

## Git workflow

Before commit:
1. Run `just check`
2. Check `git status`

Provide two commit message options:
- Full: `<type>(<scope>): <description>` with body
- Title only: single line

Do not execute git commands directly - user will commit.

### Commit format

[Conventional Commits](https://www.conventionalcommits.org/):
- Types: `feat`, `fix`, `docs`, `style`, `refactor`, `perf`, `test`, `chore`
- Scope: crate names (`theater`, `common`, `tray`, `manager`, `app`)
- Breaking changes: append `!` (e.g., `feat!:`)
- Description: imperative, lowercase, no period

## Code organization

### Imports
- Add at file top, not inline full paths
- Group: std → external → crate modules
- Full paths only for name conflicts

### Functions
- Public API first, internal helpers second

### Comments
- No step numbers
- Explain why, not what

## Error handling

- `thiserror` for custom errors in `common`
- `.context()` for error propagation
- Error types in dedicated modules

## Logging

- Use `tracing`: `error`, `warn`, `info`, `debug`, `trace`
- Include context (entity IDs, script names, etc.)

## Documentation

- `///` for public API docs
- Reference `docs/` for requirements and architecture

## Markdown formatting

- Line length: 80 characters
- Rewrap: paragraphs, lists, comments
- Keep intact: tables, code blocks, URLs, commands
