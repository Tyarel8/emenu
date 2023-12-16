#![allow(unused)]

// hide console window on Windows in release
// #![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{
    io::stdin,
    process::exit,
    sync::{mpsc, Arc},
};

use clap::Parser;
use eframe::{
    egui::{
        self, FontDefinitions, Key, Modifiers, MouseWheelUnit, Sense, Separator, Style, TextBuffer,
    },
    epaint::{Color32, FontId},
};
use nucleo::Nucleo;

mod cli;

// TODO: proper theme, config, multimode, highlight searched in matches
fn main() -> Result<(), eframe::Error> {
    let cli = cli::Cli::parse();

    // read the haystack from stdin
    let mut haystack: Vec<_> = if atty::isnt(atty::Stream::Stdin) {
        stdin().lines().flatten().collect()
    } else {
        vec![]
    };

    if cli.exit_if_empty && haystack.is_empty() {
        exit(0)
    }

    if cli.tac {
        haystack.reverse()
    }

    // let (tx, rx) = mpsc::channel();
    let nucleo = Nucleo::new(
        nucleo::Config::DEFAULT,
        Arc::new(move || {
            // tx.send(true).unwrap();
        }),
        None,
        1,
    );

    let inj = nucleo.injector();
    for s in haystack {
        inj.push(s.clone(), |c| {
            c[0] = s.into();
        });
    }

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_decorations(false)
            .with_resizable(false)
            .with_inner_size([480.0, 510.0]),
        centered: true,
        ..Default::default()
    };

    // let mul = cli.multi.clone();
    eframe::run_native(
        "emenu",
        options,
        Box::new(|cc| {
            let ctx = &cc.egui_ctx;
            ctx.set_pixels_per_point(1.25);
            // ctx.set_fonts(FontDefinitions::)
            let font = egui::FontId {
                size: 13.0,
                family: egui::FontFamily::Monospace,
            };
            ctx.set_style(egui::style::Style {
                override_font_id: Some(font.clone()),
                ..Default::default()
            });

            Box::new(Emenu::new(nucleo, cli, font))
        }),
    )
}

struct Emenu {
    input: String,
    nucleo: Nucleo<String>,
    prompt: String,
    marker: String,
    pointer: String,
    selected_idx: u32,
    first_idx: u32,
    output_number: usize, // rx: mpsc::Receiver<bool>,
    font_id: FontId,
}

impl Emenu {
    fn new(nucleo: Nucleo<String>, cli: cli::Cli, font_id: FontId) -> Self {
        Self {
            nucleo,
            prompt: cli.prompt,
            marker: cli.marker,
            pointer: cli.pointer,
            input: String::new(),
            selected_idx: 0,
            first_idx: 0,
            output_number: cli.multi.unwrap_or(1),
            font_id,
        }
    }
}

impl eframe::App for Emenu {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // if let Ok(true) = self.rx.try_recv() {
        self.nucleo.tick(10);
        // }

        self.keyboard_events_exit(ctx, _frame);

        egui::CentralPanel::default()
            .frame(
                egui::Frame::default()
                    .stroke((1.0, Color32::GRAY))
                    .inner_margin(8.0)
                    .outer_margin(4.0),
            )
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        let prompt = if !self.prompt.is_empty() {
                            Some(ui.label(&self.prompt))
                        } else {
                            None
                        };

                        let mut edit = ui.add_sized(ui.available_size(), {
                            egui::TextEdit::singleline(&mut self.input)
                        });

                        if let Some(prompt) = prompt {
                            edit = edit.labelled_by(prompt.id);
                        }

                        edit.request_focus();

                        if edit.changed() {
                            self.nucleo.pattern.reparse(
                                0,
                                &self.input,
                                nucleo::pattern::CaseMatching::Smart,
                                false,
                            );

                            // Clear the first_idx and selected_idx on new input
                            self.first_idx = 0;
                            self.selected_idx = 0;
                        }
                    });

                    ui.add_space(4.0);

                    let snap = self.nucleo.snapshot();
                    let total_count = snap.item_count();
                    let matched_count = snap.matched_item_count();

                    ui.horizontal(|ui| {
                        ui.label(format!("{matched_count}/{total_count}"));
                        ui.add(Separator::default().horizontal());
                    });

                    ui.add_space(4.0);

                    let mut view_rows = 0;

                    // dbg!(ui.min_rect());
                    ui.vertical(|ui| {
                        for (i, matched) in snap
                            .matched_items(self.first_idx..snap.matched_item_count())
                            .enumerate()
                        {
                            if ui.available_height() < 10.0 {
                                break;
                            }

                            view_rows += 1;

                            let char_size = ui.fonts(|f| f.glyph_width(&self.font_id, ' '));
                            let max_chars =
                                (ui.max_rect().width() / char_size).trunc() as usize - 2;

                            let match_string = matched.data;

                            let dots = if match_string.chars().count() > max_chars {
                                "…"
                            } else {
                                ""
                            };

                            let entry = ui.add(
                                egui::Label::new(format!(
                                    "{:<2}{}{dots}",
                                    if i == self.selected_idx as usize {
                                        &self.pointer
                                    } else {
                                        ""
                                    },
                                    match_string.char_range(0..max_chars),
                                ))
                                .sense(Sense::click())
                                .wrap(false),
                            );

                            if entry.clicked() {
                                self.selected_idx = i as u32;
                            }

                            if entry.double_clicked() {
                                todo!()
                            }
                        }
                    });

                    // Move current pointer with ctrl + p/n or mouse wheel
                    if (ctx
                        .input(|i| i.modifiers.matches(Modifiers::CTRL) && i.key_pressed(Key::N)))
                        || (ui.ui_contains_pointer() && ctx.input(|i| i.scroll_delta.y < 0.0))
                    {
                        if self.selected_idx > (view_rows - 2) {
                            self.first_idx += 1;
                        } else {
                            self.selected_idx = self.selected_idx.saturating_add(1);
                        }
                    }

                    if (ctx
                        .input(|i| i.modifiers.matches(Modifiers::CTRL) && i.key_pressed(Key::P)))
                        || (ui.ui_contains_pointer() && ctx.input(|i| i.scroll_delta.y > 0.0))
                    {
                        if self.first_idx != 0 && self.selected_idx == 0 {
                            self.first_idx -= 1;
                        }
                        self.selected_idx = self.selected_idx.saturating_sub(1);
                    }

                    // Prevent the selected_idx from overflowing
                    self.selected_idx = self.selected_idx.min(view_rows.saturating_sub(1));

                    // Handle enter
                    if ctx.input(|i| i.key_pressed(Key::Enter)) {
                        if let Some(item) = snap.get_matched_item(self.selected_idx as u32) {
                            print!("{}", item.data)
                        }
                        exit(0);
                    }
                })
            });
    }
}

impl Emenu {
    fn keyboard_events_exit(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // exit on ctrl+c or esc
        if ctx.input(|i| i.modifiers.matches(Modifiers::CTRL) && i.key_pressed(Key::C)) {
            exit(0)
        }

        if ctx.input(|i| i.key_pressed(Key::Escape)) {
            exit(0)
        }

        // Print result and exit on enter
        // if ctx.input(|i| i.key_pressed(Key::Enter)) {
        //     // exit(0)
        //     dbg!("Enter pressed");
        // }
    }
}
