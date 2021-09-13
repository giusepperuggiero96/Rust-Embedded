#![no_std]
#![no_main]

// Esempio dimostrativo gestione della concorrenza a livello di interupt

// 5 - Mutexes (Copy trait variables, mutable and immutable peripherals)
//      Un mutex in generale assicura accesso esclusivo a una variabile. 
// In generale l'utilizzo dei mutex per la sincronizzazione con gli interrupt
// handler non è accettato perchè potrebbe facilmente portare a situazioni di 
// deadlock. Se infatti l'handler fosse in attesa di un lock da parte del main,
// non ci sarebbe modo per quest'ultimo di rilasciarglielo, dato che non eseguirebbe 
// mai. Per evitare ciò si utilizza un mutex che richiede una sezione critica per 
// bloccarsi. Tale sezione critica deve durare almeno tanto a lungo quanto il lock.
// I tipi accettati dal mutex in generale sono Cell e RefCell, che forniscono 
// implementazioni safe della interiot mutability. Il tipo Cell è utilizzato per
// variabili che implementano il tratto Copy (tipi semplici), mentre per tipi composti 
// (come ad esempio una periferica) si utilizza la RefCell, che a runtime riesce a
// controllare che venga fornito un solo riferimento per la periferica in questione.

    use panic_halt as _;

    use cortex_m::{asm, iprintln, Peripherals};
    use cortex_m::peripheral::syst::SystClkSource;
    use cortex_m_rt::{entry, exception};
    use core::cell::RefCell;
    use core::cell::Cell;
    use core::ops::DerefMut;
    use cortex_m::interrupt::*;
    use stm32f3xx_hal::{
        prelude::*,
        pac::*,
        pac,
        timer::{Event, Timer},
    };

   static LED_STATE: Mutex<Cell<bool>> = Mutex::new(Cell::new(false));
    static TIM: Mutex<RefCell<Option<Timer<pac::TIM2>>>> = Mutex::new(RefCell::new(None));

    #[entry]
    fn main() -> ! {

        let p = pac::Peripherals::take().unwrap();

        //setup system clock
        let mut flash = p.FLASH.constrain();
        let mut rcc = p.RCC.constrain();
        let clocks = rcc.cfgr.freeze(&mut flash.acr);

        // Set up the LED
        let mut gpioe = p.GPIOE.split(&mut rcc.ahb);
        let mut led = gpioe
            .pe9
            .into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper);

        let mut timer = Timer::tim2(p.TIM2, 5.Hz(), clocks, &mut rcc.apb1);
        timer.listen(Event::Update);

        free(|cs| {
            TIM.borrow(cs).replace(Some(timer));
        });

        pac::NVIC::unpend(interrupt::TIM2);
        unsafe {
            pac::NVIC::unmask(interrupt::TIM2);
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
            if let Some(ref mut tim2) = TIM.borrow(cs).borrow_mut().deref_mut() { tim2.clear_update_interrupt_flag();
            }
            let led_state = LED_STATE.borrow(cs);
            led_state.replace(!led_state.get());
    });
    }
