#![no_std]
#![no_main]

use panic_halt as _; // you can put a breakpoint on `rust_begin_unwind` to catch panics

use cortex_m::asm::delay;
use cortex_m_rt::entry;
use stm32f4::stm32f407;

#[entry]
fn main() -> ! {
    let p = stm32f407::Peripherals::take().unwrap();

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

    loop {
        if gpioa.idr.read().idr0().bit_is_set() {
            gpiod.bsrr.write(|w| w.br15().reset());
            gpiod.bsrr.write(|w| w.bs12().set_bit());
            delay(2_000_000);
            gpiod.bsrr.write(|w| w.br12().reset());
            gpiod.bsrr.write(|w| w.bs13().set_bit());
            delay(2_000_000);
            gpiod.bsrr.write(|w| w.br13().reset());
            gpiod.bsrr.write(|w| w.bs14().set_bit());
            delay(2_000_000);
            gpiod.bsrr.write(|w| w.br14().reset());
            gpiod.bsrr.write(|w| w.bs15().set_bit());
            delay(2_000_000);
        } else {
            gpiod.bsrr.write(|w| w.bs12().set_bit());
            gpiod.bsrr.write(|w| w.bs13().set_bit());
            gpiod.bsrr.write(|w| w.bs14().set_bit());
            gpiod.bsrr.write(|w| w.bs15().set_bit());
            delay(8_000_000);
            gpiod.bsrr.write(|w| w.br12().reset());
            gpiod.bsrr.write(|w| w.br13().reset());
            gpiod.bsrr.write(|w| w.br14().reset());
            gpiod.bsrr.write(|w| w.br15().reset());
            delay(8_000_000);
        }
    }
}
