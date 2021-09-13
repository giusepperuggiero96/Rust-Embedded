#![no_std]
#![no_main]

/*
*   Progetto per SSSI
*
*   Cose da mostrare possibili:
*   - Utilizzo in un contesto concreto di un Board Support Package
*   - Setup di interruzioni ed eccezioni, gestendo variabili e periferiche condivise con i meccanismi sicuri di RUST, trait sync e send
*   - Gestione dei panic
*   - Utilizzo di un allocator ed highlight delle differenze nella gestione della memoria dinamica tra rust std e no_std
*   - Static guarantees, gestione delle periferiche con state machine e zero cost abstraction
*   - Nel contesto embedded evidenziare le caratteristiche generali di RUST
*
*/


// pick a panicking behavior
use panic_halt as _; // you can put a breakpoint on `rust_begin_unwind` to catch panics
// use panic_abort as _; // requires nightly
// use panic_itm as _; // logs messages over ITM; requires ITM support
// use panic_semihosting as _; // logs messages to the host stderr; requires a debugger

use cortex_m::{asm, iprintln, Peripherals};
use cortex_m_semihosting::hprintln;
use cortex_m::peripheral::syst::SystClkSource;
use cortex_m_rt::{entry, exception};

use stm32f3_discovery::wait_for_interrupt;
use stm32f3_discovery::stm32f3xx_hal::prelude::*;
use stm32f3_discovery::stm32f3xx_hal::pac;
use stm32f3_discovery::stm32f3xx_hal::interrupt;
use stm32f3_discovery::button;
use stm32f3_discovery::button::interrupt::TriggerMode;
use stm32f3_discovery::leds::Leds;
use stm32f3_discovery::switch_hal::ToggleableOutputSwitch;

use core::sync::atomic::{AtomicBool, Ordering};
// use stm32f3_discovery::compass::Compass;

// use accelerometer::{Accelerometer, RawAccelerometer};


// Variabile atomica statica
static USER_BUTTON_PRESSED: AtomicBool = AtomicBool::new(false);


#[entry]
fn main() -> ! {
    // asm::nop(); // To not have main optimize to abort in release mode, remove when you add code

    // Configurazione interrupt SysTick usando il Micro-Architecture Crate
    let mut p = cortex_m::Peripherals::take().unwrap();
    let mut syst = p.SYST;

    // Configurazione di syst per generare una eccezione ogni secondo
    syst.set_clock_source(SystClkSource::Core);
    syst.set_reload(8_000_000); // Frequenza 8Mhz
    syst.enable_counter();
    syst.enable_interrupt();

    // Apertura di un canale ITM
    let stim = &mut p.ITM.stim[0];

    // Configurazione delle periferiche del device
    let dp = pac::Peripherals::take().unwrap();
    let mut rcc = dp.RCC.constrain();

    // Inizializzazione dei LED
    let mut gpioe = dp.GPIOE.split(&mut rcc.ahb);
    let leds = Leds::new(
        gpioe.pe8,
        gpioe.pe9,
        gpioe.pe10,
        gpioe.pe11,
        gpioe.pe12,
        gpioe.pe13,
        gpioe.pe14,
        gpioe.pe15,
        &mut gpioe.moder,
        &mut gpioe.otyper,
        );
    let mut status_led = leds.ld3;

    // Abilito l'interrupt del bottone
    button::interrupt::enable(
        &dp.EXTI,
        &dp.SYSCFG,
        TriggerMode::Rising,
        );


    loop {
    
        // Application code
        hprintln!("Buonasera").unwrap();

        if USER_BUTTON_PRESSED.swap(false, Ordering::AcqRel){
            iprintln!(stim, "Buonasera but faster");
            status_led.toggle().ok();
        }

        wait_for_interrupt();
        //considerazione: perche' stiamo inserendo le print dentro al main loop anziche' dentro all'interrupt handler?
        //il problema è che bisogna trovare un modo per riferirsi in due contesti diversi (main, exception) alla stessa variabile (p che possiede stim). Le soluzioni impiegabili sono 3:
        //1) passare un riferimento a stim ogni volta che ci si trova in un contesto diverso. Per farlo si può utilizzare la funzione Peripherals::steal() che però è unsafe e quindi può portare a race conditions
        //2) si può rendere stim static mut, in modo che sia condivisa globalmente, tuttavia si hanno gli stessi problemi di race condition della prima soluzione
        //3) ci si può affidare ad un framework costruito apposta per la gestione della concorrenza nei microcontrollori, come RTIC, di cui esiste già la versione per cortex_m: https://rtic.rs/0.5/book/en/preface.html
    }
}

#[exception]
fn SysTick() {
    cortex_m::asm::nop();
}

#[interrupt]
fn EXTI0() {
    // Interrupt clear
    button::interrupt::clear();
    // Setto la variabile atomica a true
    USER_BUTTON_PRESSED.store(true, Ordering::Relaxed);
}
