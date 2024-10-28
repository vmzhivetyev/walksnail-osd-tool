use std::time::Instant;

use backend::{
    ffmpeg::{Codec, Encoder},
    font::CharacterSize,
    util::Coordinates,
};
use egui::{
    vec2, Button, CentralPanel, Checkbox, CollapsingHeader, Color32, CursorIcon, Grid, Image, Rect, RichText,
    ScrollArea, Sense, Slider, Stroke, TextStyle, Ui,
};

use crate::{
    osd_preview::{calculate_horizontal_offset, calculate_vertical_offset},
    util::{separator_with_space, tooltip_text},
    WalksnailOsdTool,
};

impl WalksnailOsdTool {
    pub fn get_selected_encoder(&self) -> Option<backend::ffmpeg::Encoder> {
        self.displayed_encoders().get(self.render_settings.selected_encoder_idx).cloned()
    }

    pub fn render_central_panel(&mut self, ctx: &egui::Context) {
        CentralPanel::default().show(ctx, |ui| {
            ScrollArea::vertical().show(ui, |ui| {
                ui.style_mut().spacing.slider_width = self.ui_dimensions.osd_position_sliders_length;

                self.osd_options(ui, ctx);

                separator_with_space(ui, 10.0);

                self.srt_options(ui, ctx);

                separator_with_space(ui, 10.0);

                self.osd_preview(ui, ctx);

                separator_with_space(ui, 10.0);

                self.rendering_options(ui);
            });
        });
    }

