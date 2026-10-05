#![no_std]
#![no_main]

use cortex_m_rt::entry;
use defmt_rtt as _;
use panic_probe as _;
use stm32f3xx_hal as _; // Links the STM32F3 device interrupt vector table

// Minimal timestamp symbol required by defmt
defmt::timestamp!("{=u32}", 0);

#[entry]
fn main() -> ! {
    defmt::info!("ST-LINK connected. Ready to start the book!");

    loop {
        cortex_m::asm::nop();
    }
}
