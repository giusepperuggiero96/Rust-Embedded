#![no_std]
#![no_main]

// Esempio dimostrativo gestione della concorrenza a livello di interupt

// 3 - Atomic Access
//      Un'altra soluzione platform specific è quella degli accessi atomici alle variabili. Invece che disabilitare le interruzioni tout court ci garantisce l'esecuzione di operazioni di read-modify-write. Non è però supportata da tutti i tipi e tutte le piattaforme.


    use panic_halt as _;

    use cortex_m::{asm, iprintln, Peripherals};
    use cortex_m_semihosting::hprintln;
    use cortex_m::peripheral::syst::SystClkSource;
    use cortex_m_rt::{entry, exception};

    use core::sync::atomic::{AtomicUsize, Ordering};

    use stm32f3_discovery;

    // Variabile atomica globale
    static COUNTER: AtomicUsize = AtomicUsize::new(0);

    #[entry]
    fn main() -> ! {

        let p = cortex_m::Peripherals::take().unwrap();
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
                // Incremento atomico
                COUNTER.fetch_add(1, Ordering::Relaxed);
            }
            last_state = state;
        }

        }


    #[exception]
    fn SysTick(){
        hprintln!("nel main valeva {}", COUNTER.load(Ordering::Relaxed));
        COUNTER.store(0, Ordering::Relaxed);
        hprintln!("adesso vale {}", COUNTER.load(Ordering::Relaxed));

    }