    fn osd_options(&mut self, ui: &mut Ui, ctx: &egui::Context) {
        let mut changed = false;

        CollapsingHeader::new(RichText::new("OSD Options").heading())
            .default_open(true)
            .show_unindented(ui, |ui| {
                Grid::new("osd_options")
                    .min_col_width(self.ui_dimensions.options_column1_width)
                    .show(ui, |ui| {
                        ui.label("Horizontal position")
                            .on_hover_text(tooltip_text("Horizontal position of the flight controller OSD (pixels from the left edge of the video)."));
                        ui.horizontal(|ui| {
                            changed |= ui
                                .add(Slider::new(&mut self.osd_options.position.x, -200..=700).text("Pixels"))
                                .changed();

                            if ui.button("Center").clicked() {
                                if let (Some(video_info), Some(osd_file), Some(font_file)) =
                                    (&self.video_info, &self.osd_file, &self.font_file)
                                {
                                    self.osd_options.position.x = calculate_horizontal_offset(
                                        video_info.width,
                                        osd_file
                                            .frames
                                            .get(self.osd_preview.preview_frame as usize - 1)
                                            .unwrap(),
                                        &font_file.character_size,
                                    );
                                    changed |= true;
                                }
                            }

                            if ui.button("Reset").clicked() {
                                self.osd_options.position.x = 0;
                                changed |= true;
                            }
                        });
                        ui.end_row();

                        //

                        ui.label("Vertical position")
                            .on_hover_text(tooltip_text("Vertical position of the flight controller OSD (pixels from the top of the video).").small());
                        ui.horizontal(|ui| {
                            changed |= ui
                                .add(Slider::new(&mut self.osd_options.position.y, -200..=700).text("Pixels"))
                                .changed();

                            if ui.button("Center").clicked() {
                                if let (Some(video_info), Some(osd_file), Some(font_file)) =
                                    (&self.video_info, &self.osd_file, &self.font_file)
                                {
                                    self.osd_options.position.y = calculate_vertical_offset(
                                        video_info.height,
                                        osd_file
                                            .frames
                                            .get(self.osd_preview.preview_frame as usize - 1)
                                            .unwrap(),
                                        &font_file.character_size,
                                    );
                                    changed |= true
                                }
                            }

                            if ui.button("Reset").clicked() {
                                self.osd_options.position.y = 0;
                                changed |= true
                            }
                        });
                        ui.end_row();

                        ui.label("Mask")
                            .on_hover_text(tooltip_text("Click edit to select OSD elements on the preview that should not be rendered on the video. This can be useful to hide GPS coordinates, etc."));
                        ui.horizontal(|ui| {
                            let txt = if !self.osd_preview.mask_edit_mode_enabled || !self.all_files_loaded() {"Edit"} else {"Save"};
                            if ui.add_enabled(self.all_files_loaded(), Button::new(txt))
                                .on_disabled_hover_text(tooltip_text("First load the input files")).clicked() {
                                self.osd_preview.mask_edit_mode_enabled = !self.osd_preview.mask_edit_mode_enabled;
                            }
                            if ui.button("Reset").clicked() {
                                self.osd_options.reset_mask();
                                self.config_changed = Instant::now().into();
                                self.update_osd_preview(ctx);
                            }
                            let masked_positions = self.osd_options.masked_grid_positions.len();
                            ui.label(format!("{masked_positions} positions masked"));
                        });
                        ui.end_row();

                        ui.label("Adjust playback speed")
                            .on_hover_text(tooltip_text("Attempt to correct for wrong OSD timestamps in <=32.37.10 firmwares that causes video and OSD to get out of sync."));
                        ui.horizontal(|ui| {
                            changed |= ui
                                .add(Checkbox::without_text(&mut self.osd_options.adjust_playback_speed))
                                .changed()
                        });
                        ui.end_row();

                        ui.label("Disable OSD rendering")
                            .on_hover_text(tooltip_text("Do not render OSD."));
                        ui.horizontal(|ui| {
                            changed |= ui
                                .add(Checkbox::without_text(&mut self.osd_options.no_osd))
                                .changed()
                        });
                        ui.end_row();

                        // Enable playback offset only if playback speed adjustment is disabled, since it tries to fix basically the same problem
                        if !self.osd_options.adjust_playback_speed && self.video_info.is_some() {
                            ui.label("Adjust playback offset")
                                .on_hover_text(tooltip_text("Start render OSD from N-th second. Usefull for splitted videos with lost OSD frames."));
                            ui.horizontal(|ui| {
                                ui
                                .add(Slider::new(
                                    &mut self.osd_options.osd_playback_offset,
                                    0.0..=self.video_info.as_ref().map(|v| v.duration.as_secs_f32()).unwrap_or(0.0))
                                    .text("Seconds"))
                                    .on_hover_text("If your video file and osd file has the same duration, you probaby do not need this");

                                if ui.add(Button::new(RichText::new("Auto")).small()).clicked() {
                                    let difference = self.video_info.as_ref().unwrap().duration.as_secs_f32() - self.osd_file.as_ref().unwrap().duration.as_secs_f32();
                                    self.osd_options.osd_playback_offset = difference.abs();
                                }
                            });
                            ui.end_row();
                        }

                        if self.video_info.is_some() {
                            if self.osd_options.character_size.is_none() {
                                self.osd_options.character_size = Some(backend::overlay::get_character_size(self.video_info.as_ref().unwrap().height));
                            }

                            egui::ComboBox::from_label("Character size")
                                .selected_text(self.osd_options.character_size.clone().map_or(String::from("No video"), |s| format!("{:?}", s)))
                                .width(100.0)
                                .show_ui(ui, |ui| {
                                    if self.osd_options.character_size.is_some() {
                                        changed |= ui.selectable_value(self.osd_options.character_size.as_mut().unwrap(), CharacterSize::Race, "Race").changed();
                                        changed |= ui.selectable_value(self.osd_options.character_size.as_mut().unwrap(), CharacterSize::Small, "Small").changed();
                                        changed |= ui.selectable_value(self.osd_options.character_size.as_mut().unwrap(), CharacterSize::Large, "Large").changed();
                                        changed |= ui.selectable_value(self.osd_options.character_size.as_mut().unwrap(), CharacterSize::XLarge, "Extra Large (2k)").changed();
                                        changed |= ui.selectable_value(self.osd_options.character_size.as_mut().unwrap(), CharacterSize::Ultra, "Ultra Large (4k)").changed();
                                    }
                                });
                            ui.end_row();
                        }
                    });
            });

        if changed {
            self.update_osd_preview(ctx);
            self.config_changed = Some(Instant::now());
        }
    }

