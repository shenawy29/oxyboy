use eframe::egui::{self, Image, TextureFilter};
use std::{
    path::PathBuf,
    sync::mpsc::{Receiver, Sender},
};

use eframe::egui::ColorImage;

use crate::{
    ppu::{SCREEN_H, SCREEN_W},
    Press,
};

pub(crate) struct Ui {
    pub(crate) rx: Receiver<Vec<u8>>,
    pub(crate) tx: Sender<Press>,
    pub(crate) fn_tx: Sender<PathBuf>,
}

impl Ui {
    pub(crate) fn new(rx: Receiver<Vec<u8>>, tx: Sender<Press>, fn_tx: Sender<PathBuf>) -> Self {
        Self { rx, tx, fn_tx }
    }
}

impl eframe::App for Ui {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top")
            .show_separator_line(true)
            .show(ctx, |ui| {
                egui::menu::bar(ui, |ui| {
                    ui.menu_button("File", |ui| {
                        if ui.button("Load").clicked() {
                            let file = rfd::FileDialog::new()
                                .add_filter("rom", &["gb"])
                                .pick_file();

                            if let Some(file) = file {
                                self.fn_tx.send(file).unwrap();
                            };
                        }
                    });
                });
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            let data = self.rx.try_recv();

            if data.is_err() {
                return;
            }

            #[rustfmt::skip]
            ui.input(|i| {
                if i.key_pressed(egui::Key::ArrowDown) || i.key_pressed(egui::Key::J) { _ = self.tx.send(Press::Down(0b0000_1000)); }
                if i.key_released(egui::Key::ArrowDown) || i.key_released(egui::Key::J) { _ = self.tx.send(Press::Up(0b0000_1000)); }
                if i.key_pressed(egui::Key::ArrowUp) || i.key_pressed(egui::Key::K) { _ = self.tx.send(Press::Down(0b0000_0100)); }
                if i.key_released(egui::Key::ArrowUp) || i.key_released(egui::Key::K) { _ = self.tx.send(Press::Up(0b0000_0100)); }
                if i.key_pressed(egui::Key::ArrowLeft) || i.key_pressed(egui::Key::H) { _ = self.tx.send(Press::Down(0b0000_0010)); }
                if i.key_released(egui::Key::ArrowLeft) || i.key_released(egui::Key::H) { _ = self.tx.send(Press::Up(0b0000_0010)); }
                if i.key_pressed(egui::Key::ArrowRight) || i.key_pressed(egui::Key::L) { _ = self.tx.send(Press::Down(0b0000_0001)); }
                if i.key_released(egui::Key::ArrowRight) || i.key_released(egui::Key::L) { _ = self.tx.send(Press::Up(0b0000_0001)); }
                if i.key_pressed(egui::Key::Z) { _ = self.tx.send(Press::Down(0b0001_0000)); }
                if i.key_released(egui::Key::Z) { _ = self.tx.send(Press::Up(0b0001_0000)); }
                if i.key_pressed(egui::Key::X) { _ = self.tx.send(Press::Down(0b0010_0000)); }
                if i.key_released(egui::Key::X) { _ = self.tx.send(Press::Up(0b0010_0000)); }
                if i.key_pressed(egui::Key::Space) { _ = self.tx.send(Press::Down(0b0100_0000)); }
                if i.key_released(egui::Key::Space) { _ = self.tx.send(Press::Up(0b0100_0000)); }
                if i.key_pressed(egui::Key::Enter) { _ = self.tx.send(Press::Down(0b1000_0000)); }
                if i.key_released(egui::Key::Enter) { _ = self.tx.send(Press::Up(0b1000_0000)); }
            });

            let t = ColorImage::from_rgb([SCREEN_W, SCREEN_H], &data.unwrap());

            let s = ui
                .ctx()
                .load_texture("Screen", t, egui::TextureOptions {
                    wrap_mode: egui::TextureWrapMode::ClampToEdge,
                    magnification: TextureFilter::Nearest,
                    minification: TextureFilter::Nearest,
                });

            Image::from_texture(&s).paint_at(ui, ui.ctx().screen_rect());
        });

        ctx.request_repaint();
    }
}
