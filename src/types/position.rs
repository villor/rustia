use crate::constants::Direction;

#[derive(Debug, Default, Copy, Clone)]
pub struct Position {
    pub x: u16,
    pub y: u16,
    pub z: u8,
}

impl Position {
    fn can_see(&self, other: &Position, range_x: u16, range_y: u16) -> bool {
        if (self.z <= 7 && other.z > 7)
        || (self.z >= 8 && self.distance_y(other) > 2) {
            return false;
        }

        let offset_z = self.y - other.y;
        (other.x >= self.x - range_x + offset_z)
        && (other.x <= self.x + range_x + offset_z)
        && (other.y >= self.y - range_y + offset_z)
        && (other.y <= self.y + range_y + offset_z)
    }

    fn distance_x(&self, other: &Position) -> u8 {
        i8::abs(self.x as i8 - other.x as i8) as u8
    }

    fn distance_y(&self, other: &Position) -> u8 {
        i8::abs(self.y as i8 - other.y as i8) as u8
    }

    fn distance_z(&self, other: &Position) -> u8 {
        i8::abs(self.z as i8 - other.z as i8) as u8
    }
}

impl std::ops::Add<Direction> for Position {
    type Output = Self;

    fn add(self, direction: Direction) -> Self::Output {
        match direction {
            Direction::North => Position { y: self.y - 1, ..self },
            Direction::East  => Position { x: self.x + 1, ..self },
            Direction::South => Position { y: self.y + 1, ..self },
            Direction::West  => Position { x: self.x - 1, ..self },
        }
    }
}
