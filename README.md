# cortex-m-test2

Rust 2024 bare-metal LED and clock-output demo for STM32 and Nordic Cortex-M
boards. The firmware uses `no_std`, `cortex-m-rt`, board-specific HALs, and
`panic-halt`.

The primary LED repeats three 100 ms pulses, each followed by 400 ms off, then
an additional 1 s pause. On nRF52840 boards, three secondary LEDs (RGB channels
on the dongle) cycle during the primary LED's off intervals.

## Supported targets

Choose exactly one board feature. The helper script selects its HAL, Rust
target, and probe chip identifier automatically.

| Board / configuration | Selector | Cargo feature | Rust target | Linker memory map |
| --- | --- | --- | --- | --- |
| Blue Pill (STM32F103C8) | `b` | `blue_pill` | `thumbv7m-none-eabi` | `memory-stm32f103.x` |
| Black Pill (STM32F401CC) | `k401` | `black_pill_f401` | `thumbv7em-none-eabihf` | `memory-stm32f401.x` |
| Black Pill (STM32F411CE) | `k` | `black_pill` | `thumbv7em-none-eabihf` | `memory-stm32f411.x` |
| Nucleo F411 (STM32F411RE) | `n` | `nucleo_f411` | `thumbv7em-none-eabihf` | `memory-stm32f411.x` |
| nRF52840 dongle, USB DFU bootloader | `r` | `nrf52840_dongle` | `thumbv7em-none-eabihf` | `memory-nrf52840-dongle.x` |
| nRF52840 development kit | `d` | `nrf52840_dk` | `thumbv7em-none-eabihf` | `memory-nrf52840-dk.x` |
| nRF52840 dongle, S140 SoftDevice | `s` | `nrf52840_dongle_sd` | `thumbv7em-none-eabihf` | `memory-nrf52840-dongle-sd.x` |

The `stm32f103`, `stm32f401`, `stm32f411`, and `nrf52840` features are internal
chip/family selectors enabled by board features; they are not complete board
configurations.
Other target triples commented out in `.cargo/config.toml` are template examples,
not implemented board support.

**Default mismatch:** `Cargo.toml` defaults to `nrf52840_dongle_sd`, while
`.cargo/config.toml` defaults to `thumbv7m-none-eabi` (Cortex-M3). Use the helper
or explicitly pass both the board feature and target rather than relying on
plain `cargo build`.

## Setup and build

Install Rust through rustup and Bash. `rust-toolchain.toml` selects an unpinned
nightly toolchain. Install the compilation targets and development components:

```sh
rustup toolchain install nightly
rustup target add --toolchain nightly thumbv7m-none-eabi thumbv7em-none-eabihf
rustup component add --toolchain nightly rustfmt clippy
```

Build any supported configuration with `bash build <selector> b`:

```sh
bash build b b     # Blue Pill
bash build k b     # Black Pill F411CE
bash build k401 b  # Black Pill F401CC
bash build n b     # Nucleo F411
bash build r b     # nRF52840 dongle
bash build d b     # nRF52840 DK
bash build s b     # nRF52840 dongle with S140
```

The helper always uses release mode and disables default features. An equivalent
explicit command is:

```sh
cargo build --release --no-default-features \
  --features nrf52840_dongle_sd --target thumbv7em-none-eabihf
```

The ELF executable is `target/<rust-target>/release/cortex-m-test2`. Release
builds optimize for size with LTO and retain debug symbols.

## Dependencies

The application uses `embedded-hal` 1.0 (`DelayNs` for delays). The current
resolved direct dependencies are:

| Crate | Version |
| --- | --- |
| `cortex-m` | 0.7.10 |
| `cortex-m-rt` | 0.7.7 |
| `embedded-hal` | 1.0.0 |
| `panic-halt` | 1.0.0 |
| `stm32f1xx-hal` | 0.11.0 |
| `stm32f4xx-hal` | 0.23.0 |
| `nrf52840-hal` | 0.19.0 |

`Cargo.lock` records transitive versions. `cortex-m` and the STM32 HALs still
require `embedded-hal` 0.2 alongside 1.0; application code uses 1.0.
To refresh compatible transitive releases, run `cargo update`. Check newer direct
releases with `cargo outdated --root-deps-only` (requires `cargo-outdated`), then
update manifest requirements and rebuild/lint all seven board configurations.

## Flashing and running

This firmware runs on a physical board. No Cargo runner is enabled; `cargo run`
does not provide a configured flashing or debugging workflow.

| Operation | Command pattern | Requirements and behavior |
| --- | --- | --- |
| Debug-probe flash | `bash build <selector> f` | Uses `cargo-flash` over SWD at 100 kHz by default, with the board's chip identifier. |
| STM32 USB DFU | `bash build <selector> d` | Uses `cargo objcopy` (from `cargo-binutils`, with `llvm-tools-preview`) and `dfu-util`; writes `target/out.bin` and prompts for DFU mode. |
| Nordic DFU | `bash build <selector> n` | Uses an installed `nrfdfu` command and the existing ELF; build the same selector first. |

For example, `bash build d f` flashes the development kit through its probe.
USB DFU requires a compatible bootloader and device permissions. For the
F401CC Black Pill, use `bash build k401 f` for probe flashing, or
`bash build k401 d` for USB DFU at `0x08000000`.

