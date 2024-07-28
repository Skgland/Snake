use crate::game::{GameState, KeyMap};
use eframe::egui;
use std::fmt::{Debug, Display, Error, Formatter};

#[derive(Debug)]
pub enum Gui {
    GameOnly(GameState),
    PauseMenu(GameState),
    MainMenu,
}

impl eframe::App for Gui {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let (pressed_escape, f11_pressed, is_fullscreen) = ui.input(|ui| {
                (
                    ui.key_pressed(egui::Key::Escape),
                    ui.key_pressed(egui::Key::F11),
                    ui.viewport().fullscreen.unwrap_or_default(),
                )
            });

            if pressed_escape {
                if !self.handle_esc() {
                    ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                }
            }

            if f11_pressed {
                ui.ctx()
                    .send_viewport_cmd(egui::ViewportCommand::Fullscreen(!is_fullscreen))
            }

            match self {
                Gui::GameOnly(game) => {
                    let key_bindings = KeyMap {
                        up: vec![egui::Key::ArrowUp, egui::Key::W],
                        down: vec![egui::Key::ArrowDown, egui::Key::S],
                        left: vec![egui::Key::ArrowLeft, egui::Key::A],
                        right: vec![egui::Key::ArrowRight, egui::Key::D],
                    };

                    for action in key_bindings.actions(ui) {
                        game.perform(action);
                    }

                    game.step_time();
                    ui.ctx().request_repaint();

                    ui.add(game);
                }
                Gui::PauseMenu(game) => {
                    ui.add(&mut *game);
                    let game = game.clone();

                    egui::Window::new("Pause Menu").show(ctx, |ui| {
                        if ui.button("Continue").clicked() {
                            *self = Gui::GameOnly(game);
                        };
                    });
                }
                Gui::MainMenu => {
                    ui.group(|ui| {
                        ui.heading("Main Menu");

                        if ui.button("Start Game").clicked() {
                            *self = Gui::GameOnly(GameState::new());
                        };
                    });
                }
            };
        });
    }
}

impl Gui {
    pub fn new() -> Self {
        Gui::MainMenu
    }

    pub fn handle_esc(&mut self) -> bool {
        match self {
            Gui::GameOnly(state) => {
                if let GameState::GameOver { .. } = state {
                    *self = Gui::MainMenu
                } else {
                    *self = Gui::PauseMenu(state.clone());
                }
            }
            Gui::PauseMenu(_) => {
                *self = Gui::MainMenu;
            }
            Gui::MainMenu => {
                return false;
            }
        }
        true
    }
}

impl Display for Gui {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        Debug::fmt(self, f)
    }
}