    fn srt_options(&mut self, ui: &mut Ui, ctx: &egui::Context) {
        let mut changed = false;

        CollapsingHeader::new(RichText::new("SRT Options").heading())
            .default_open(true)
            .show_unindented(ui, |ui| {
                Grid::new("srt_options")
                    .min_col_width(self.ui_dimensions.options_column1_width)
                    .show(ui, |ui| {
                        ui.label("Horizontal position").on_hover_text(tooltip_text(
                            "Horizontal position of the SRT data (% of the total video width from the left edge).",
                        ));
                        ui.horizontal(|ui| {
                            changed |= ui
                                .add(Slider::new(&mut self.srt_options.position.x, 0.0..=100.0).fixed_decimals(1))
                                .changed();

                            if ui.button("Reset").clicked() {
                                self.srt_options.position.x = 1.5;
                                changed |= true;
                            }
                        });
                        ui.end_row();

                        ui.label("Vertical position").on_hover_text(tooltip_text(
                            "Vertical position of the SR data (% of video height from the top edge).",
                        ));
                        ui.horizontal(|ui| {
                            changed |= ui
                                .add(Slider::new(&mut self.srt_options.position.y, 0.0..=100.0).fixed_decimals(1))
                                .changed();

                            if ui.button("Reset").clicked() {
                                self.srt_options.position.y = 95.0;
                                changed |= true;
                            }
                        });
                        ui.end_row();

                        ui.label("Size")
                            .on_hover_text(tooltip_text("Font size of the SRT data."));
                        ui.horizontal(|ui| {
                            changed |= ui
                                .add(Slider::new(&mut self.srt_options.scale, 10.0..=60.0).fixed_decimals(1))
                                .changed();

                            if ui.button("Reset").clicked() {
                                self.srt_options.scale = 35.0;
                                changed |= true;
                            }
                        });
                        ui.end_row();

                        ui.label("SRT data").on_hover_text(tooltip_text(
                            "Select data from the SRT file to be rendered on the video.",
                        ));
                        let options = &mut self.srt_options;
                        let has_distance = self.srt_file.as_ref().map(|s| s.has_distance).unwrap_or(true);
                        let has_debug = self.srt_file.as_ref().map(|s| s.has_debug).unwrap_or(false);
                        Grid::new("srt_selection").show(ui, |ui| {
                            changed |= ui.checkbox(&mut options.show_signal, "Signal/MCS").changed();
                            changed |= ui
                                .add_enabled(!has_debug, Checkbox::new(&mut options.show_time, "Time"))
                                .changed();
                            changed |= ui.checkbox(&mut options.show_channel, "Channel").changed();
                            changed |= ui
                                .add_enabled(!has_debug, Checkbox::new(&mut options.show_gbat, "GBat"))
                                .changed();
                            changed |= ui
                                .add_enabled(!has_debug, Checkbox::new(&mut options.show_sbat, "SBat"))
                                .changed();
                            ui.end_row();

                            changed |= ui.checkbox(&mut options.show_latency, "Delay").changed();
                            changed |= ui
                                .add_enabled(!has_debug, Checkbox::new(&mut options.show_bitrate, "Bitrate"))
                                .changed();
                            changed |= ui
                                .add_enabled(has_distance, Checkbox::new(&mut options.show_distance, "Distance"))
                                .changed();
                            ui.end_row();

                            // debug srt data
                            if has_debug {
                                ui.end_row();

                                changed |= ui
                                    .checkbox(&mut options.show_gp, "GP")
                                    .on_hover_text("Ground RSSI -dBm. 40-60 is good, 130 is low")
                                    .changed();
                                changed |= ui
                                    .checkbox(&mut options.show_sp, "SP")
                                    .on_hover_text("Sky RSSI -dBm. 40-60 is good, 130 is low")
                                    .changed();
                                changed |= ui
                                    .checkbox(&mut options.show_gtp, "GTP")
                                    .on_hover_text("Ground transmit dBm")
                                    .changed();
                                changed |= ui
                                    .checkbox(&mut options.show_stp, "STP")
                                    .on_hover_text("Sky transmit dBm")
                                    .changed();
                                ui.end_row();

                                changed |= ui
                                    .checkbox(&mut options.show_gsnr, "GSNR")
                                    .on_hover_text("Ground signal to noise ratio. 23 is excellent.")
                                    .changed();
                                changed |= ui
                                    .checkbox(&mut options.show_ssnr, "SSNR")
                                    .on_hover_text("Sky signal to noise ratio. 23 is excellent.")
                                    .changed();
                                changed |= ui
                                    .checkbox(&mut options.show_gtemp, "GTemp")
                                    .on_hover_text("Ground temperature. 75 is hot.")
                                    .changed();
                                changed |= ui
                                    .checkbox(&mut options.show_stemp, "STemp")
                                    .on_hover_text("Sky temperature. 75 is hot.")
                                    .changed();
                                ui.end_row();

                                changed |= ui
                                    .checkbox(&mut options.show_fps, "FPS")
                                    .on_hover_text("Frames received per second")
                                    .changed();
                                changed |= ui
                                    .checkbox(&mut options.show_err, "Errors")
                                    .on_hover_text("Error count")
                                    .changed();
                                ui.end_row();
                                changed |= ui
                                    .checkbox(&mut options.show_settings_cam, "Camera Set")
                                    .on_hover_text("Camera ISO/exposure parameters")
                                    .changed();
                                changed |= ui
                                    .checkbox(&mut options.show_actual_cam, "Camera Act")
                                    .on_hover_text("Actual camera parameters")
                                    .changed();
                                ui.end_row();
                                changed |= ui
                                    .checkbox(&mut options.show_cct, "CCT")
                                    .on_hover_text("Correlated Color Temperature")
                                    .changed();
                                changed |= ui
                                    .checkbox(&mut options.show_rb, "Red Balance")
                                    .on_hover_text("Red Balance values")
                                    .changed();
                                ui.end_row();
                            }
                        });
                        ui.end_row();
                    });
            });

        if changed {
            self.update_osd_preview(ctx);
            self.config_changed = Some(Instant::now());
        }
    }

