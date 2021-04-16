#![allow(dead_code, unused_variables)]

use std::collections::btree_set::BTreeSet;

use conrod_core::{
    color::Colorable, input::RenderArgs, position::Positionable, widget, widget::Widget,
    Borderable, Labelable,
};

use glutin_window::GlutinWindow;
use graphics::Context;
use opengl_graphics::GlGraphics;
pub use piston_window::*;
use piston_window::{texture::UpdateTexture, PistonWindow};
use std::collections::btree_map::BTreeMap;

use crate::{
    game::level::Direction, game::GameState, game::APPLE_COLOR, game::GAME_SIZE, game::PLAYER_SIZE,
    game::PLAYER_SQUARE, game::TILE_SIZE, gui::GUIVisibility::GameOnly,
    gui::GUIVisibility::OverlayMenu, gui::*,
};

pub struct App {
    pub gui: GUI,
    keys_down: BTreeSet<Key>,
}

pub enum Action {
    UP,
    DOWN,
    LEFT,
    RIGHT,
}

impl Action {
    pub fn perform(&self, state: &mut GameState) {
        if let GameState::GameState { direction, .. } = state {
            match self {
                Action::UP => *direction = Direction::UP,
                Action::DOWN => *direction = Direction::DOWN,
                Action::LEFT => *direction = Direction::LEFT,
                Action::RIGHT => *direction = Direction::RIGHT,
            }
        }
    }
}

type G = opengl_graphics::GlGraphics;

const GREEN: [f32; 4] = [0.0, 1.0, 0.0, 1.0];
const BLUE: [f32; 4] = [0.0, 0.0, 1.0, 1.0];
const BLACK: [f32; 4] = [0.0, 0.0, 0.0, 1.0];

impl App {
    pub fn new(gui: GUI) -> Self {
        App {
            gui,
            keys_down: BTreeSet::new(),
        }
    }

    pub fn render(&self, context: &mut RenderContext<G>, args: &RenderArgs) {
        use graphics::*;

        // Specify how to get the drawable texture from the image. In this case, the image
        // *is* the texture.
        fn texture_from_image<T>(img: &T) -> &T {
            img
        }

        let RenderContext {
            gl,
            glyph_cache,
            text_texture_cache,
            text_vertex_data,
            ..
        } = context;

        let App {
            gui:
                GUI {
                    ui,
                    image_map,
                    active_menu,
                    ..
                },
            ..
        } = self;

        // A function used for caching glyphs to the texture cache.
        let cache_queued_glyphs = |_graphics: &mut GlGraphics,
                                   cache: &mut opengl_graphics::Texture,
                                   rect: conrod_core::text::rt::Rect<u32>,
                                   data: &[u8]| {
            let offset = [rect.min.x, rect.min.y];
            let size = [rect.width(), rect.height()];
            let format = piston_window::texture::Format::Rgba8;
            text_vertex_data.clear();
            text_vertex_data.extend(data.iter().flat_map(|&b| vec![255, 255, 255, b]));
            UpdateTexture::update(cache, &mut (), format, &text_vertex_data[..], offset, size)
                .expect("failed to update texture")
        };

        gl.draw(args.viewport(), |c, gl| {
            match active_menu {
                GUIVisibility::GameOnly(_) => {
                    // Clear the screen.
                    clear(super::game::color::D_RED, gl);
                }
                _ => clear(BLACK, gl),
            }

            self.render_game(args, c, gl);

            let view = c.store_view();

            conrod_piston::draw::primitives(
                ui.draw(),
                view,
                gl,
                text_texture_cache,
                glyph_cache,
                image_map,
                cache_queued_glyphs,
                texture_from_image,
            );
        });
    }

    fn render_game(&self, args: &RenderArgs, c: Context, gl: &mut GlGraphics) {
        if let GameOnly(state) | OverlayMenu(_, state) = &self.gui.active_menu {
            if let GameState::GameState { apple, .. } = &state {
                let (x, y) = (args.window_size[0] / 2.0, args.window_size[1] / 2.0);

                let c = c.trans(x, y);

                {
                    let transform = c
                        .trans(-PLAYER_SIZE / 2.0, -PLAYER_SIZE / 2.0)
                        .trans(-GAME_SIZE as f64 * TILE_SIZE, -GAME_SIZE as f64 * TILE_SIZE)
                        .transform;
                    let game_area = [
                        0.0,
                        0.0,
                        GAME_SIZE as f64 * 2.0 * TILE_SIZE,
                        GAME_SIZE as f64 * 2.0 * TILE_SIZE,
                    ];

                    rectangle(BLUE, game_area, transform, gl);
                }

                let transform = c
                    .trans(-PLAYER_SIZE / 2.0, -PLAYER_SIZE / 2.0)
                    .trans(TILE_SIZE * (apple.x as f64), TILE_SIZE * (apple.y as f64))
                    .transform;

                rectangle(APPLE_COLOR, PLAYER_SQUARE, transform, gl);

                state.draw_player(c, gl);
            } else if let GameState::GameOver { score } = &state {
            }
        }

        use graphics::*;
    }

