# Oxyboy

Oxyboy is a partially-implemented Gameboy emulator written in Rust, using WebGPU and Egui/Eframe for rendering and window handling. This is a work-in-progress project. While many games run, not all of them do, due to the lack of MBC1 emulation.

To find games, go to [emulatorgames.net](https://emulatorgames.net). [Here](https://www.emulatorgames.net/roms/gameboy-color/tetris/) is link for Tetris.

![image](https://github.com/user-attachments/assets/7d160421-52b3-48a7-b956-3585b6a280af)

## Features

Full CPU instruction set emulation (including fetch-decode-execute loop).

Platform-independent rendering using Wgpu.

## Installation

To build and run Oxyboy locally, you'll need the Rust toolchain.

## Build Instructions

> [!NOTE]
> For Nix users, there is a flake ready for you.

#### 1. Clone the repository:

```bash
git clone https://github.com/shenawy29/oxyboy.git
cd oxyboy
```

#### 2. Install dependencies:

```bash
cargo build --release
```

#### 3. Run the binary:

```bash
cargo run --release
```

## Keybindings

| Key on Keyboard    | Emulator Key       |
| ------------------ | ------------------ |
| Z                  | A                  |
| X                  | B                  |
| Up/Down/Left/Right | Up/Down/Left/Right |
| K/J/H/L            | Up/Down/Left/Right |
| Space              | Select             |
| Return/Enter       | Start              |

## Roadmap

- Sound emulation
- Save games
- WASM release

## Credits

- [The Ultimate Gameboy Talk](https://youtu.be/HyzD8pNlpwI)
- [The EmuDev Discord](https://discord.gg/7nuaqZ2)
- [SM83_decoding.pdf](https://cdn.discordapp.com/attachments/465586075830845475/742438340078469150/SM83_decoding.pdf)
- https://github.com/SingleStepTests/sm83
- https://github.com/mvdnes/rboy
- https://gbdev.io/pandocs/
