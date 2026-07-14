mod state;
mod requests;

use crate::{requests::{get_data, update_data}, state::{
    ButtonType::{self, EIGHT, FIVE, FOUR, SIX},
    Config, MainAppRawState, MainAppState,
}};
use eframe::egui::{self, Color32, FontData, FontDefinitions, FontFamily, RichText, TextEdit};
use serde::{Deserialize, Serialize};
use serde_json;
use strum::IntoEnumIterator;
use notify_rust::Notification;

fn main() {
    let native_options = eframe::NativeOptions::default();

    eframe::run_native(
        "DJMAX 디스코드 위젯 관리자",
        native_options,
        Box::new(|cc| Ok(Box::new(MainApp::new(cc)))),
    )
    .expect("앱을 키지 못했습니다.");
}

fn get_font() -> FontDefinitions {
    let mut fonts = FontDefinitions::default();
    fonts.font_data.insert(
        "pretendard".to_owned(),
        std::sync::Arc::new(FontData::from_static(include_bytes!(
            "../assets/NanumGothic.otf"
        ))),
    );

    fonts
        .families
        .get_mut(&FontFamily::Proportional)
        .unwrap()
        .insert(0, "pretendard".to_owned());

    fonts
}

#[derive(Default, Serialize, Deserialize)]
struct MainApp {
    state: MainAppState,
    raw: MainAppRawState,
}

impl MainApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_global_style.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.
        cc.egui_ctx.set_fonts(get_font());

        if let Some(storage) = cc.storage {
            // eframe saves app state under the key eframe::APP_KEY
            if let Some(app) = eframe::get_value::<Self>(storage, eframe::APP_KEY) {
                return app;
            }
        }

        Self::default()
    }
}

impl eframe::App for MainApp {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ui, |ui| {
            let raw: &mut MainAppRawState = &mut self.raw;

            ui.heading("설정");
            ui.separator();

            let mut enabled = false;
            ui.columns(2, |columns| {
                columns[0].vertical(|ui| {
                    ui.label("콘피그");
                    ui.add(
                        TextEdit::multiline(&mut raw.config)
                            .font(egui::TextStyle::Monospace)
                            .code_editor()
                            .desired_rows(5)
                            .desired_width(f32::INFINITY),
                    )
                });

                enabled = match serde_json::from_str::<Config>(&raw.config) {
                    Ok(parsed_config) => {
                        self.state.config = parsed_config;
                        true
                    }
                    Err(_) => false,
                };

                columns[1].vertical(|ui| {
                    ui.add_enabled_ui(enabled, |ui| {
                        ui.label("V-ARCHIVE 닉네임");
                        ui.text_edit_singleline(&mut self.state.username);

                        ui.label("사용 버튼");
                        for button in ButtonType::iter() {
                            ui.radio_value(&mut self.state.button, button, button.to_string());
                        }
                    })
                });
            });

            ui.add_space(60.0);

            ui.add_enabled_ui(enabled && !self.state.username.is_empty(), |ui| {
                ui.heading("데이터");
                ui.separator();

                ui.spacing_mut().button_padding = egui::vec2(20.0, 10.0);

                let data_btn = egui::Button::new
                    (RichText::new("데이터 구하기").color(Color32::WHITE))
                    .fill(Color32::from_rgb(130, 106, 237));

                ui.horizontal(|ui| {
                    if ui.add(data_btn).clicked() {
                        self.state.summary = get_data(&self.state.username, match self.state.button {
                            FOUR => 4,
                            FIVE => 5,
                            SIX => 6,
                            EIGHT => 8
                        });
                    }

                    if let Some(summary) = &self.state.summary {
                        if ui.button("데이터 업로드").clicked() {
                            let _ = Notification::new()
                                .summary("위젯 데이터 업데이트")
                                .body(&update_data(&self.state.config, &summary, &self.state.button, &self.state.username))
                                .show();
                        }
                    }
                });

                ui.add_space(15.0);

                if let Some(summary) = &self.state.summary {
                    ui.label(format!("- 클리어 패턴 수: {}개", summary.clears));
                    ui.label(format!("- 퍼펙트 패턴 수: {}개", summary.perfects));
                    ui.label(format!("- 맥스 콤보 수: {}개", summary.max_combos));
                    ui.label(format!("- 평균 정확도: {}%", summary.avg_rating));
                    ui.label(format!("- 티어 포인트: {}", summary.tier_point));
                    ui.label(format!("- DJ Class: {}", summary.dj_class));
                } else {
                    ui.label("아무 데이터도 없습니다...");
                }
            });
        });
    }
}
