use eframe::egui::{self, pos2, vec2, Color32, Painter, Rect, Rounding, Sense, Stroke};
use std::{
    collections::VecDeque,
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
    },
    GameState {
        apple: ObjectCoordinate,
        snake: VecDeque<ObjectCoordinate>,
        direction: Direction,
        next_step: Option<Instant>,
    },
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
                GameState::GameState { apple, snake, .. } => {
                    const SIZE: f32 = GAME_SIZE as f32 * (TILE_SIZE + TILE_PADDING) + TILE_PADDING;
                    let scale = ui.available_size().min_elem() / SIZE;

                    let (_response, painter) = ui.allocate_painter(
                        vec2(SIZE * scale, SIZE * scale),
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
                            Rounding::ZERO,
                            color,
                        );
                    }

                    // draw background
                    painter.rect_filled(
                        Rect::from_min_size(pos2(0.0, 0.0), vec2(SIZE * scale, SIZE * scale)),
                        Rounding::ZERO,
                        Color32::BLUE,
                    );

                    // draw corners
                    for x in 1..GAME_SIZE {
                        let cx =
                            ((x as f32) * (TILE_SIZE + TILE_PADDING) + TILE_PADDING / 2.0) * scale;

                        for y in 1..GAME_SIZE {
                            let cy = ((y as f32) * (TILE_SIZE + TILE_PADDING) + TILE_PADDING / 2.0)
                                * scale;
                            painter.circle_filled(pos2(cx, cy), scale, Color32::BLACK);
                        }
                    }

                    painter.rect_stroke(
                        Rect::from_min_size(
                            pos2(TILE_PADDING / 2.0 * scale, TILE_PADDING / 2.0 * scale),
                            vec2(
                                GAME_SIZE as f32 * (TILE_SIZE + TILE_PADDING) * scale,
                                GAME_SIZE as f32 * (TILE_SIZE + TILE_PADDING) * scale,
                            ),
                        ),
                        Rounding::ZERO,
                        Stroke::new(scale, Color32::BLACK),
                    );

                    // draw apple
                    draw_tile(&painter, apple, Color32::GREEN, scale);

                    // draw snake
                    for segment in snake {
                        draw_tile(&painter, segment, Color32::RED, scale);
                    }
                }
                GameState::GameOver { score } => {
                    ui.heading(format!("Final length: {score}"));

                    if ui.button("Start New game").clicked() {
                        *self = GameState::new()
                    }
                }
            }
        })
        .response
    }
}

pub fn delay_for_length(length: u32) -> Duration {
    const MAX_LENGTH: u32 = GAME_SIZE as u32 * GAME_SIZE as u32;
    const MAX_DELAY: Duration = Duration::from_millis(250);

    MAX_DELAY - MAX_DELAY * length / MAX_LENGTH
}

pub fn new_snake() -> VecDeque<ObjectCoordinate> {
    vec![
        ObjectCoordinate {
            x: GAME_SIZE / 2,
            y: GAME_SIZE / 2,
        },
        ObjectCoordinate {
            x: GAME_SIZE / 2,
            y: GAME_SIZE / 2 + 1,
        },
        ObjectCoordinate {
            x: GAME_SIZE / 2,
            y: GAME_SIZE / 2 + 2,
        },
    ]
    .into()
}

pub fn generate_apple() -> ObjectCoordinate {
    use rand::distributions::{Distribution, Uniform};

    let range: Uniform<i8> = Uniform::from(0..GAME_SIZE);
    let mut rng = ::rand::thread_rng();

    let x: i8 = range.sample(&mut rng);
    let y: i8 = range.sample(&mut rng);
    ObjectCoordinate { x, y }
}

pub fn wrap_position(mut obj: ObjectCoordinate) -> ObjectCoordinate {
    while obj.x < 0 {
        obj.x += GAME_SIZE;
    }
    if obj.x >= GAME_SIZE {
        obj.x %= GAME_SIZE;
    }

    while obj.y < 0 {
        obj.y += GAME_SIZE;
    }
    if obj.x >= GAME_SIZE {
        obj.y %= GAME_SIZE;
    }
    obj
}

pub fn wall_death(obj: &ObjectCoordinate) -> bool {
    !(0..GAME_SIZE).contains(&obj.x) || !(0..GAME_SIZE).contains(&obj.y)
}

//Controls whether touching the wall should kill or wrap around
const DO_WALL_DEATH: bool = true;

impl GameState {
    pub fn new() -> GameState {
        GameState::GameState {
            // Rotation for the square.
            apple: generate_apple(),
            snake: new_snake(),
            direction: Direction::UP,
            next_step: None,
        }
    }

    pub fn step_time(&mut self) {
        if let GameState::GameState {
            apple,
            snake,
            direction,
            next_step,
        } = self
        {
            let Some(next_step) = next_step else {
                *next_step = Some(Instant::now() + delay_for_length(snake.len() as u32));
                return;
            };

            if *next_step <= Instant::now() {
                //spawn new head in current direction of previous head
                let new_head = if let Some(head) = snake.front() {
                    if !DO_WALL_DEATH {
                        wrap_position(*head + *direction)
                    } else {
                        *head + *direction
                    }
                } else {
                    ObjectCoordinate { x: 0, y: 0 }
                };

                if DO_WALL_DEATH && wall_death(&new_head) {
                    *self = GameState::GameOver { score: snake.len() };
                    return;
                }

                //consume apple
                if apple == &new_head {
                    // generate new apple
                    *apple = generate_apple();
                } else {
                    // no apple consumed remove last tail
                    snake.pop_back();
                }

                //check if new_head collides with old tile
                if snake.iter().any(|tile| tile == &new_head) {
                    // +1 as tail has already been removed or apple eaten but new_head not yet added
                    *self = GameState::GameOver {
                        score: snake.len() + 1,
                    };
                    return;
                }

                //add new head
                snake.push_front(new_head);

                *next_step = Instant::now() + delay_for_length(snake.len() as u32);
            }
        }
    }
}

pub const TILE_PADDING: f32 = 2.0;
pub const TILE_SIZE: f32 = 10.0;
pub const GAME_SIZE: i8 = 32;
