use crate::leds::Direction;
use stm32f3xx_hal::hal::blocking::i2c::{Write, WriteRead};

/// Raw 12-bit accelerometer readings in counts
#[derive(Copy, Clone, Debug)]
pub struct AccelReading {
    pub x: i16,
    pub y: i16,
    pub z: i16,
}

impl AccelReading {
    /// Classifies tilt vector into an 8-octant compass direction using integer ratios
    pub fn to_direction(&self) -> Option<Direction> {
        let abs_x = self.x.abs() as i32;
        let abs_y = self.y.abs() as i32;
        let mag_sq = (self.x as i32) * (self.x as i32) + (self.y as i32) * (self.y as i32);

        // Deadzone threshold (~150 LSBs ≈ ±10-15° tilt)
        if mag_sq < (150 * 150) {
            None
        } else if abs_y * 1000 < abs_x * 414 {
            // Predominantly East / West
            if self.x > 0 {
                Some(Direction::East)
            } else {
                Some(Direction::West)
            }
        } else if abs_x * 1000 < abs_y * 414 {
            // Predominantly North / South
            if self.y > 0 {
                Some(Direction::North)
            } else {
                Some(Direction::South)
            }
        } else {
            // Intercardinal diagonals
            match (self.x > 0, self.y > 0) {
                (true, true) => Some(Direction::NorthEast),
                (false, true) => Some(Direction::NorthWest),
                (true, false) => Some(Direction::SouthEast),
                (false, false) => Some(Direction::SouthWest),
            }
        }
    }
}

/// Generic LSM303DLHC accelerometer driver
pub struct Lsm303<I2C> {
    i2c: I2C,
}

impl<I2C, E> Lsm303<I2C>
where
    I2C: Write<Error = E> + WriteRead<Error = E>,
{
    pub const ACCEL_ADDR: u8 = 0x19;
    pub const CTRL_REG1_A: u8 = 0x20;
    pub const OUT_X_L_A_AUTO_INC: u8 = 0x28 | 0x80;

    /// Initialize sensor: 100 Hz output data rate, normal power mode, XYZ enabled
    pub fn new(mut i2c: I2C) -> Result<Self, E> {
        i2c.write(Self::ACCEL_ADDR, &[Self::CTRL_REG1_A, 0x57])?;
        Ok(Self { i2c })
    }

    /// Read raw 6-byte burst and convert into 16-bit signed values
    pub fn read_accel(&mut self) -> Result<AccelReading, E> {
        let mut raw = [0u8; 6];
        self.i2c
            .write_read(Self::ACCEL_ADDR, &[Self::OUT_X_L_A_AUTO_INC], &mut raw)?;

        let x = i16::from_le_bytes([raw[0], raw[1]]) >> 4;
        let y = i16::from_le_bytes([raw[2], raw[3]]) >> 4;
        let z = i16::from_le_bytes([raw[4], raw[5]]) >> 4;

        Ok(AccelReading { x, y, z })
    }
}
