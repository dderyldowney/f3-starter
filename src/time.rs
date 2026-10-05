use core::sync::atomic::{AtomicU32, Ordering};
use cortex_m::peripheral::syst::SystClkSource;
use cortex_m::peripheral::SYST;
use cortex_m_rt::exception;

static MILLIS: AtomicU32 = AtomicU32::new(0);

// Provides hardware millisecond timestamps to defmt logs across the crate
defmt::timestamp!("{=u32:ms}", MILLIS.load(Ordering::Relaxed));

/// SysTick interrupt handler firing every 1 ms
#[exception]
fn SysTick() {
    MILLIS.fetch_add(1, Ordering::Relaxed);
}

/// Initialize SysTick to trigger a periodic 1 ms interrupt
pub fn init(mut syst: SYST, hclk_hz: u32) {
    syst.set_clock_source(SystClkSource::Core);
    syst.set_reload(hclk_hz / 1000 - 1);
    syst.clear_current();
    syst.enable_interrupt();
    syst.enable_counter();
}

/// Returns elapsed milliseconds since boot
pub fn millis() -> u32 {
    MILLIS.load(Ordering::Relaxed)
}

/// Low-power blocking delay using Wait-For-Interrupt (WFI)
pub fn delay_ms(ms: u32) {
    let start = millis();
    while millis().wrapping_sub(start) < ms {
        cortex_m::asm::wfi();
    }
}