    pub fn input(&mut self, event: Input, window: &mut PistonWindow<GlutinWindow>) {
        if let Some(cr_event) = conrod_piston::event::convert(
            Event::Input(event.clone(), None),
            self.gui.ui.win_w,
            self.gui.ui.win_h,
        ) {
            self.gui.ui.handle_event(cr_event);
        }

        match (&self.gui.active_menu, event) {
            (
                GUIVisibility::GameOnly(_),
                Input::Button(ButtonArgs {
                    button: Button::Keyboard(key),
                    state,
                    ..
                }),
            ) => {
                match state {
                    ButtonState::Press => self.keys_down.insert(key),
                    ButtonState::Release => self.keys_down.remove(&key),
                };
                //println!("{:?}", key);
            }
            (_, _) => (),
        };
    }

    pub fn toggle_fullscreen(window: &mut PistonWindow<GlutinWindow>, current: &mut bool) {
        if *current {
            window.window.ctx.window().set_fullscreen(None);
            *current = false;
        } else {
            let monitor = window.window.ctx.window().get_primary_monitor();
            window.window.ctx.window().set_fullscreen(Some(monitor));
            *current = true;
        }
    }

    pub fn update(&mut self, args: UpdateArgs, window: &mut PistonWindow<GlutinWindow>) {
        use GUIVisibility::*;

        let ui = &mut self.gui.ui.set_widgets();

        {
            use conrod_core::event::{Button, Event, Release, Ui};
            for event in ui.global_input().events() {
                if let Event::Ui(event) = event {
                    match event {
                        Ui::Release(
                            _,
                            Release {
                                button: Button::Keyboard(Key::F11),
                                ..
                            },
                        ) => Self::toggle_fullscreen(window, &mut self.gui.fullscreen),
                        Ui::Release(
                            _,
                            Release {
                                button: Button::Keyboard(Key::Escape),
                                ..
                            },
                        ) => {
                            self.gui.active_menu.handle_esc(window);
                        }
                        _ => (),
                    }
                }
            }
        }

        //necessary so that when we stop drawing anything in F1 mode, Resize events will still be processed
        widget::canvas::Canvas::new()
            .border_rgba(0.0, 0.0, 0.0, 0.0)
            .rgba(0.0, 0.0, 0.0, 0.0)
            .set(self.gui.ids.main_canvas, ui);

        // Rotate 2 radians per second.

        let mut key_map: BTreeMap<Key, Action> = BTreeMap::new();

        key_map.insert(Key::W, Action::UP);
        key_map.insert(Key::A, Action::LEFT);
        key_map.insert(Key::S, Action::DOWN);
        key_map.insert(Key::D, Action::RIGHT);
        key_map.insert(Key::Up, Action::UP);
        key_map.insert(Key::Left, Action::LEFT);
        key_map.insert(Key::Down, Action::DOWN);
        key_map.insert(Key::Right, Action::RIGHT);

        match &mut self.gui.active_menu {
            //update game state while in game
            GameOnly(state) => {
                let keys_down = &self.keys_down;
                key_map
                    .iter()
                    .filter(|(&k, _)| keys_down.contains(&k))
                    .for_each(|(_, action)| action.perform(state));
                state.handle_input(); // also dose game update
            }
            MenuOnly(..) | OverlayMenu(..) => {}
        }

        match &self.gui.active_menu {
            GameOnly(GameState::GameOver { score }) => {
                widget::Text::new(&format!("Final length: {}", score))
                    .font_size(30)
                    .middle_of(self.gui.ids.main_canvas)
                    .set(self.gui.ids.menu_title, ui);

                let mut result: Box<dyn FnMut(&mut GUIVisibility) -> ()> = Box::new(|_| {});

                for press in widget::Button::new()
                    .label("Start New Game")
                    .down_from(self.gui.ids.menu_title, 10.0)
                    .set(self.gui.ids.editor_button, ui)
                {
                    result = Box::new(|a| *a = GUIVisibility::GameOnly(GameState::new()));
                }

                result(&mut self.gui.active_menu)
            }
            GameOnly(_) => (),
            MenuOnly(menu) | OverlayMenu(menu, _) => {
                let mut fun = menu.update(ui, &mut self.gui.ids);
                fun(&mut self.gui.active_menu);
            }
        }
    }
}
