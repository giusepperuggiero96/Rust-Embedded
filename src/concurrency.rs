#![no_std]
#![no_main]

// Esempio dimostrativo gestione della concorrenza a livello di interupt

// 1 - Global Mutable Data
//      Dato che in rust embedded non abbiamo solitamente accesso a metodi di allocazione dinamica della memoria per passarne riferimenti ai vari thread per condividerla, dobbiamo necessariamente utilizzare memorie statiche globali e mutable. In rust le operazioni su queste variabili sono intrinsecamente unsafe dato che possono generare race condition. L'utilizzo di questa soluzione per la gestione di memoria condivisa tra l'applicazione e gli handler delle interruzioni può essere senza dubbio porta d'ingresso per diversi tipi di exploit (e.g. se un attaccante ha la possibilità di innescare intrruzioni o eccezioni ripetutamente potrebbe portarci forzatamente in una RC).


    use panic_halt as _; 

    use cortex_m::{asm, iprintln, Peripherals};
    use cortex_m::peripheral::syst::SystClkSource;
    use cortex_m_rt::{entry, exception};

    // Varibile statica globale mut
    static mut COUNTER: u32 = 0;

    #[entry]
    fn main -> ! {

        let mut p = cortex_m::Peripherals::take().unwrap();
        let mut syst = p.SYST;

        // Configurazione di syst per generare una eccezione ogni secondo
        syst.set_clock_source(SystClkSource::Core);
        syst.set_reload(8_000_000); // Frequenza 8Mhz
        syst.enable_counter();
        syst.enable_interrupt();

        let last_state = false;

        loop{
            let state = !last_state;

            if state && !last_state {
                // POSSIBLE DATA RACE
                unsafe { COUNTER += 1 };
            }
            last_state = state;
        }

        }


    #[exception]
    fn SysTick(){
        unsafe{ COUNTER = 0; }
    }


// 2 - Critical Sections
//      Una delle prime soluzioni offerte per le data races sono le critical sections, che in questo contesto vuol dire disabilitare le interruzioni. Le implementazioni sono quindi dipendenti dal micro architecture crate, ma sono praticamente sempre disponibili. Questa soluzione non ci soddisfa appieno dato che siamo comunque costretti a scrivere del codice unsafe nel corpo della nostra applicazione. Vi è inotre una perdita di prestazioni genrale andando a disabilitare le interruzioni.

/*
    use panic_halt as _; 

    use cortex_m::{asm, iprintln, Peripherals};
    use cortex_m::peripheral::syst::SystClkSource;
    use cortex_m_rt::{entry, exception};

    // Varibile statica globale mut
    static mut COUNTER: u32 = 0;

    #[entry]
    fn main -> ! {

        let mut p = cortex_m::Peripherals::take().unwrap();
        let mut syst = p.SYST;

        // Configurazione di syst per generare una eccezione ogni secondo
        syst.set_clock_source(SystClkSource::Core);
        syst.set_reload(8_000_000); // Frequenza 8Mhz
        syst.enable_counter();
        syst.enable_interrupt();

        let last_state = false;

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
        unsafe{ COUNTER = 0; }
    }
*/

// 3 - Atomic Access
//      Un'altra soluzione platform specific è quella degli accessi atomici alle variabili. Invece che disabilitare le interruzioni tout court ci garantisce l'esecuzione di operazioni di read-modify-write. Non è però supportata da tutti i tipi e tutte le piattaforme.

/*
    use panic_halt as _; 

    use cortex_m::{asm, iprintln, Peripherals};
    use cortex_m::peripheral::syst::SystClkSource;
    use cortex_m_rt::{entry, exception};

    use core::sync::atomic::{AtomicUsize, Ordering}

    // Variabile atomica globale
    static COUNTER: AtomicUsize = AtomicUsize::new(0);

    #[entry]
    fn main -> ! {

        let mut p = cortex_m::Peripherals::take().unwrap();
        let mut syst = p.SYST;

        // Configurazione di syst per generare una eccezione ogni secondo
        syst.set_clock_source(SystClkSource::Core);
        syst.set_reload(8_000_000); // Frequenza 8Mhz
        syst.enable_counter();
        syst.enable_interrupt();

        let last_state = false;

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
        COUNTER.store(0, Ordering::Relaxed);
    }
*/


