// #![allow(unused)]

// hide console window on Windows in release
// #![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{io::stdin, process::exit, sync::Arc, thread, time::Duration};

use clap::Parser;
use eframe::{
    egui::{self, EventFilter, Key, Modifiers, Sense, Separator, TextBuffer},
    epaint::{Color32, FontId},
    HardwareAcceleration,
};
use nucleo::Nucleo;

mod cli;

// TODO: proper theme, config, multimode, highlight searched in matches
fn main() -> Result<(), eframe::Error> {
    let cli = cli::Cli::parse();

    if cli.exit_if_empty && atty::is(atty::Stream::Stdin) {
        exit(0)
    }

    let nucleo = Nucleo::new(
        {
            let mut conf = nucleo::Config::DEFAULT;
            conf.ignore_case = cli.case_insensitive;
            conf.normalize = !cli.literal;
            conf
        },
        Arc::new(|| {}),
        None,
        1,
    );

    let inj = nucleo.injector();

    // Read from stdin in another thread
    // TODO: nucleo has to add support for --tac
    thread::spawn(move || {
        if atty::isnt(atty::Stream::Stdin) {
            stdin().lines().flatten().for_each(|s| {
                inj.push(s.clone(), |c| {
                    c[0] = s.into();
                });
            })
        }
    });

    let window_height = 510.0;
    let window_width = 480.0;

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_always_on_top()
            .with_decorations(false)
            .with_resizable(false)
            .with_inner_size([window_width, window_height])
            .with_max_inner_size((window_width, window_height)),
        centered: true,
        hardware_acceleration: HardwareAcceleration::Required,
        ..Default::default()
    };

    // let mul = cli.multi.clone();
    eframe::run_native(
        "emenu",
        options,
        Box::new(|cc| {
            let ctx = &cc.egui_ctx;
            ctx.set_pixels_per_point(1.225);
            // ctx.set_fonts(FontDefinitions::)
            let font = egui::FontId {
                size: 13.0,
                family: egui::FontFamily::Monospace,
                ..FontId::default()
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
    ellipsis: char,
    cycle: bool,
    selected_idx: u32,
    first_idx: u32,
    output_number: usize, // rx: mpsc::Receiver<bool>,
    multi_output: Vec<String>,
    _font_id: FontId,
}

impl Emenu {
    fn new(nucleo: Nucleo<String>, cli: cli::Cli, font_id: FontId) -> Self {
        Self {
            nucleo,
            prompt: cli.prompt,
            marker: cli.marker,
            pointer: cli.pointer,
            ellipsis: cli.ellipsis,
            cycle: cli.cycle,
            input: String::new(),
            selected_idx: 0,
            first_idx: 0,
            output_number: 1,
            // output_number: cli.multi.unwrap_or(1),
            multi_output: vec![],
            _font_id: font_id,
        }
    }
}

impl eframe::App for Emenu {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // if let Ok(true) = self.rx.try_recv() {
        let snapshot_changed = self.nucleo.tick(10).changed;
        // }

        if snapshot_changed {
            ctx.request_repaint_after(Duration::from_secs(1));
        }

        self.keyboard_events_exit(ctx, _frame);

        // ctx.fonts(|f| dbg!(f.pixels_per_point()));

        let inner_margin = 8.0;

        egui::CentralPanel::default()
            .frame(
                egui::Frame::default()
                    .stroke((1.0, Color32::GRAY))
                    .inner_margin(inner_margin)
                    .outer_margin(4.0)
                    .rounding(2.0),
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

                        // Prevent other widgets from taking focus
                        edit.request_focus();

                        ui.memory_mut(|m| {
                            m.set_focus_lock_filter(
                                edit.id,
                                EventFilter {
                                    tab: true,
                                    arrows: true,
                                    escape: true,
                                },
                            )
                        });

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

                    let count_string = format!(
                        "{matched_count}/{total_count}{}",
                        if self.output_number > 1 {
                            format!("({})", self.multi_output.len())
                        } else {
                            "".to_string()
                        }
                    );

                    let count_label = ui.horizontal(|ui| {
                        let count_label = ui.label(&count_string);
                        ui.add(Separator::default().horizontal());
                        count_label
                    });

                    ui.add_space(4.0);

                    // Get the width of a single char to truncate the matched items
                    let char_width = count_label.inner.rect.width() / (count_string.len() as f32);
                    let char_height = count_label.inner.rect.height();

                    let mut view_rows: u32 = 0;

                    ui.vertical(|ui| {
                        for (i, matched) in snap
                            .matched_items(self.first_idx..snap.matched_item_count())
                            .enumerate()
                        {
                            if ui.available_height() < char_height {
                                break;
                            }

                            view_rows += 1;

                            let match_string = matched.data;

                            let pointer = if i == self.selected_idx as usize {
                                &self.pointer
                            } else {
                                ""
                            };

                            // TODO: marker
                            let marker = "";

                            let pointer_len = self.pointer.chars().count();
                            let marker_len = self.marker.chars().count();

                            // TODO: get the correct amount of characters that fit
                            let max_chars = get_max_chars_in_ui(ui, char_width, inner_margin)
                                - (marker_len + pointer_len);

                            let ellipsis = if match_string.chars().count() > max_chars {
                                self.ellipsis.to_string()
                            } else {
                                "".to_string()
                            };

                            let entry = ui.add(
                                egui::Label::new(format!(
                                    "{pointer:>pointer_len$}{marker:>marker_len$}{}{ellipsis}",
                                    match_string.char_range(0..max_chars),
                                ))
                                .sense(Sense::click())
                                .wrap(false),
                            );

                            if entry.clicked() {
                                self.selected_idx = i as u32;
                            }

                            if entry.double_clicked() {
                                print!("{match_string}");
                                exit(0);
                            }
                        }
                    });

                    // Move current pointer with ctrl + p/n, arrows or mouse wheel
                    let tab_multi =
                        ctx.input(|i| i.key_pressed(Key::Tab)) && self.output_number > 1;
                    if view_rows > 0
                        && ((ctx.input(|i| {
                            (i.modifiers.matches(Modifiers::CTRL) && i.key_pressed(Key::N))
                                || i.key_pressed(Key::ArrowDown)
                        })) || (ui.ui_contains_pointer()
                            && ctx.input(|i| i.scroll_delta.y < 0.0))
                            || tab_multi)
                    {
                        // if tab_multi {}

                        if self.selected_idx >= (view_rows - 1)
                            && self.selected_idx < (matched_count - 1 - self.first_idx)
                        {
                            self.first_idx += 1;
                        } else if self.cycle && self.selected_idx == view_rows.saturating_sub(1) {
                            self.selected_idx = 0;
                            self.first_idx = 0;
                        } else {
                            self.selected_idx = self.selected_idx.saturating_add(1);
                        }
                    }

                    if view_rows > 0
                        && ((ctx.input(|i| {
                            (i.modifiers.matches(Modifiers::CTRL) && i.key_pressed(Key::P))
                                || i.key_pressed(Key::ArrowUp)
                        })) || (ui.ui_contains_pointer()
                            && ctx.input(|i| i.scroll_delta.y > 0.0)))
                    {
                        if self.first_idx != 0 && self.selected_idx == 0 {
                            self.first_idx -= 1;
                        }

                        if self.cycle && self.selected_idx == 0 {
                            self.selected_idx = matched_count;
                            self.first_idx = matched_count - view_rows;
                        }

                        self.selected_idx = self.selected_idx.saturating_sub(1);
                    }

                    // Prevent the selected_idx from overflowing
                    self.selected_idx = self.selected_idx.min(view_rows.saturating_sub(1));

                    // Handle enter
                    if ctx.input(|i| i.key_pressed(Key::Enter)) {
                        if let Some(item) =
                            snap.get_matched_item(self.first_idx + self.selected_idx)
                        {
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

fn get_max_chars_in_ui(ui: &mut egui::Ui, char_width: f32, inner_margin: f32) -> usize {
    // let char_width = ui.fonts(|f| f.glyph_width(font_id, ' '));
    ((ui.max_rect().width() - inner_margin * 2.0) / char_width).round() as usize
}
