#![no_std]
#![no_main]

// Esempio dimostrativo gestione della concorrenza a livello di interupt

// 2 - Critical Sections
//      Una delle prime soluzioni offerte per le data races sono le critical sections, che in questo contesto vuol dire disabilitare le interruzioni. Le implementazioni sono quindi dipendenti dal micro architecture crate, ma sono praticamente sempre disponibili. Questa soluzione non ci soddisfa appieno dato che siamo comunque costretti a scrivere del codice unsafe nel corpo della nostra applicazione. Vi è inotre una perdita di prestazioni genrale andando a disabilitare le interruzioni.


    use panic_halt as _;

    use cortex_m::{asm, iprintln, Peripherals};
    use cortex_m_semihosting::hprintln;
    use cortex_m::peripheral::syst::SystClkSource;
    use cortex_m_rt::{entry, exception};

    use stm32f3_discovery;

    // Varibile statica globale mut
    static mut COUNTER: u32 = 0;

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

                // Sezione critica
                cortex_m::interrupt::free(|_| {
                    unsafe { COUNTER += 1 };
                });

            }
            last_state = state;
        }

        }


    #[exception]
    fn SysTick(){
        // Qui non è necessaria una sezione critica per due ragioni
        // 1 - Scrivere 0 (quindi resettare un registro) non può essere affetto da race condition, dato che non è prevista una lettura
        // 2 - L'handler non può essere interrotto in nessun caso dal main (ma può essere interrotto da altri handler a priorità maggiore).
        unsafe{ hprintln!("nel main valeva {}",COUNTER).unwrap();
        COUNTER = 0;
        hprintln!("ora vale {}",COUNTER).unwrap();
        }
    }
