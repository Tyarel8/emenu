// #![allow(unused)]

// hide console window on Windows in release
// #![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use anyhow::{anyhow, ensure};
use std::{io::stdin, process::exit, sync::Arc, thread, time::Duration};

use clap::Parser;
use eframe::{
    egui::{self, EventFilter, FontData, Key, Modifiers, Sense, Separator},
    epaint::{Color32, FontId},
    HardwareAcceleration,
};
use font_kit::{family_name::FamilyName, source::SystemSource};
use nucleo::{pattern::Normalization, Nucleo};

mod cli;
mod layout;

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
            stdin().lines().map_while(Result::ok).for_each(|s| {
                inj.push(s.clone(), |c| {
                    c[0] = s.into();
                });
            })
        }
    });

    let window_height = cli.window_height;
    let window_width = cli.window_width;
    // let window_height = 510.0;
    // let window_width = 480.0;

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

    eframe::run_native(
        "emenu",
        options,
        Box::new(|cc| {
            let ctx = &cc.egui_ctx;

            if let Some(font_family) = cli.font.clone() {
                let font_data = get_font_data(&font_family)
                    .map_err(|e| {
                        eprintln!("Error loading the font `{font_family}`: {e}");
                        exit(1);
                    })
                    .unwrap();
                let mut fonts = egui::FontDefinitions::default();
                fonts.font_data.insert(font_family.clone(), font_data);

                fonts
                    .families
                    .entry(egui::FontFamily::Monospace)
                    .or_default()
                    .insert(0, font_family.clone());

                ctx.set_fonts(fonts);
            }

            // ctx.set_pixels_per_point(1.225);
            // ctx.set_fonts(FontDefinitions::)
            let font = egui::FontId {
                // size: 13.0,
                size: cli.font_size,
                family: egui::FontFamily::Monospace,
            };
            ctx.set_style(egui::style::Style {
                override_font_id: Some(font.clone()),
                ..Default::default()
            });

            Ok(Box::new(Emenu::new(nucleo, cli, font)))
        }),
    )
}

struct Emenu {
    input: String,
    nucleo: Nucleo<String>,
    prompt: String,
    marker: String,
    pointer: String,
    cycle: bool,
    exit_lost_focus: bool,
    has_focus: bool,
    selected_idx: u32,
    first_idx: u32,
    output_number: usize,
    multi_output: Vec<String>,
    font_id: FontId,
}

impl Emenu {
    fn new(nucleo: Nucleo<String>, cli: cli::Cli, font_id: FontId) -> Self {
        Self {
            nucleo,
            prompt: cli.prompt,
            marker: cli.marker,
            pointer: cli.pointer,
            cycle: cli.cycle,
            exit_lost_focus: cli.exit_lost_focus,
            has_focus: false,
            input: String::new(),
            selected_idx: 0,
            first_idx: 0,
            output_number: 1,
            // output_number: cli.multi.unwrap_or(1),
            multi_output: vec![],
            font_id,
        }
    }
}

impl eframe::App for Emenu {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let snapshot_changed = self.nucleo.tick(10).changed;

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
                            Some(ui.add_sized(
                                // Use available height to center label
                                [ui.available_height(), 0.0],
                                egui::Label::new(&self.prompt),
                            ))
                        } else {
                            None
                        };

                        let mut edit = ui.add_sized(
                            ui.available_size(),
                            egui::TextEdit::singleline(&mut self.input)
                                .vertical_align(egui::Align::Center),
                        );

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
                                    escape: true,
                                    horizontal_arrows: true,
                                    vertical_arrows: true,
                                },
                            )
                        });

                        if edit.changed() {
                            self.nucleo.pattern.reparse(
                                0,
                                &self.input,
                                nucleo::pattern::CaseMatching::Smart,
                                Normalization::Smart,
                                false,
                            );

                            // Clear the first_idx and selected_idx on new input
                            self.first_idx = 0;
                            self.selected_idx = 0;
                        }
                    });

                    ui.add_space(8.0);

                    let snap = self.nucleo.snapshot();
                    let total_count = snap.item_count();
                    let matched_count = snap.matched_item_count();

                    // dbg!(snap.pattern());

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
                        ui.add(
                            Separator::default()
                                .horizontal()
                                .spacing(count_label.rect.height()),
                        );
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
                                self.pointer.clone()
                            } else {
                                " ".repeat(self.pointer.chars().count())
                            };

                            // TODO: marker
                            let marker = " ";

                            // let pointer_len = self.pointer.chars().count();
                            // let marker_len = self.marker.chars().count();

                            // TODO: get the correct amount of characters that fit
                            // let max_chars = get_max_chars_in_ui(ui, char_width, inner_margin)
                            //     - (marker_len + pointer_len);
                            let max_chars = get_max_chars_in_ui(ui, char_width, inner_margin);

                            // let ellipsis = if match_string.chars().count() > max_chars {
                            //     '…'.to_string()
                            // } else {
                            //     "".to_string()
                            // };

                            let layout = layout::create_layout(
                                &self.input,
                                match_string,
                                &pointer,
                                marker,
                                max_chars,
                                self.font_id.clone(),
                            );

                            let entry = ui.add(
                                egui::Label::new(
                                    // format!(
                                    //     "{pointer}{marker}{}{ellipsis}",
                                    //     match_string.char_range(0..max_chars),
                                    // ),
                                    layout,
                                )
                                .sense(Sense::click())
                                .wrap_mode(egui::TextWrapMode::Truncate),
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
                            (i.modifiers.matches_exact(Modifiers::CTRL) && i.key_pressed(Key::N))
                                || i.key_pressed(Key::ArrowDown)
                        })) || (ui.ui_contains_pointer()
                            && ctx.input(|i| i.raw_scroll_delta.y < 0.0))
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
                            (i.modifiers.matches_exact(Modifiers::CTRL) && i.key_pressed(Key::P))
                                || i.key_pressed(Key::ArrowUp)
                        })) || (ui.ui_contains_pointer()
                            && ctx.input(|i| i.raw_scroll_delta.y > 0.0)))
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
        if ctx.input(|i| i.modifiers.matches_exact(Modifiers::CTRL) && i.key_pressed(Key::C)) {
            exit(0)
        }

        if ctx.input(|i| i.key_pressed(Key::Escape)) {
            exit(0)
        }

        // Exit when focus is lost if activated
        match ctx.input(|i| i.focused) {
            true => self.has_focus = true,
            false => {
                if self.exit_lost_focus && self.has_focus {
                    exit(0)
                }
            }
        }
    }
}

fn get_font_data(font_name: &str) -> anyhow::Result<FontData> {
    let font = SystemSource::new()
        .select_best_match(
            &[FamilyName::Title(font_name.to_string())],
            &font_kit::properties::Properties::default(),
        )?
        .load()?;

    ensure!(font.is_monospace(), "Font is not monospaced");

    let font_data = font.copy_font_data().ok_or(anyhow!("No font data"))?;

    Ok(FontData::from_owned((*font_data).clone()))
}

fn get_max_chars_in_ui(ui: &mut egui::Ui, char_width: f32, inner_margin: f32) -> usize {
    // let char_width = ui.fonts(|f| f.glyph_width(font_id, ' '));
    ((ui.max_rect().width() - inner_margin * 2.0) / char_width).round() as usize
}