The helper explicitly passes `--protocol swd` and defaults to `--speed 100`
(kHz). Black Magic probes can reject connections when the protocol is unspecified;
the F411-based BMP also required a low speed in the tested setup. Include these
flags when invoking `cargo flash` directly, and close any GDB connection to the
probe first. The probe may round the requested speed to an available clock.

Override the requested speed through `PROBE_RS_SPEED`, in kHz:

```sh
./build k401 f                        # Request 100 kHz
PROBE_RS_SPEED=1000 ./build k401 f     # Request 1 MHz instead
```

F401CC hardware flashing succeeded with an F411-based BMP running
`v2.1.0-41-g3424d0a0` and `cargo-flash` 0.32.0 at the 100 kHz requested setting.
Native BMP/GDB flash verification and execution also passed with that firmware
and `v1.10.2-dirty`.

### Flash layout limitations

The following are the application regions defined by the checked-in linker
scripts, not the total physical capacities. Sizes are in KiB.

| Configuration | Flash origin / size | RAM origin / size |
| --- | --- | --- |
| Blue Pill | `0x08000000` / 56 | `0x20000000` / 20 |
| Black Pill F401CC | `0x08000000` / 256 | `0x20000000` / 64 |
| Black Pill F411CE / Nucleo F411 | `0x08000000` / 512 | `0x20000000` / 128 |
| nRF52840 dongle | `0x00001000` / 768 | `0x20000000` / 256 |
| nRF52840 DK | `0x00000000` / 768 | `0x20000000` / 256 |
| nRF52840 dongle with S140 | `0x00027000` / 740 | `0x2000B000` / 212 |

The S140 layout reserves 156 KiB of flash for the SoftDevice, 128 KiB for DFU,
and 44 KiB of RAM. This repository supplies an application memory map; it does
not install or initialize the SoftDevice or implement Bluetooth functionality.

The Blue Pill DFU operation writes at `0x08002000`, while its linker script
starts at `0x08000000`. Reconcile the linker origin and bootloader requirements
before using that operation. The helper's nRF DFU rejection check compares
against `nrf52840`, which is not a selected board feature; it does not prevent
using the STM32 DFU operation with Nordic selectors. Use the appropriate Nordic
DFU workflow instead.

## Hardware behavior

| Board | Primary LED | Secondary LEDs | Clock setup / outputs |
| --- | --- | --- | --- |
| Blue Pill | PC13, active low | None | 8 MHz HSE → 72 MHz SYSCLK; MCO on PA8 |
| Black Pill F401CC | PC13, active low | None | 25 MHz HSE → 84 MHz SYSCLK; MCO1 on PA8, MCO2 on PC9 |
| Black Pill F411CE | PC13, active low | None | 25 MHz HSE → 100 MHz SYSCLK; MCO1 on PA8, MCO2 on PC9 |
| Nucleo F411 | PA5, active high | None | Internal HSI → 100 MHz SYSCLK; MCO1 on PA8, MCO2 on PC9 |
| nRF52840 dongle (both layouts) | P0.06, active low | P0.08, P1.09, P0.12, active low | SysTick delay |
| nRF52840 DK | P0.13, active low | P0.14, P0.15, P0.16, active low | SysTick delay |

The STM32F401 and STM32F411 configurations divide both clock outputs by five:
MCO1 uses HSI (3.2 MHz output) and MCO2 uses SYSCLK (16.8 MHz on F401, 20 MHz
on F411). The F401CC configuration assumes a 25 MHz crystal and active-low PC13
LED, matching the [WeAct Black Pill board documentation](https://github.com/WeActStudio/WeActStudio.MiniSTM32F4x1).
Its 84 MHz clock limit, 256 KiB flash, and 64 KiB RAM follow the
[STM32F401xB/xC datasheet](https://www.st.com/resource/en/datasheet/stm32f401cc.pdf).

## Development and validation

```sh
cargo fmt --all -- --check
bash build b c     # Clippy for Blue Pill
bash build k401 c  # Clippy for the F401CC Black Pill
bash build s c     # Clippy for the S140 dongle
```

Build and lint every affected board configuration. There is no automated test
suite or coverage requirement, and the firmware binary disables the Cargo test
harness. Verify LED timing, polarity, secondary LED sequencing, and any changed
clock outputs on hardware; report which boards were actually tested.

## Repository layout

- `src/main.rs`: entry point, feature-gated initialization, GPIO abstraction,
  and LED sequencing.
- `Cargo.toml` / `Cargo.lock`: board features and dependency versions.
- `build`: build, lint, and flashing helper.
- `build.rs`: copies the selected `memory-*.x` into the build output as `memory.x`.
- `.cargo/config.toml`: target defaults and linker flags.
- `dev.txt`: device/core/flash/RAM reference data.
- [AGENTS.md](AGENTS.md): contributor guidelines; [CLAUDE.md](CLAUDE.md) delegates
  to the same guidance.

Keep all Markdown documentation aligned when changing supported boards,
features, commands, memory layouts, or behavior.

## License

[MIT](LICENSE), copyright Sami J. Mäkinen.
