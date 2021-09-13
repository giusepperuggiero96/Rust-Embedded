#![no_std]
#![no_main]

// Esempio dimostrativo gestione della concorrenza a livello di interupt

// 4 - Safe Abstractions, Send and Sync traits
//      Rust ci permette di astrarre le nostre variabili da condividere (intrinsecamente unsafe)
// in interfacce safe da usare ovunque nel nostro programma. Questo design richiede che l'applicazione 
// ci passi un token che ci assicuri di trovarci in una sezione critica, senza dover effettuare il lock in
// loco. Questa garanzia è fornita a tempo di compilazione e non ci sarà quindi overhead a runtime. Il tipo 
// di dato di cui si fa uso in questa astrazione è l'UnsafeCell, uno dei pochi tipi di rust che non implementa
// di default il trait sync e pertanto di default non potrebbe essere condiviso da più thread. Va quindi specificato
// esplicitamente di voler implementare forzatamente questo trait. Questa soluzione è funzionale solo per sistemi mono-core.


    use panic_halt as _;

    use cortex_m::{asm, iprintln, Peripherals};
    use cortex_m_semihosting::hprintln;
    use cortex_m::peripheral::syst::SystClkSource;
    use cortex_m_rt::{entry, exception};

    use core::cell::UnsafeCell;
    use cortex_m::interrupt;

    use stm32f3_discovery;

    struct CSCounter(UnsafeCell<u32>);
    const CS_COUNTER_INIT: CSCounter = CSCounter(UnsafeCell::new(0));

    impl CSCounter {
        pub fn reset(&self, _cs: &interrupt::CriticalSection) {
            // By requiring a CriticalSection be passed in, we know we must
            // be operating inside a CriticalSection, and so can confidently
            // use this unsafe block (required to call UnsafeCell::get).
            unsafe { *self.0.get() = 0 };
        }

        pub fn increment(&self, _cs: &interrupt::CriticalSection) {
            unsafe { *self.0.get() += 1 };
        }

        pub fn get(&self, _cs: &interrupt::CriticalSection) -> u32 {
            unsafe { *self.0.get() }
        }
    }

    unsafe impl Sync for CSCounter {}

    static COUNTER: CSCounter = CS_COUNTER_INIT;

    #[entry]
    fn main() -> ! {

        let mut p = cortex_m::Peripherals::take().unwrap();
        let mut syst = p.SYST;

        // Configurazione di syst per generare una eccezione ogni secondo
        syst.set_clock_source(SystClkSource::Core);
        syst.set_reload(8_000_000); // Frequenza 8Mhz
        syst.enable_counter();
        syst.enable_interrupt();

        let mut last_state = false;

        loop{
            let state = !last_state;

            if state && !last_state {
                interrupt::free(|cs| COUNTER.increment(cs));

            }
            last_state = state;
        }

        }


    #[exception]
    fn SysTick(){
        interrupt::free(|cs| {
            hprintln!("nel main valeva {}", COUNTER.get(cs));
            COUNTER.reset(cs);
            hprintln!("adesso vale {}", COUNTER.get(cs));
            });
    }
