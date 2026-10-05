use stm32f3xx_hal::gpio::{gpioe, Output, PushPull};
use stm32f3xx_hal::prelude::*;

/// Cardinal and intercardinal compass directions for the LED ring
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Direction {
    North,
    NorthEast,
    East,
    SouthEast,
    South,
    SouthWest,
    West,
    NorthWest,
}

/// Circular LED ring driver on GPIO Port E
pub struct Leds {
    ld3: gpioe::PE9<Output<PushPull>>,   // North (Red)
    ld5: gpioe::PE10<Output<PushPull>>,  // North-East (Orange)
    ld7: gpioe::PE11<Output<PushPull>>,  // East (Green)
    ld9: gpioe::PE12<Output<PushPull>>,  // South-East (Blue)
    ld10: gpioe::PE13<Output<PushPull>>, // South (Red)
    ld8: gpioe::PE14<Output<PushPull>>,  // South-West (Orange)
    ld6: gpioe::PE15<Output<PushPull>>,  // West (Green)
    ld4: gpioe::PE8<Output<PushPull>>,   // North-West (Blue)
}

impl Leds {
    /// Configure Port E pins into push-pull outputs
    pub fn new(mut gpioe: gpioe::Parts) -> Self {
        Self {
            ld3: gpioe.pe9.into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper),
            ld5: gpioe.pe10.into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper),
            ld7: gpioe.pe11.into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper),
            ld9: gpioe.pe12.into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper),
            ld10: gpioe.pe13.into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper),
            ld8: gpioe.pe14.into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper),
            ld6: gpioe.pe15.into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper),
            ld4: gpioe.pe8.into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper),
        }
    }

    /// Turn off all 8 LEDs
    pub fn turn_all_off(&mut self) {
        self.ld3.set_low().unwrap();
        self.ld5.set_low().unwrap();
        self.ld7.set_low().unwrap();
        self.ld9.set_low().unwrap();
        self.ld10.set_low().unwrap();
        self.ld8.set_low().unwrap();
        self.ld6.set_low().unwrap();
        self.ld4.set_low().unwrap();
    }

    /// Illuminate the single LED matching the active tilt direction
    pub fn show_direction(&mut self, direction: Option<Direction>) {
        self.turn_all_off();
        match direction {
            Some(Direction::North) => self.ld3.set_high().unwrap(),
            Some(Direction::NorthEast) => self.ld5.set_high().unwrap(),
            Some(Direction::East) => self.ld7.set_high().unwrap(),
            Some(Direction::SouthEast) => self.ld9.set_high().unwrap(),
            Some(Direction::South) => self.ld10.set_high().unwrap(),
            Some(Direction::SouthWest) => self.ld8.set_high().unwrap(),
            Some(Direction::West) => self.ld6.set_high().unwrap(),
            Some(Direction::NorthWest) => self.ld4.set_high().unwrap(),
            None => {} // Flat: all LEDs remain off
        }
    }
}
