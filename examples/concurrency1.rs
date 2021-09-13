#![no_std]
#![no_main]

// Esempio dimostrativo gestione della concorrenza a livello di interupt

// 1 - Global Mutable Data
//      Dato che in rust embedded non abbiamo solitamente accesso a metodi di allocazione dinamica della memoria per passarne riferimenti ai vari thread per condividerla, dobbiamo necessariamente utilizzare memorie statiche globali e mutable. In rust le operazioni su queste variabili sono intrinsecamente unsafe dato che possono generare race condition. L'utilizzo di questa soluzione per la gestione di memoria condivisa tra l'applicazione e gli handler delle interruzioni può essere senza dubbio porta d'ingresso per diversi tipi di exploit (e.g. se un attaccante ha la possibilità di innescare intrruzioni o eccezioni ripetutamente potrebbe portarci forzatamente in una RC).


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
                // POSSIBLE DATA RACE
                unsafe { COUNTER += 1};
            }
            last_state = state;
        }

        }


    #[exception]
    fn SysTick(){
        unsafe{ hprintln!("nel main valeva {}",COUNTER).unwrap();
        COUNTER = 0;
        hprintln!("ora vale {}",COUNTER).unwrap();
        }
    }
