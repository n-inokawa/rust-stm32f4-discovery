#![no_std]
#![no_main]

use core::cell::{Cell, RefCell};

use panic_halt as _; // you can put a breakpoint on `rust_begin_unwind` to catch panics

use cortex_m::{self, asm::delay, interrupt::Mutex};
use cortex_m_rt::entry;
use stm32f4::stm32f407::{self, interrupt};

static P_GPIOD: Mutex<RefCell<Option<stm32f407::GPIOD>>> = Mutex::new(RefCell::new(None));
static P_GPIOA: Mutex<RefCell<Option<stm32f407::GPIOA>>> = Mutex::new(RefCell::new(None));
static P_EXTI: Mutex<RefCell<Option<stm32f407::EXTI>>> = Mutex::new(RefCell::new(None));

static IS_ROULETTE: Mutex<Cell<bool>> = Mutex::new(Cell::new(false));

#[entry]
fn main() -> ! {
    let p = stm32f407::Peripherals::take().unwrap();
    {
        // Setup PD12~PD15 for User leds
        p.RCC.ahb1enr.modify(|_, w| w.gpioden().set_bit());
        let gpiod = &p.GPIOD;
        gpiod.moder.modify(|_, w| w.moder12().output());
        gpiod.moder.modify(|_, w| w.moder13().output());
        gpiod.moder.modify(|_, w| w.moder14().output());
        gpiod.moder.modify(|_, w| w.moder15().output());

        // Setup PA0 for User switch
        p.RCC.ahb1enr.modify(|_, w| w.gpioaen().set_bit());
        let gpioa = &p.GPIOA;
        gpioa.moder.modify(|_, w| w.moder0().input());
        // pull-down
        gpioa.pupdr.modify(|_, w| unsafe { w.pupdr0().bits(0b10) });

        // Setup EXTI0 for PA0
        p.RCC.apb2enr.modify(|_, w| w.syscfgen().set_bit());
        // connect to PA0
        p.SYSCFG
            .exticr1
            .modify(|_, w| unsafe { w.exti0().bits(0b000) });
        // unmask interrupt
        let exti = &p.EXTI;
        exti.imr.modify(|_, w| w.mr0().set_bit());
        // trigger on rising-edge
        exti.rtsr.modify(|_, w| w.tr0().set_bit());
        // trigger on falling-edge
        exti.ftsr.modify(|_, w| w.tr0().set_bit());
        // enable EXTI0 on NVIC
        unsafe {
            cortex_m::peripheral::NVIC::unmask(interrupt::EXTI0);
        }
    }

    // Share peripherals with mutex
    cortex_m::interrupt::free(|cs| {
        P_GPIOD.borrow(cs).replace(Some(p.GPIOD));
        P_GPIOA.borrow(cs).replace(Some(p.GPIOA));
        P_EXTI.borrow(cs).replace(Some(p.EXTI));
    });

    loop {
        let is_roulette = cortex_m::interrupt::free(|cs| IS_ROULETTE.borrow(cs).get());
        if is_roulette {
            cortex_m::interrupt::free(|cs| {
                let gpiod = P_GPIOD.borrow(cs).borrow();
                gpiod.as_ref().unwrap().bsrr.write(|w| w.br15().reset());
                gpiod.as_ref().unwrap().bsrr.write(|w| w.bs12().set_bit());
            });
            delay(2_000_000);
            cortex_m::interrupt::free(|cs| {
                let gpiod = P_GPIOD.borrow(cs).borrow();
                gpiod.as_ref().unwrap().bsrr.write(|w| w.br12().reset());
                gpiod.as_ref().unwrap().bsrr.write(|w| w.bs13().set_bit());
            });
            delay(2_000_000);
            cortex_m::interrupt::free(|cs| {
                let gpiod = P_GPIOD.borrow(cs).borrow();
                gpiod.as_ref().unwrap().bsrr.write(|w| w.br13().reset());
                gpiod.as_ref().unwrap().bsrr.write(|w| w.bs14().set_bit());
            });
            delay(2_000_000);
            cortex_m::interrupt::free(|cs| {
                let gpiod = P_GPIOD.borrow(cs).borrow();
                gpiod.as_ref().unwrap().bsrr.write(|w| w.br14().reset());
                gpiod.as_ref().unwrap().bsrr.write(|w| w.bs15().set_bit());
            });
            delay(2_000_000);
        } else {
            cortex_m::interrupt::free(|cs| {
                let gpiod = P_GPIOD.borrow(cs).borrow();
                gpiod.as_ref().unwrap().bsrr.write(|w| w.bs12().set_bit());
                gpiod.as_ref().unwrap().bsrr.write(|w| w.bs13().set_bit());
                gpiod.as_ref().unwrap().bsrr.write(|w| w.bs14().set_bit());
                gpiod.as_ref().unwrap().bsrr.write(|w| w.bs15().set_bit());
            });
            delay(8_000_000);
            cortex_m::interrupt::free(|cs| {
                let gpiod = P_GPIOD.borrow(cs).borrow();
                gpiod.as_ref().unwrap().bsrr.write(|w| w.br12().reset());
                gpiod.as_ref().unwrap().bsrr.write(|w| w.br13().reset());
                gpiod.as_ref().unwrap().bsrr.write(|w| w.br14().reset());
                gpiod.as_ref().unwrap().bsrr.write(|w| w.br15().reset());
            });
            delay(8_000_000);
        }
    }
}

#[interrupt]
fn EXTI0() {
    cortex_m::interrupt::free(|cs| {
        // clear pending register
        let exti = P_EXTI.borrow(cs).borrow();
        exti.as_ref().unwrap().pr.modify(|_, w| w.pr0().set_bit());
    });
    cortex_m::interrupt::free(|cs| {
        let is_set = {
            let gpioa = P_GPIOA.borrow(cs).borrow();
            gpioa.as_ref().unwrap().idr.read().idr0().bit_is_set()
        };
        IS_ROULETTE.borrow(cs).set(is_set);
    });
}
