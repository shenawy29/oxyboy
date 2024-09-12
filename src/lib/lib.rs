mod cartridge;
mod cpu;
mod joypad;
mod mmu;
mod ppu;
mod registers;
mod timer;
mod ui;

use crate::cpu::Cpu;
use eframe::egui::{Vec2, ViewportBuilder};
use mmu::Mmu;
use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, SyncSender};
use ui::Ui;

enum Press {
    Down(u8),
    Up(u8),
}

pub struct Emulator {
    pub(crate) paused: bool,
}

impl Emulator {
    fn new() -> Self {
        Self { paused: false }
    }

    pub(crate) fn run_cpu(
        &mut self,
        cpu: &mut Cpu,
        sender: SyncSender<Vec<u8>>,
        receiver: Receiver<Press>,
        fnreceiver: Receiver<PathBuf>,
    ) {
        let file = loop {
            if let Ok(file) = fnreceiver.try_recv() {
                break file;
            };
        };

        cpu.mmu = Mmu::from(file);

        loop {
            if let Ok(file) = fnreceiver.try_recv() {
                cpu.reset();
                cpu.mmu = Mmu::from(file);
            }

            if self.paused {
                continue;
            }

            cpu.docycle();

            if cpu.mmu.ppu.updated {
                let data = cpu.get_gpu_data().to_vec();
                sender.send(data).unwrap();
                cpu.mmu.ppu.updated = false;
            }

            if let Ok(press) = receiver.try_recv() {
                match press {
                    Press::Down(key) => cpu.mmu.joypad.keydown(key),
                    Press::Up(key) => cpu.mmu.joypad.keyup(key),
                }
            }
        }
    }

    pub fn start() {
        let mut emu = Emulator::new();

        let (graphics_tx, graphics_rx) = mpsc::sync_channel(1);
        let (joypad_tx, joypad_rx) = mpsc::channel();
        let (filename_tx, filename_rx) = mpsc::channel::<PathBuf>();

        let native_options = eframe::NativeOptions {
            viewport: ViewportBuilder::default().with_inner_size(Vec2::new(160.0, 144.0)),
            ..Default::default()
        };

        let mut cpu = Cpu::new();
        std::thread::spawn(move || emu.run_cpu(&mut cpu, graphics_tx, joypad_rx, filename_rx));

        eframe::run_native(
            "Oxyboy",
            native_options,
            Box::new(|_cc| Ok(Box::new(Ui::new(graphics_rx, joypad_tx, filename_tx)))),
        )
        .unwrap();
    }
}