// 4 - Safe Abstractions, Send and Sync traits
//      Rust ci permette di astrarre le nostre variabili da condividere (intrinsecamente unsafe) in interfacce safe da usare ovunque nel nostro programma. Questo design richiede che l'applicazione ci passi un token che ci assicuri di trovarci in una sezione critica, senza dover effettuare il lock in loco. Questa garanzia è fornita a tempo di compilazione e non ci sarà quindi overhead a runtime. Il tipo di dato di cui si fa uso in questa astrazione è l'UnsafeCell, uno dei pochi tipi di rust che non implementa di default il trait sync e pertanto di default non potrebbe essere condiviso da più thread. Va quindi specificato esplicitamente di voler implementare forzatamente questo trait. Questa soluzione è funzionale solo per sistemi mono-core.

/*
    use panic_halt as _;

    use cortex_m::{asm, iprintln, Peripherals};
    use cortex_m::peripheral::syst::SystClkSource;
    use cortex_m_rt::{entry, exception};
    use core::cell::UnsafeCell;
    //use cortex_m::interrupt;

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
    }

    unsafe impl Sync for CSCounter {}

    static COUNTER: CSCounter = CS_COUNTER_INIT;

    #[entry]
    fn main -> ! {

        let mut p = cortex_m::Peripherals::take().unwrap();
        let mut syst = p.SYST;

        // Configurazione di syst per generare una eccezione ogni secondo
        syst.set_clock_source(SystClkSource::Core);
        syst.set_reload(8_000_000); // Frequenza 8Mhz
        syst.enable_counter();
        syst.enable_interrupt();

        let last_state = false;

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
        interrupt::free(|cs| COUNTER.reset(cs));
    }
*/


// 5 - Mutexes (Copy trait variables, mutable and immutable peripherals)
//      Un mutex in generale assicura accesso esclusivo a una variabile. In generale l'utilizzo dei mutex per la sincronizzazione con gli interrupt handler non è accettato perchè potrebbe facilmente portare a situazioni di deadlock. Se infatti l'handler fosse in attesa di un lock da parte del main, non ci sarebbe modo per quest'ultimo di rilasciarglielo, dato che non eseguirebbe mai. Per evitare ciò si utilizza un mutex che richiede una sezione critica per bloccarsi. Tale sezione critica deve durare almeno tanto a lungo quanto il lock. I tipi accettati dal mutex in generale sono Cell e RefCell, che forniscono implementazioni safe della interiot mutability. Il tipo Cell è utilizzato per variabili che implementano il tratto Copy (tipi semplici), mentre per tipi composti (come ad esempio una periferica) si utilizza la RefCell, che a runtime riesce a controllare che venga fornito un solo riferimento per la periferica in questione.

/*
    use panic_halt as _;

    use cortex_m::{asm, iprintln, Peripherals};
    use cortex_m::peripheral::syst::SystClkSource;
    use cortex_m_rt::{entry, exception};
    use core::cell::RefCell;
    use core::ops::DerefMut;
    use cortex_m::interrupt;
    use stm32f3xx_hal::{
        prelude::*,
        stm32,
        timer::{Event, Timer},
    };

    static LED_STATE: Mutex<Cell<bool>> = Mutex::new(Cell::new(false));
    static TIM: Mutex<RefCell<Option<Timer<stm32::TIM2>>>> = Mutex::new(RefCell::new(None));

    #[entry]
    fn main -> ! {

        let p = stm32::Peripherals::take().unwrap();

        //setup system clock
        let mut flash = p.FLASH.constrain();
        let mut rcc = p.RCC.constrain();
        let clocks = rcc.cfgr.freeze(&mut flash.acr);

        // Set up the LED
        let mut gpioe = dp.GPIOE.split(&mut rcc.ahb);
        let mut led = gpioe
            .pe9
            .into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper);

        let mut timer = Timer::tim2(p.TIM2, 5.hz(), clocks, &mut rcc.apb1);
        timer.listen(Event::Update);

        free(|cs| {
            TIM.borrow(cs).replace(Some(timer));
        });

        stm32::NVIC::unpend(Interrupt::TIM2);
        unsafe {
            stm32::NVIC::unmask(Interrupt::TIM2);
        }

        loop{
            if free(|cs| LED_STATE.borrow(cs).get()) {
                led.set_high().unwrap();
            } else {
                led.set_low().unwrap();
            }
        }

    }

    #[interrupt]
    fn TIM2() {
        free(|cs| {
            if let Some(ref mut tim2) = TIMER_TIM2.borrow(cs).borrow_mut().deref_mut() { tim2.clear_update_interrupt_flag();
            }
            let led_state = LED_STATE.borrow(cs);
            led_state.replace(!led_state.get());
    });
}

*/
