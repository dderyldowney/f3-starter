#![no_std]
#![no_main]

use core::sync::atomic::{AtomicU32, Ordering};
use cortex_m::peripheral::syst::SystClkSource;
use cortex_m_rt::{entry, exception};
use defmt_rtt as _;
use panic_probe as _;
use stm32f3xx_hal::{i2c::I2c, pac, prelude::*};

static MILLIS: AtomicU32 = AtomicU32::new(0);

defmt::timestamp!("{=u32:ms}", MILLIS.load(Ordering::Relaxed));

#[exception]
fn SysTick() {
    MILLIS.fetch_add(1, Ordering::Relaxed);
}

fn delay_ms(ms: u32) {
    let start = MILLIS.load(Ordering::Relaxed);
    while MILLIS.load(Ordering::Relaxed).wrapping_sub(start) < ms {
        cortex_m::asm::wfi();
    }
}

/// Compass direction representations for the circular LED ring
#[derive(Copy, Clone)]
enum Direction {
    North,
    NorthEast,
    East,
    SouthEast,
    South,
    SouthWest,
    West,
    NorthWest,
}

#[entry]
fn main() -> ! {
    let dp = pac::Peripherals::take().unwrap();
    let cp = cortex_m::Peripherals::take().unwrap();

    let mut rcc = dp.RCC.constrain();
    let mut flash = dp.FLASH.constrain();
    let clocks = rcc.cfgr.freeze(&mut flash.acr);

    // 1. SysTick configuration (1 ms tick)
    let mut syst = cp.SYST;
    syst.set_clock_source(SystClkSource::Core);
    let hclk_hz = clocks.hclk().0;
    syst.set_reload(hclk_hz / 1000 - 1);
    syst.clear_current();
    syst.enable_interrupt();
    syst.enable_counter();

    defmt::info!("System booted. Clocks and SysTick initialized.");

    // 2. Configure GPIOB pins for I2C1 (PB6 = SCL, PB7 = SDA)
    let mut gpiob = dp.GPIOB.split(&mut rcc.ahb);
    let scl = gpiob
        .pb6
        .into_af_open_drain(&mut gpiob.moder, &mut gpiob.otyper, &mut gpiob.afrl);
    let sda = gpiob
        .pb7
        .into_af_open_drain(&mut gpiob.moder, &mut gpiob.otyper, &mut gpiob.afrl);

    let mut i2c = I2c::new(
        dp.I2C1,
        (scl, sda),
        400_000.Hz(),
        clocks,
        &mut rcc.apb1,
    );
    defmt::info!("I2C1 initialized at 400 kHz.");

    // 3. Step 1: Activate the LSM303 Accelerometer (Address: 0x19)
    // Write 0x57 to CTRL_REG1_A (0x20): 100 Hz ODR, Normal Power, X/Y/Z enabled
    match i2c.write(0x19, &[0x20, 0x57]) {
        Ok(_) => defmt::info!("LSM303DLHC Accelerometer initialized (100 Hz, XYZ active)."),
        Err(_) => defmt::error!("Failed to initialize accelerometer over I2C."),
    }

    // 4. Configure GPIO Port E for all 8 circular LEDs
    let mut gpioe = dp.GPIOE.split(&mut rcc.ahb);
    let mut ld3 = gpioe.pe9.into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper);  // N  (Red)
    let mut ld5 = gpioe.pe10.into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper); // NE (Orange)
    let mut ld7 = gpioe.pe11.into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper); // E  (Green)
    let mut ld9 = gpioe.pe12.into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper); // SE (Blue)
    let mut ld10 = gpioe.pe13.into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper); // S  (Red)
    let mut ld8 = gpioe.pe14.into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper); // SW (Orange)
    let mut ld6 = gpioe.pe15.into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper); // W  (Green)
    let mut ld4 = gpioe.pe8.into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper);  // NW (Blue)

    defmt::info!("Spirit level active. Monitoring board orientation...");

    let mut log_tick: u32 = 0;
    let mut raw_accel = [0u8; 6];

    loop {
        // Step 2: Read Tri-Axis acceleration via 6-byte burst read (OUT_X_L_A | 0x80)
        if i2c.write_read(0x19, &[0x28 | 0x80], &mut raw_accel).is_ok() {
            let x = i16::from_le_bytes([raw_accel[0], raw_accel[1]]) >> 4;
            let y = i16::from_le_bytes([raw_accel[2], raw_accel[3]]) >> 4;
            let z = i16::from_le_bytes([raw_accel[4], raw_accel[5]]) >> 4;

            // Step 3: Determine tilt direction using integer ratio math
            let abs_x = x.abs() as i32;
            let abs_y = y.abs() as i32;
            let mag_sq = (x as i32) * (x as i32) + (y as i32) * (y as i32);

            // Deadzone: If tilt vector is small (< ~150 LSBs), board is flat
            let active_led = if mag_sq < (150 * 150) {
                None
            } else if abs_y * 1000 < abs_x * 414 {
                // Predominantly East or West
                if x > 0 {
                    Some(Direction::East)
                } else {
                    Some(Direction::West)
                }
            } else if abs_x * 1000 < abs_y * 414 {
                // Predominantly North or South
                if y > 0 {
                    Some(Direction::North)
                } else {
                    Some(Direction::South)
                }
            } else {
                // Diagonal directions
                match (x > 0, y > 0) {
                    (true, true) => Some(Direction::NorthEast),
                    (false, true) => Some(Direction::NorthWest),
                    (true, false) => Some(Direction::SouthEast),
                    (false, false) => Some(Direction::SouthWest),
                }
            };

            // Reset all LEDs to low
            ld3.set_low().unwrap();
            ld5.set_low().unwrap();
            ld7.set_low().unwrap();
            ld9.set_low().unwrap();
            ld10.set_low().unwrap();
            ld8.set_low().unwrap();
            ld6.set_low().unwrap();
            ld4.set_low().unwrap();

            // Light up the single LED matching the tilt heading
            match active_led {
                Some(Direction::North) => ld3.set_high().unwrap(),
                Some(Direction::NorthEast) => ld5.set_high().unwrap(),
                Some(Direction::East) => ld7.set_high().unwrap(),
                Some(Direction::SouthEast) => ld9.set_high().unwrap(),
                Some(Direction::South) => ld10.set_high().unwrap(),
                Some(Direction::SouthWest) => ld8.set_high().unwrap(),
                Some(Direction::West) => ld6.set_high().unwrap(),
                Some(Direction::NorthWest) => ld4.set_high().unwrap(),
                None => {} // Flat: all LEDs remain off
            }

            // Log raw axes at ~2 Hz (every 25 loops @ 20 ms)
            log_tick += 1;
            if log_tick % 25 == 0 {
                defmt::debug!("Raw Accel: X={=i16}, Y={=i16}, Z={=i16}", x, y, z);
            }
        }

        // 50 Hz sampling loop rate
        delay_ms(20);
    }
}
