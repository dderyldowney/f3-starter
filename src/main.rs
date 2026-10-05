#![no_std]
#![no_main]

mod leds;
mod sensors;
mod time;

use cortex_m_rt::entry;
use defmt_rtt as _;
use panic_probe as _;
use stm32f3xx_hal::{i2c::I2c, pac, prelude::*};

use crate::leds::Leds;
use crate::sensors::Lsm303;

#[entry]
fn main() -> ! {
    let dp = pac::Peripherals::take().unwrap();
    let cp = cortex_m::Peripherals::take().unwrap();

    let mut rcc = dp.RCC.constrain();
    let mut flash = dp.FLASH.constrain();
    let clocks = rcc.cfgr.freeze(&mut flash.acr);

    // 1. Initialize monotonic timer and delay runtime
    time::init(cp.SYST, clocks.hclk().0);
    defmt::info!("System booted. Clocks and SysTick initialized.");

    // 2. Configure I2C1 (PB6 = SCL, PB7 = SDA)
    let mut gpiob = dp.GPIOB.split(&mut rcc.ahb);
    let scl = gpiob
        .pb6
        .into_af_open_drain(&mut gpiob.moder, &mut gpiob.otyper, &mut gpiob.afrl);
    let sda = gpiob
        .pb7
        .into_af_open_drain(&mut gpiob.moder, &mut gpiob.otyper, &mut gpiob.afrl);

    let i2c = I2c::new(
        dp.I2C1,
        (scl, sda),
        400_000.Hz(),
        clocks,
        &mut rcc.apb1,
    );
    defmt::info!("I2C1 initialized at 400 kHz.");

    // 3. Initialize accelerometer sensor driver
    let mut sensor = match Lsm303::new(i2c) {
        Ok(s) => {
            defmt::info!("LSM303DLHC Accelerometer initialized (100 Hz, XYZ active).");
            s
        }
        Err(_) => {
            defmt::error!("Failed to initialize accelerometer over I2C.");
            loop {
                cortex_m::asm::wfi();
            }
        }
    };

    // 4. Initialize Port E LEDs
    let gpioe = dp.GPIOE.split(&mut rcc.ahb);
    let mut leds = Leds::new(gpioe);
    defmt::info!("Spirit level active. Monitoring board orientation...");

    let mut log_tick: u32 = 0;

    // 5. Main 50 Hz control loop
    loop {
        if let Ok(reading) = sensor.read_accel() {
            let direction = reading.to_direction();
            leds.show_direction(direction);

            log_tick += 1;
            if log_tick % 25 == 0 {
                defmt::debug!(
                    "Raw Accel: X={=i16}, Y={=i16}, Z={=i16}",
                    reading.x,
                    reading.y,
                    reading.z
                );
            }
        }

        time::delay_ms(20);
    }
}
