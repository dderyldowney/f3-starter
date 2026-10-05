# f3-starter

A bare-metal Rust starter template and reference firmware for the **STM32F3DISCOVERY** development board (STM32F303VCT6). Configured for Rust Edition 2024, running `probe-rs` over onboard ST-LINK with zero-overhead `defmt` RTT logging.

---

## Toolchain Prerequisites

* **Rust Toolchain:** 1.85.0 or newer (required for Edition 2024)
* **Target Architecture:** `thumbv7em-none-eabihf` (ARM Cortex-M4F with hardware floating-point)
* **Debug Runner:** `probe-rs-tools`

### Setup

```bash
# Install bare-metal ARM target
rustup target add thumbv7em-none-eabihf

# Install probe-rs runner and flashing utility
cargo install probe-rs-tools --locked

# (Optional) Binary footprint inspection tools
cargo install cargo-binutils
rustup component add llvm-tools
```

---

## Branch Architecture

| Branch | Purpose | State |
|---|---|---|
| **`main`** | Production reference baseline | Complete modular firmware: SysTick runtime, Port E LED ring, and LSM303 accelerometer spirit level. |
| **`develop`** | Active feature development | Working branch for driver additions, calibration, and peripheral experiments. |
| **`book_project`** | Clean learning template | Minimal `#![no_std]` harness with logging and runtime entry point. Intended for chapter-by-chapter exercises from the *Embedded Rust Book*. |

---

## Hardware Pinout & Peripherals

### 1. Radial LED Compass Ring (GPIO Port E)

Eight circular user LEDs driven in push-pull output mode:

| Pin | Designator | Direction / Color | Active State |
|---|---|---|---|
| `PE9` | LD3 | North (Red) | High |
| `PE10` | LD5 | North-East (Orange) | High |
| `PE11` | LD7 | East (Green) | High |
| `PE12` | LD9 | South-East (Blue) | High |
| `PE13` | LD10 | South (Red) | High |
| `PE14` | LD8 | South-West (Orange) | High |
| `PE15` | LD6 | West (Green) | High |
| `PE8` | LD4 | North-West (Blue) | High |

### 2. Onboard Sensors (I2C1 on GPIO Port B)

Hardwired to the onboard ST LSM303DLHC/AGR 9-DoF motion sensor with onboard 4.7 kΩ pull-ups:

| Pin | Function | Mode | Target Hardware |
|---|---|---|---|
| `PB6` | `I2C1_SCL` | Alternate Function 4 (Open-Drain) | Clock line |
| `PB7` | `I2C1_SDA` | Alternate Function 4 (Open-Drain) | Data line |
| — | Accelerometer | Address: `0x19` | 3-axis acceleration |
| — | Magnetometer | Address: `0x1E` | 3-axis magnetic heading |

### 3. User Controls (GPIO Port A)

| Pin | Hardware | Configuration | Behavior |
|---|---|---|---|
| `PA0` | Blue User Button (B1) | Floating Input (pulled low externally) | Reads `1` when pressed |

---

## Architecture (`main` / `develop`)

The firmware avoids monolith files by isolating hardware domains into clean submodules:

* **`src/time.rs`**: Configures the 1 kHz SysTick hardware interrupt, maintains an atomic monotonic millisecond counter, exposes `wfi`-based low-power sleep delays, and registers the `defmt::timestamp!` hook.
* **`src/leds.rs`**: Encapsulates Port E push-pull outputs into a strongly typed `Leds` abstraction driven by an exhaustive `enum Direction`.
* **`src/sensors.rs`**: Generic `embedded-hal` LSM303 driver that configures 100 Hz output data rate, issues 6-byte auto-incremented burst reads, and runs integer-ratio fixed-point tilt classification.
* **`src/main.rs`**: Top-level 50 Hz scheduling loop coordinating sensors, orientation math, and LED output.

---

## Build, Flash, and Inspect

Connect the STM32F3DISCOVERY board to your PC using a Mini-USB cable plugged into the top ST-LINK port (**`CN1`**).

### Build & Flash

```bash
# Build and flash to target via probe-rs, then attach RTT console
cargo run
```

### Inspect Binary Footprint

```bash
cargo size -- -A
```

*Typical profile on `main`: ~16.5 KB Flash (6.4% of 256 KB) and ~1.1 KB SRAM (2.7% of 40 KB).*
