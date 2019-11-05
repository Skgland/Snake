use super::*;

#[derive(Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub enum Direction {
    UP,
    DOWN,
    LEFT,
    RIGHT,
}


#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Serialize, Deserialize)]
pub enum TileTextureIndex {
    Apple,
    Background,
    SnakeHead,
}


#[derive(Debug, Ord, PartialOrd, PartialEq, Eq, Clone, Copy, Deserialize, Serialize)]
pub struct ObjectCoordinate {
    pub x: i64,
    pub y: i64,
}

impl std::ops::Add<Direction> for ObjectCoordinate {
    type Output = ObjectCoordinate;

    fn add(self, other: Direction) -> Self {
        match other {
            Direction::DOWN => { ObjectCoordinate { x: self.x, y: self.y + 1 } }
            Direction::UP => { ObjectCoordinate { x: self.x, y: self.y - 1 } }
            Direction::RIGHT => { ObjectCoordinate { x: self.x + 1, y: self.y } }
            Direction::LEFT => { ObjectCoordinate { x: self.x - 1, y: self.y } }
        }
    }
}