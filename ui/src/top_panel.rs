use std::time::Instant;

use egui::{vec2, Align2, Button, Frame, Label, RichText, Sense, TextStyle, Ui, Visuals, Window};

use crate::util::{AVATAR_EXTENSIONS, VIDEO_EXTENSIONS};

use super::WalksnailOsdTool;

impl WalksnailOsdTool {
    pub fn render_top_panel(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.add_space(5.0);
            ui.horizontal(|ui| {
                self.import_files(ui, ctx);
                self.reset_files(ui);
                ui.add_space(ui.available_width() - 55.0);
                self.toggle_light_dark_theme(ui, ctx);
                self.about_window(ui, ctx);
            });
            ui.add_space(3.0);
        });
    }

    fn import_files(&mut self, ui: &mut Ui, ctx: &egui::Context) {
        if ui
            .add_enabled(self.render_status.is_not_in_progress(), Button::new("Open files"))
            .clicked()
        {
            if let Some(file_handles) = rfd::FileDialog::new()
                .add_filter("Avatar files", AVATAR_EXTENSIONS)
                .add_filter("Video files", VIDEO_EXTENSIONS)
                .pick_files()
            {
                tracing::info!("Opened files {:?}", file_handles);
                self.import_video_file(&file_handles);
                self.import_osd_file(&file_handles);
                self.import_font_file(&file_handles);
                self.import_srt_file(&file_handles);

                self.update_osd_preview(ctx);
                self.render_status.reset();
            }
        }

        // Collect dropped files
        let file_handles = ctx.input(|i| {
            i.raw
                .dropped_files
                .iter()
                .flat_map(|f| f.path.clone())
                .collect::<Vec<_>>()
        });
        if !file_handles.is_empty() {
            tracing::info!("Dropped files {:?}", file_handles);
            self.import_video_file(&file_handles);
            self.import_osd_file(&file_handles);
            self.import_font_file(&file_handles);
            self.import_srt_file(&file_handles);
            self.update_osd_preview(ctx);
            self.render_status.reset();
        }
    }

    fn reset_files(&mut self, ui: &mut Ui) {
        if ui
            .add_enabled(self.render_status.is_not_in_progress(), Button::new("Reset files"))
            .clicked()
        {
            // Note: don't do `self.font_file = None` here, that just makes UX bad.
            self.input_video_file = None;
            self.video_info = None;
            self.osd_file = None;
            self.srt_file = None;
            self.osd_preview.texture_handle = None;
            self.osd_preview.preview_frame = 1;
            self.render_status.reset();
            self.update_output_video_path(); // this will reset it to empty since input_video_file is none.
            tracing::info!("Reset files");
        }
    }

    fn toggle_light_dark_theme(&mut self, ui: &mut Ui, ctx: &egui::Context) {
        let icon = if self.dark_mode { "☀" } else { "🌙" };
        if ui.add(Button::new(icon).frame(false)).clicked() {
            let mut visuals = if self.dark_mode {
                Visuals::light()
            } else {
                Visuals::dark()
            };
            visuals.indent_has_left_vline = false;
            ctx.set_visuals(visuals);
            self.dark_mode = !self.dark_mode;
            self.config_changed = Some(Instant::now());
        }
    }

    fn about_window(&mut self, ui: &mut Ui, ctx: &egui::Context) {
        if ui.add(Button::new(RichText::new("ℹ")).frame(false)).clicked() {
            self.about_window_open = !self.about_window_open;
        }

        let frame = Frame::window(&ctx.style());
        if self.about_window_open {
            Window::new("About")
                .anchor(Align2::CENTER_CENTER, vec2(0.0, 0.0))
                .frame(frame)
                .open(&mut self.about_window_open)
                .auto_sized()
                .collapsible(false)
                .show(ctx, |ui| {
                    ui.add_space(10.0);

                    ui.horizontal(|ui| {
                        ui.style_mut().override_text_style = Some(TextStyle::Button);
                        ui.style_mut().text_styles.get_mut(&TextStyle::Button).unwrap().size = 20.0;

                        ui.hyperlink_to(
                            "This project is a fork",
                            "https://github.com/vmzhivetyev/walksnail-osd-tool",
                        );
                        ui.label("of");
                        ui.hyperlink_to("walksnail-osd-tool", "https://github.com/avsaase/walksnail-osd-tool");
                    });

                    ui.add_space(16.0);

                    egui::Grid::new("about").spacing(vec2(10.0, 5.0)).show(ui, |ui| {
                        ui.label("Original Author:");
                        ui.label("Alexander van Saase");
                        ui.end_row();

                        ui.label("Version:");
                        let version = &self.app_version;
                        if ui
                            .add(Label::new(version).sense(Sense::click()))
                            .on_hover_text_at_pointer("Double-click to copy to clipboard")
                            .double_clicked()
                        {
                            ui.output_mut(|o| o.copied_text.clone_from(version));
                        }
                        ui.end_row();

                        ui.label("Target:");
                        ui.label(&self.target);
                        ui.end_row();

                        ui.label("License:");
                        ui.hyperlink_to(
                            "General Public License v3.0",
                            "https://github.com/vmzhivetyev/walksnail-osd-tool/blob/master/LICENSE.md",
                        );
                        ui.end_row();
                    });

                    ui.add_space(10.0);

                    ui.hyperlink_to("Original Author’s Support Link", "https://www.buymeacoffee.com/avsaase");
                });
        }
    }
}