    fn osd_preview(&mut self, ui: &mut Ui, ctx: &egui::Context) {
        CollapsingHeader::new(RichText::new("Preview").heading())
            .default_open(true)
            .show_unindented(ui, |ui| {
                if let (Some(handle), Some(video_info)) = (&self.osd_preview.texture_handle, &self.video_info) {
                    let preview_width = ui.available_width();
                    let aspect_ratio = video_info.width as f32 / video_info.height as f32;
                    let preview_height = preview_width / aspect_ratio;
                    let image = Image::new(handle).max_width(preview_width).max_height(preview_height);
                    let rect = ui.add(image.bg_fill(Color32::DARK_GRAY)).rect;

                    if self.osd_preview.mask_edit_mode_enabled {
                        self.draw_grid(ui, ctx, rect);
                    }

                    ui.horizontal(|ui| {
                        ui.label("Preview frame").on_hover_text(tooltip_text(
                            "The selected frame is also used for centering the OSD under OSD Options.",
                        ));
                        let preview_frame_slider = ui.add(
                            Slider::new(
                                &mut self.osd_preview.preview_frame,
                                1..=self.osd_file.as_ref().map(|f| f.frame_count).unwrap_or(1),
                            )
                            .smart_aim(false),
                        );
                        if preview_frame_slider.changed() {
                            self.update_osd_preview(ctx);
                        }
                    });
                }
            });
    }

