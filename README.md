# Oxyboy

Oxyboy is a partially-implemented Gameboy emulator written in Rust, using WebGPU and Egui/Eframe for rendering and window handling. This is a work-in-progress project. While many games run, not all of them do, due to the lack of MBC1 emulation.


To find games, go to emulatorgames.net.

Here is the link for the tetris ROM: https://www.emulatorgames.net/roms/gameboy-color/tetris/

<p align="center">
  <img title="Oxyboy running on Linux natively." alt="Oxyboy running on Linux natively." src="https://github.com/user-attachments/assets/78d53ed7-e630-494b-b766-2ba345269d61" width="45%">
  <img title="Oxyboy running on Linux using Wine." alt="Oxyboy.exe running on Linux using Wine." src="https://github.com/user-attachments/assets/6c02417b-8fb6-4bf8-9e67-18f8d0280f2e" width="45%">
</p>

## Features

Full CPU instruction set emulation (including fetch-decode-execute loop).

Platform-independent rendering using Wgpu.

## Installation

To build and run Oxyboy locally, you'll need the Rust toolchain.

## Build Instructions

###### For Nix users, there is a flake ready for you.

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

- MBC1 emulation
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
