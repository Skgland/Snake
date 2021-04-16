use crate::game::GameState;
use conrod_core::image::Id;
use conrod_core::image::Map;
use conrod_core::position::Positionable;
use conrod_core::position::Sizeable;
use conrod_core::widget;
use conrod_core::widget::Widget;
use conrod_core::widget_ids;
use conrod_core::Labelable;
use conrod_core::Ui;
use conrod_core::UiCell;
use core::fmt::Display;
use glutin_window::GlutinWindow;
use graphics::Graphics;
use piston_window::PistonWindow;
use piston_window::Window;
use rusttype::gpu_cache::Cache;
use std::fmt::Debug;
use std::fmt::Error;
use std::fmt::Formatter;

// Generate a unique `WidgetId` for each widget.
widget_ids! {
    pub struct Ids {
        main_canvas,
        menu_title,
        level_buttons[],
        level_selection_button,
        editor_button,
        contiue_button,
        options_button,
        back_button,
        quit_button,
    }
}

pub struct RenderContext<'font, G: Graphics> {
    pub gl: G,
    pub text_texture_cache: opengl_graphics::Texture,
    pub text_vertex_data: Vec<u8>,
    pub glyph_cache: Cache<'font>,
}

pub struct GUI {
    pub image_map: Map<opengl_graphics::Texture>,
    pub image_ids: Vec<Id>,
    pub ui: Ui,
    pub ids: Ids,
    pub active_menu: GUIVisibility,
    pub fullscreen: bool,
}

#[allow(dead_code)]
pub enum GUIVisibility {
    //*NO GUI VISIBLE (ONLY GAME VISIBLE)
    GameOnly(GameState),
    //*INTERACTIVE MENU VISIBLE ON TOP OF GAME
    //* E.g. Inventory, Pause Menu
    OverlayMenu(MenuType, GameState),
    //*ONLY MENU VISIBLE (NO GAME VISIBLE)
    //* Main Menu, Level Selection, Options
    MenuOnly(MenuType),
}

impl Debug for GUIVisibility {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        use self::GUIVisibility::*;
        match self {
            GameOnly(_) => Ok(()),

            MenuOnly(menu) | OverlayMenu(menu, _) => Debug::fmt(&menu.menu_name(), f),
        }
    }
}

impl GUIVisibility {
    pub fn handle_esc(&mut self, window: &mut PistonWindow<GlutinWindow>) {
        match self {
            GUIVisibility::GameOnly(state) => {
                if let GameState::GameOver { .. } = state {
                    *self = GUIVisibility::MenuOnly(MenuType::Main)
                } else {
                    *self = GUIVisibility::OverlayMenu(MenuType::Pause, state.clone());
                }
            }
            GUIVisibility::MenuOnly(menu_type) | GUIVisibility::OverlayMenu(menu_type, _) => {
                let menu = menu_type.back();
                if let Some(menu) = menu {
                    *self = menu
                } else {
                    window.set_should_close(true);
                }
            }
        }
    }
}

impl Display for GUIVisibility {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        Debug::fmt(self, f)
    }
}

#[derive(Debug)]
pub enum MenuType {
    Main,
    Pause,
}

impl Display for MenuType {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        Debug::fmt(self, f)
    }
}

pub trait Menu: Debug {
    fn menu_name(&self) -> String;

    fn handle_input(&self);

    fn update(&self, ui: &mut UiCell, ids: &mut Ids) -> Box<dyn FnMut(&mut GUIVisibility) -> ()>;

    fn back(&self) -> Option<GUIVisibility>;
}

impl Menu for MenuType {
    fn menu_name(&self) -> String {
        match self {
            MenuType::Main => String::from("Main Menu"),
            MenuType::Pause => String::from("Pause Menu"),
        }
    }

    fn handle_input(&self) {
        match self {
            MenuType::Main => {}
            MenuType::Pause => {}
        }
    }

    fn update(&self, ui: &mut UiCell, ids: &mut Ids) -> Box<dyn FnMut(&mut GUIVisibility) -> ()> {
        match self {
            MenuType::Pause => {
                widget::Text::new("Pause Menu")
                    .font_size(30)
                    .mid_top_of(ids.main_canvas)
                    .set(ids.menu_title, ui);

                let mut result: Box<dyn FnMut(&mut GUIVisibility) -> ()> = Box::new(|_| {});

                for _press in widget::Button::new()
                    .label("Continue")
                    .label_font_size(30)
                    .middle_of(ids.main_canvas)
                    .padded_kid_area_wh_of(ids.main_canvas, ui.win_h / 4.0)
                    .set(ids.contiue_button, ui)
                {
                    result = Box::new(|vis| match vis {
                        GUIVisibility::OverlayMenu(_, game) => {
                            *vis = GUIVisibility::GameOnly(game.clone())
                        }
                        _ => {}
                    });
                }
                result
            }

            MenuType::Main => {
                let mut result: Box<dyn FnMut(&mut GUIVisibility) -> ()> = Box::new(|_| {});

                for _press in widget::Button::new()
                    .label("Start Game")
                    .middle_of(ids.main_canvas)
                    .padded_kid_area_wh_of(ids.main_canvas, ui.win_h / 4.0)
                    .set(ids.editor_button, ui)
                {
                    result = Box::new(|a| *a = GUIVisibility::GameOnly(GameState::new()));
                }

                widget::Text::new("Main Menu")
                    .font_size(30)
                    .mid_top_of(ids.main_canvas)
                    .set(ids.menu_title, ui);
                result
            }
        }
    }

    fn back(&self) -> Option<GUIVisibility> {
        match self {
            MenuType::Main => None,
            MenuType::Pause => Some(GUIVisibility::MenuOnly(MenuType::Main)),
        }
    }
}