    fn draw_grid(&mut self, ui: &mut Ui, ctx: &egui::Context, image_rect: Rect) {
        let video_width = self.video_info.as_ref().unwrap().width as f32;
        let video_height = self.video_info.as_ref().unwrap().height as f32;

        let top_left = image_rect.left_top();
        let preview_width = image_rect.width();
        let preview_height = image_rect.height();

        let grid_width = preview_width * 0.99375;
        let grid_height = preview_height;
        let cell_width = grid_width / 53.0;
        let cell_height = grid_height / 20.0;

        let painter = ui.painter_at(image_rect);

        let horizontal_offset = self.osd_options.position.x as f32 / video_width * preview_width;
        let vertical_offset = self.osd_options.position.y as f32 / video_height * preview_height;

        let response = ui
            .allocate_rect(image_rect, Sense::click())
            .on_hover_cursor(CursorIcon::Crosshair);

        for i in 0..53 {
            for j in 0..20 {
                let rect = Rect::from_min_size(
                    top_left
                        + vec2(i as f32 * cell_width, j as f32 * cell_height)
                        + vec2(horizontal_offset, vertical_offset),
                    vec2(cell_width, cell_height),
                );

                let grid_position = Coordinates::new(i, j);
                let masked = self.osd_options.get_mask(&grid_position);
                if masked {
                    painter.rect_filled(rect, 0.0, Color32::RED.gamma_multiply(0.5));
                }

                if let Some(hover_pos) = ctx.pointer_hover_pos() {
                    if rect.contains(hover_pos) {
                        painter.rect_filled(rect, 0.0, Color32::RED.gamma_multiply(0.2));
                    }
                }

                if response.clicked() {
                    if let Some(click_pos) = ctx.pointer_interact_pos() {
                        if rect.contains(click_pos) {
                            self.osd_options.toggle_mask(grid_position);
                            self.update_osd_preview(ctx);
                            self.config_changed = Instant::now().into();
                        }
                    }
                }
            }
        }

        let line_stroke = Stroke::new(1.0, Color32::GRAY.gamma_multiply(0.5));

        for i in 0..=53 {
            let x = top_left.x + i as f32 * cell_width + horizontal_offset;
            let y_min = image_rect.y_range().min + vertical_offset;
            let y_max = image_rect.y_range().max + vertical_offset;
            painter.vline(x, y_min..=y_max, line_stroke);
        }
        for i in 0..=20 {
            let x_min = image_rect.x_range().min + horizontal_offset;
            let x_max = image_rect.x_range().max + horizontal_offset;
            let y = top_left.y + i as f32 * cell_height + vertical_offset;
            painter.hline(x_min..=x_max, y, line_stroke);
        }
    }

