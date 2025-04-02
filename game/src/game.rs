use eframe::egui::{
    self, pos2, vec2, Color32, CornerRadius, Painter, Rect, Sense, Stroke, StrokeKind,
};
use rand::distr::{Distribution, Uniform};
use std::{
    collections::VecDeque,
    convert::TryFrom,
    time::{Duration, Instant},
};

#[derive(Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Debug)]
pub enum Direction {
    UP,
    DOWN,
    LEFT,
    RIGHT,
}

#[derive(Debug, Ord, PartialOrd, PartialEq, Eq, Clone, Copy)]
pub struct ObjectCoordinate {
    pub x: i8,
    pub y: i8,
}

impl std::ops::Add<Direction> for ObjectCoordinate {
    type Output = ObjectCoordinate;

    fn add(self, other: Direction) -> Self {
        match other {
            Direction::DOWN => ObjectCoordinate {
                x: self.x,
                y: self.y + 1,
            },
            Direction::UP => ObjectCoordinate {
                x: self.x,
                y: self.y - 1,
            },
            Direction::RIGHT => ObjectCoordinate {
                x: self.x + 1,
                y: self.y,
            },
            Direction::LEFT => ObjectCoordinate {
                x: self.x - 1,
                y: self.y,
            },
        }
    }
}

#[derive(Clone, Debug)]
pub enum GameState {
    GameOver {
        score: usize,
        settings: GameSettings,
    },
    GameState {
        apple: ObjectCoordinate,
        snake: VecDeque<ObjectCoordinate>,
        direction: Direction,
        next_step: Option<Instant>,
        settings: GameSettings,
    },
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum WallBehaviour {
    #[default]
    Death,
    Loop,
}
impl WallBehaviour {
    pub(crate) fn name(&self) -> &'static str {
        match self {
            WallBehaviour::Death => "Death",
            WallBehaviour::Loop => "Loop",
        }
    }
}

#[derive(Debug, Clone)]
pub struct GameSettings {
    pub wall_behaviour: WallBehaviour,
    pub size: [i8; 2],
}

impl Default for GameSettings {
    fn default() -> Self {
        Self {
            wall_behaviour: Default::default(),
            size: [32; 2],
        }
    }
}

impl GameState {
    pub fn perform(&mut self, action: Action) {
        if let GameState::GameState { direction, .. } = self {
            match action {
                Action::UP => *direction = Direction::UP,
                Action::DOWN => *direction = Direction::DOWN,
                Action::LEFT => *direction = Direction::LEFT,
                Action::RIGHT => *direction = Direction::RIGHT,
            }
        }
    }
}
pub enum Action {
    UP,
    DOWN,
    LEFT,
    RIGHT,
}
pub struct KeyMap {
    pub up: Vec<egui::Key>,
    pub down: Vec<egui::Key>,
    pub left: Vec<egui::Key>,
    pub right: Vec<egui::Key>,
}
impl KeyMap {
    pub fn actions(&self, ui: &mut egui::Ui) -> impl IntoIterator<Item = Action> {
        ui.ctx().input(|input| {
            IntoIterator::into_iter([
                (&self.up, Action::UP),
                (&self.down, Action::DOWN),
                (&self.left, Action::LEFT),
                (&self.right, Action::RIGHT),
            ])
            .flat_map(|(keys, action)| {
                keys.iter()
                    .any(|key| input.key_pressed(*key))
                    .then_some(action)
            })
            .collect::<Vec<_>>()
        })
    }
}

impl eframe::egui::Widget for &mut GameState {
    fn ui(self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        ui.centered_and_justified(|ui| {
            match self {
                GameState::GameState {
                    apple,
                    snake,
                    settings,
                    ..
                } => {
                    let size_x: f32 =
                        settings.size[0] as f32 * (TILE_SIZE + TILE_PADDING) + TILE_PADDING;
                    let size_y: f32 =
                        settings.size[0] as f32 * (TILE_SIZE + TILE_PADDING) + TILE_PADDING;
                    let scale = (ui.available_width() / size_x).min(ui.available_height() / size_y);

                    let (_response, painter) = ui.allocate_painter(
                        vec2(size_x * scale, size_y * scale),
                        Sense::focusable_noninteractive(),
                    );

                    fn draw_tile(
                        painter: &Painter,
                        tile: &ObjectCoordinate,
                        color: Color32,
                        scale: f32,
                    ) {
                        painter.rect_filled(
                            Rect::from_min_size(
                                pos2(
                                    ((tile.x as f32) * (TILE_SIZE + TILE_PADDING) + TILE_PADDING)
                                        * scale,
                                    ((tile.y as f32) * (TILE_SIZE + TILE_PADDING) + TILE_PADDING)
                                        * scale,
                                ),
                                vec2(TILE_SIZE * scale, TILE_SIZE * scale),
                            ),
                            CornerRadius::ZERO,
                            color,
                        );
                    }

                    // draw background
                    painter.rect_filled(
                        Rect::from_min_size(pos2(0.0, 0.0), vec2(size_x * scale, size_y * scale)),
                        CornerRadius::ZERO,
                        Color32::BLUE,
                    );

                    // draw corners
                    for x in 1..settings.size[0] {
                        let cx =
                            ((x as f32) * (TILE_SIZE + TILE_PADDING) + TILE_PADDING / 2.0) * scale;

                        for y in 1..settings.size[1] {
                            let cy = ((y as f32) * (TILE_SIZE + TILE_PADDING) + TILE_PADDING / 2.0)
                                * scale;
                            painter.circle_filled(pos2(cx, cy), scale, Color32::BLACK);
                        }
                    }

                    painter.rect_stroke(
                        Rect::from_min_size(
                            pos2(TILE_PADDING / 2.0 * scale, TILE_PADDING / 2.0 * scale),
                            vec2(
                                settings.size[0] as f32 * (TILE_SIZE + TILE_PADDING) * scale,
                                settings.size[1] as f32 * (TILE_SIZE + TILE_PADDING) * scale,
                            ),
                        ),
                        CornerRadius::ZERO,
                        Stroke::new(scale, Color32::BLACK),
                        StrokeKind::Inside,
                    );

                    // draw apple
                    draw_tile(&painter, apple, Color32::GREEN, scale);

                    // draw snake
                    for segment in snake {
                        draw_tile(&painter, segment, Color32::RED, scale);
                    }
                }
                GameState::GameOver { score, settings } => {
                    let score = *score;
                    let settings = settings.clone();
                    ui.vertical_centered(|ui| {
                        ui.heading(format!("Final length: {score}"));

                        if ui.button("Start New game").clicked() {
                            *self = GameState::new(settings)
                        }
                    });
                }
            }
        })
        .response
    }
}

