# Repository Guidelines

## Project Structure & Module Organization

Rust 2024 firmware uses `#![no_std]` and `#![no_main]` to blink LEDs on STM32F103, STM32F401, STM32F411, and nRF52840 boards.

- `src/main.rs`: entry point, board initialization, GPIO abstraction, and LED sequencing; board differences use `#[cfg(feature = "...")]`.
- `Cargo.toml`: board features, optional HAL dependencies, and release settings.
- `build.rs` and `memory-*.x`: select and supply the board-specific linker memory map.
- `.cargo/config.toml`: ARM linker flags and default compilation target.
- `build`: Bash helper for building, linting, and flashing.
- `target/`: ignored build artifacts. No dedicated tests or asset directories exist.

## Build, Test, and Development Commands

Use nightly from `rust-toolchain.toml`; install the appropriate ARM target with `rustup target add <target>`.

- `bash build b b`: release build for Blue Pill.
- `bash build k b`: release build for Black Pill F411.
- `bash build k401 b`: release build for Black Pill F401CC.
- `bash build s b`: release build for the nRF52840 dongle with S140.
- `bash build s c`: run Clippy for that board.
- `cargo fmt --all -- --check`: check Rust formatting; use `cargo fmt --all` to apply it.

See [README.md](README.md#supported-targets) for all seven board configurations and target mappings. Select exactly one board feature. Prefer the helper: it supplies `--no-default-features` and the matching target. The default feature is `nrf52840_dongle_sd`, while the configured default target is Cortex-M3.

Probe flashing (`f`) uses SWD at 100 kHz; override with `PROBE_RS_SPEED=<kHz>`.

## Documentation Maintenance

Keep Markdown current when changing boards, commands, memory layouts, or behavior. Maintain targets and flashing limitations in `README.md`; keep `CLAUDE.md` delegating here.

## Coding Style & Naming Conventions

Use rustfmt defaults, four-space indentation, `snake_case` functions and variables, and `UpperCamelCase` types and traits. Keep hardware-specific code behind board features. Explain pin mappings, clock settings, and unsafe-code invariants in nearby comments.

## Testing Guidelines

There is no automated test suite or coverage threshold; the firmware binary disables the Cargo test harness. Build and lint each affected board configuration. On hardware, verify the three-blink sequence, pauses, LED polarity, and RGB sequencing where applicable. Record boards tested and hardware checks omitted.

## Commit & Pull Request Guidelines

History uses short subjects such as `cargo update` and `Support nRF52840 with s140`; no enforced prefix scheme is evident. Use concise, specific subjects. Pull requests should describe behavior changes, affected boards, build/lint results, and hardware observations; link relevant issues. For memory-map changes, explain bootloader and SoftDevice reservations.
