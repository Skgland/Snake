use std::num::NonZeroU32;

use graphics::{
    Context,
    Graphics,
    rectangle,
};
use piston_window::Transformed;
use serde::{
    Serialize,
    Deserialize,
};

pub use level::*;
use crate::TextureMap;

pub mod level;
pub mod color;


#[derive(Copy, Clone, Debug)]
pub struct SnakeTile {
    time_to_live: NonZeroU32,
    position: ObjectCoordinate,
}

#[derive(Clone, Debug)]
pub enum GameState {
    GameOver {
        score: usize,
    },
    GameState {
        apple: Option<ObjectCoordinate>,
        snake: Vec<SnakeTile>,
        direction: Direction,
        update_till_move: u32,
    },
}

pub fn delay_for_length(length: usize) -> u32 {
    const MAX_LENGTH: f64 = GAME_SIZE as f64 * GAME_SIZE as f64;
    const MAX_DELAY: f64 = 25.0;

    0.0f64.max(MAX_DELAY - length as f64 * MAX_DELAY / MAX_LENGTH) as u32
}

pub fn new_snake() -> Vec<SnakeTile> {
    vec![SnakeTile { time_to_live: NonZeroU32::new(3).unwrap(), position: ObjectCoordinate { x: 0, y: 0 } }
         , SnakeTile { time_to_live: NonZeroU32::new(2).unwrap(), position: ObjectCoordinate { x: 0, y: 1 } }
         , SnakeTile { time_to_live: NonZeroU32::new(1).unwrap(), position: ObjectCoordinate { x: 0, y: 2 } }
    ]
}

pub fn generate_apple() -> ObjectCoordinate {
    use rand::distributions::{Distribution, Uniform};

    let range: Uniform<i8> = Uniform::from(-GAME_SIZE..GAME_SIZE);
    let mut rng = ::rand::thread_rng();

    let x: i8 = range.sample(&mut rng);
    let y: i8 = range.sample(&mut rng);
    ObjectCoordinate { x: x as i64, y: y as i64 }
}

pub fn wrap_position(mut obj: ObjectCoordinate) -> ObjectCoordinate {
    if obj.x > (GAME_SIZE as i64) - 1 {
        obj.x = -GAME_SIZE as i64
    } else if obj.x < (-GAME_SIZE as i64) {
        obj.x = (GAME_SIZE as i64) - 1
    }
    if obj.y > (GAME_SIZE as i64) - 1 {
        obj.y = -GAME_SIZE as i64
    } else if obj.y < (-GAME_SIZE as i64) {
        obj.y = (GAME_SIZE as i64) - 1
    }
    obj
}

pub fn wall_death(obj : &ObjectCoordinate) -> bool {
    obj.x > (GAME_SIZE as i64) - 1 || obj.x < (-GAME_SIZE as i64) || obj.y > (GAME_SIZE as i64) - 1 || obj.y < (-GAME_SIZE as i64)
}

//Controls whether touching the wall should kill or wrap around
const DO_WALL_DEATH: bool = true;

impl GameState {
    pub fn new() -> GameState {
        GameState::GameState {
            // Rotation for the square.
            apple: None,
            snake: new_snake(),
            direction: Direction::UP,
            update_till_move: delay_for_length(3),
        }
    }

    //during update
    pub fn handle_input(&mut self) {
        if let GameState::GameState { apple, snake, direction, update_till_move } = self {
            if *update_till_move == 0 {

                //spawn new head in current direction of previous head
                let new_head = if let Some(head) = snake.get(0) {
                    SnakeTile {
                        time_to_live: NonZeroU32::new(snake.len() as u32 + 1).unwrap(),
                        position: if !DO_WALL_DEATH { wrap_position(head.position + *direction) } else { head.position + *direction },

                    }
                } else {
                    SnakeTile {
                        time_to_live: NonZeroU32::new(1).unwrap(),
                        position: ObjectCoordinate {
                            x: 0,
                            y: 0,
                        },
                    }
                };

                if DO_WALL_DEATH && wall_death(&new_head.position) {
                    *self = GameState::GameOver { score: snake.len() };
                    return;
                }

                //consume apple and increment ttl to add new tile
                if apple.map(|c| c == new_head.position).unwrap_or(false) {
                    snake.iter_mut().for_each(|tile| tile.time_to_live = NonZeroU32::new(tile.time_to_live.get() + 1).unwrap());
                    *apple = None;
                }


                //remove dieing element
                snake.retain(|e| e.time_to_live.get() > 1);

                //check if new_head collides with old tile
                if snake.iter().any(|tile| tile.position == new_head.position) {
                    // +1 as tail has already been removed or apple eaten but new_head not yet added
                    *self = GameState::GameOver { score: snake.len() +1 };
                    return;
                }

                //add new head
                snake.insert(0, new_head);

                //decrease all ttl
                for tile in snake.iter_mut() {
                    //ttl 1 tiles should have been deleted
                    tile.time_to_live = NonZeroU32::new(tile.time_to_live.get() - 1).unwrap();
                }

                if let None = apple {
                    *apple = Some(generate_apple())
                }

                *update_till_move = delay_for_length(snake.len());
            } else {
                *update_till_move -= 1;
            }
        }
    }

    #[allow(unused_variables)]
    pub fn draw_player<G: Graphics>(&self, context: Context, gl: &mut G, texture_map: &TextureMap<G>) {
        if let GameState::GameState { snake, .. } = self {
            for tile in snake {
                let transform = context.trans(-PLAYER_SIZE / 2.0, -PLAYER_SIZE / 2.0).trans(TILE_SIZE * (tile.position.x as f64), TILE_SIZE * (tile.position.y as f64)).transform;

                rectangle(PLAYER_COLOR, PLAYER_SQUARE, transform, gl);
            }
        }
    }
}


pub const PLAYER_SQUARE: graphics::types::Rectangle = [0.0, 0.0, PLAYER_SIZE, PLAYER_SIZE];
pub const TILE_SIZE: f64 = 12.0;
pub const PLAYER_SIZE: f64 = 10.0;
pub const PLAYER_COLOR: color::Color = color::RED;
pub const APPLE_COLOR: color::Color = color::GREEN;
pub const GAME_SIZE: i8 = 16;