    fn rendering_options(&mut self, ui: &mut Ui) {
        let mut changed = false;
        CollapsingHeader::new(RichText::new("Rendering Options").heading())
            .default_open(true)
            .show_unindented(ui, |ui| {
                Grid::new("render_options")
                    .min_col_width(self.ui_dimensions.options_column1_width)
                    .show(ui, |ui| {

                        let displayed_encoders = self.displayed_encoders();

                        ui.label("Encoder")
                            .on_hover_text(tooltip_text("Encoder used for rendering. In some cases not all available encoders are detected. Check the box to also show these."));
                        ui.horizontal(|ui| {
                            let selection = egui::ComboBox::from_id_source("encoder").width(350.0).show_index(
                                ui,
                                &mut self.render_settings.selected_encoder_idx,
                                displayed_encoders.len(),
                                |i| {
                                    displayed_encoders
                                        .get(i)
                                        .map(|e| e.to_string())
                                        .unwrap_or("None".to_string())
                                },
                            );
                            if selection.changed() {
                                changed |= true;
                            }

                            #[cfg(debug_assertions)]
                            if ui
                                .add(Checkbox::without_text(&mut self.render_settings.show_undetected_encoders))
                                .on_hover_text(tooltip_text("Show undetected encoders."))
                                .changed() {
                                    self.render_settings.selected_encoder_idx = 0;
                                    changed |= true;
                            }
                        });
                        ui.end_row();

                        let selected_encoder = self.get_selected_encoder();
                        let bitrate_enabled = !self.render_settings.keep_quality;
                        let mut constant_quality_available = false;
                        
                        if let Some(selected_encoder) = selected_encoder {
                            if selected_encoder.constant_quality_args != None {
                                constant_quality_available = true;
                            } else {
                                changed |= self.render_settings.keep_quality;
                                self.render_settings.keep_quality = false;
                            }
                        }

                        ui.label("Encoding bitrate").on_hover_text(tooltip_text("Target bitrate of the rendered video."));
                        if bitrate_enabled {
                            changed = ui.add_enabled(bitrate_enabled, Slider::new(&mut self.render_settings.bitrate_mbps, 0..=160).text("Mbps")).changed();
                        } else {
                            ui.label("[AUTO]");
                        }
                        ui.end_row();

                        ui.label("Constant quality mode").on_hover_text(tooltip_text("Automatically adjust bitrate to preserve as much details as possible. Uses less disk space than comparable quality with constant bitrate."));
                        changed |= ui.add_enabled(constant_quality_available, Checkbox::without_text(&mut self.render_settings.keep_quality)).changed();
                        ui.end_row();

                        ui.label("Upscale to 1440p for YT").on_hover_text(tooltip_text("Upscale the output video to 1440p to get better quality after uploading to YouTube."));
                        changed |= ui.add(Checkbox::without_text(&mut self.render_settings.upscale)).changed();
                        ui.end_row();

                        ui.label("Rescale to 4:3 aspect ratio").on_hover_text(tooltip_text("Rescale the output video to 4:3 aspect ratio, useful when you have 4:3 camera and recording is done by VRX in \"4:3 Fullscreen\" mode."));
                        changed |= ui.add(Checkbox::without_text(&mut self.render_settings.rescale_to_4x3_aspect)).changed();
                        ui.end_row();

                        ui.label("Rendering live view").on_hover_text(tooltip_text("Enables live view of rendered frames once rendering is started."));
                        changed |= ui.add(Checkbox::without_text(&mut self.render_settings.rendering_live_view)).changed();
                        ui.end_row();

                        ui.label("Chroma key").on_hover_text(tooltip_text("Render the video with a chroma key background instead of the input video so the OSD can be used as overlay video."));
                        ui.horizontal(|ui| {
                            changed |= ui.add(Checkbox::without_text(&mut self.render_settings.use_chroma_key)).changed();
                            changed |= ui.color_edit_button_rgba_unmultiplied(&mut self.render_settings.chroma_key).changed();
                        });
                        ui.end_row();
                    });

                ui.label(RichText::new("Note: Apple ProRes codec supports transparency.").text_style(TextStyle::Name("Tooltip".into())).weak());
            });

        if changed {
            self.config_changed = Some(Instant::now());
        }
    }

    pub fn displayed_encoders(&self) -> Vec<Encoder> {
        #[cfg(debug_assertions)]
        if self.render_settings.show_undetected_encoders {
            return self.encoders.clone()
        }
        return self.detected_encoders.clone();
    }

    pub fn sort_and_filter_encoders(encoders: &Vec<Encoder>) -> Vec<Encoder> {
        let mut filtered_encoders: Vec<Encoder> = encoders
            .iter()
            .filter(|e| e.detected)
            .map(|x| x.clone())
            .collect();

        filtered_encoders.sort_by(|a, b| {
            const CODEC_PRIORITY: [Codec; 4] = [Codec::H265, Codec::H264, Codec::VP9, Codec::ProRes];

            let a_priority = CODEC_PRIORITY
                .iter()
                .position(|c| *c == a.codec)
                .unwrap_or(CODEC_PRIORITY.len());
            let b_priority = CODEC_PRIORITY
                .iter()
                .position(|c| *c == b.codec)
                .unwrap_or(CODEC_PRIORITY.len());
            let type_cmp = a_priority.cmp(&b_priority);

            let hardware_cmp = b.hardware.cmp(&a.hardware);

            let name_cmp = a.name.cmp(&b.name);

            hardware_cmp.then_with(|| type_cmp).then_with(|| name_cmp)
        });

        filtered_encoders
    }
}