pub fn delay_for_length(length: u32, settings: &GameSettings) -> Duration {
    const MAX_DELAY: Duration = Duration::from_millis(250);

    let max_length: u32 = settings.size[0] as u32 * settings.size[1] as u32;
    MAX_DELAY - MAX_DELAY * length / max_length
}

pub fn new_snake(settings: &GameSettings) -> VecDeque<ObjectCoordinate> {
    vec![
        ObjectCoordinate {
            x: settings.size[0] / 2,
            y: settings.size[1] / 2,
        },
        ObjectCoordinate {
            x: settings.size[0] / 2,
            y: settings.size[1] / 2 + 1,
        },
        ObjectCoordinate {
            x: settings.size[0] / 2,
            y: settings.size[1] / 2 + 2,
        },
    ]
    .into()
}

pub fn generate_apple(settings: &GameSettings) -> ObjectCoordinate {
    let mut rng = ::rand::rng();

    let x: i8 = Uniform::try_from(0..settings.size[0])
        .expect("the game area width shouldn't be 0")
        .sample(&mut rng);
    let y: i8 = Uniform::try_from(0..settings.size[1])
        .expect("the game area height shouldn't be 0")
        .sample(&mut rng);
    ObjectCoordinate { x, y }
}

pub fn new_position(
    old_pos: ObjectCoordinate,
    dir: Direction,
    settings: &GameSettings,
) -> Option<ObjectCoordinate> {
    let mut obj = old_pos + dir;

    match settings.wall_behaviour {
        WallBehaviour::Death => {
            if !(0..settings.size[0]).contains(&obj.x) || !(0..settings.size[1]).contains(&obj.y) {
                None
            } else {
                Some(obj)
            }
        }
        WallBehaviour::Loop => {
            while obj.x < 0 {
                obj.x += settings.size[0];
            }
            if obj.x >= settings.size[0] {
                obj.x %= settings.size[0];
            }

            while obj.y < 0 {
                obj.y += settings.size[1];
            }
            if obj.y >= settings.size[1] {
                obj.y %= settings.size[1];
            }
            Some(obj)
        }
    }
}

impl GameState {
    pub fn new(settings: GameSettings) -> GameState {
        GameState::GameState {
            // Rotation for the square.
            apple: generate_apple(&settings),
            snake: new_snake(&settings),
            direction: Direction::UP,
            next_step: None,
            settings,
        }
    }

    pub fn step_time(&mut self) {
        if let GameState::GameState {
            apple,
            snake,
            direction,
            next_step,
            settings,
        } = self
        {
            let Some(next_step) = next_step else {
                *next_step = Some(Instant::now() + delay_for_length(snake.len() as u32, &settings));
                return;
            };

            if *next_step <= Instant::now() {
                //spawn new head in current direction of previous head
                let new_head = if let Some(head) = snake.front() {
                    let Some(pos) = new_position(*head, *direction, &settings) else {
                        *self = GameState::GameOver {
                            score: snake.len(),
                            settings: settings.clone(),
                        };
                        return;
                    };
                    pos
                } else {
                    unreachable!("Snake always has some body segments")
                };

                //check if new_head collides with old tile
                if snake.iter().any(|tile| tile == &new_head) {
                    *self = GameState::GameOver {
                        score: snake.len(),
                        settings: settings.clone(),
                    };
                    return;
                }

                //add new head
                snake.push_front(new_head);

                // either consume an apple or remove the tail
                if apple == &new_head {
                    // generate new apple
                    *apple = generate_apple(&settings);
                } else {
                    // no apple consumed remove last tail
                    snake.pop_back();
                }

                *next_step = Instant::now() + delay_for_length(snake.len() as u32, &settings);
            }
        }
    }
}

pub const TILE_PADDING: f32 = 2.0;
pub const TILE_SIZE: f32 = 10.0;
