#![no_std]
#![no_main]

use core::cell::RefCell;

use panic_halt as _; // you can put a breakpoint on `rust_begin_unwind` to catch panics

use cortex_m::{self, asm::delay, interrupt::Mutex};
use cortex_m_rt::entry;
use stm32f4::stm32f407;

mod lis302dl;
mod spi;

static P_GPIOA: Mutex<RefCell<Option<stm32f407::GPIOA>>> = Mutex::new(RefCell::new(None));
static P_GPIOD: Mutex<RefCell<Option<stm32f407::GPIOD>>> = Mutex::new(RefCell::new(None));
static P_GPIOE: Mutex<RefCell<Option<stm32f407::GPIOE>>> = Mutex::new(RefCell::new(None));
static P_SPI1: Mutex<RefCell<Option<stm32f407::SPI1>>> = Mutex::new(RefCell::new(None));

#[entry]
fn main() -> ! {
    let p = stm32f407::Peripherals::take().unwrap();
    {
        // Setup PD12~PD15 for User leds
        p.RCC.ahb1enr.modify(|_, w| w.gpioden().set_bit());
        let gpiod = &p.GPIOD;
        gpiod.moder.modify(|_, w| {
            w.moder12().output();
            w.moder13().output();
            w.moder14().output();
            w.moder15().output()
        });

        // Setup PE3 to Set SPI mode of LIS302DL
        p.RCC.ahb1enr.modify(|_, w| w.gpioeen().set_bit());
        let gpioe = &p.GPIOE;
        gpioe.moder.modify(|_, w| w.moder3().output());
        // disable for now
        gpioe.bsrr.write(|w| w.bs3().set_bit());

        // Setup GPIO for SPI to control onboard LIS302DL (PA5 -> SCK, PA6 -> MISO, PA7 -> MOSI)
        p.RCC.ahb1enr.modify(|_, w| w.gpioaen().set_bit());
        let gpioa = &p.GPIOA;
        // set alternate function
        gpioa.moder.modify(|_, w| {
            w.moder5().alternate();
            w.moder6().alternate();
            w.moder7().alternate()
        });
        // enable alternate function of SPI
        gpioa.afrl.modify(|_, w| {
            w.afrl5().af5();
            w.afrl6().af5();
            w.afrl7().af5()
        });
        // set max speed
        gpioa.ospeedr.modify(|_, w| {
            w.ospeedr5().very_high_speed();
            w.ospeedr6().very_high_speed();
            w.ospeedr7().very_high_speed()
        });
        // set output to push pull
        gpioa.otyper.modify(|_, w| {
            w.ot5().push_pull();
            w.ot6().push_pull();
            w.ot7().push_pull()
        });

        // Setup SPI1 for onboard LIS302DL
        p.RCC.apb2enr.modify(|_, w| w.spi1en().set_bit());
        let spi1 = &p.SPI1;
        // CR1
        spi1.cr1.modify(|_, w| {
            // master
            w.mstr().master();
            // MSB first
            w.lsbfirst().msbfirst();
            // set max speed clock (system clock / 2)
            w.br().div2();
            // CK to 0 when idle
            w.cpol().idle_low();
            // the first clock transition is the first data capture edge
            w.cpha().first_edge();
            // disable slave function
            w.ssm().enabled();
            w.ssi().slave_not_selected();
            // enable peripheral
            w.spe().enabled()
        });
        // CR2 is all default
    }

    // Share peripherals with mutex
    cortex_m::interrupt::free(|cs| {
        P_GPIOA.borrow(cs).replace(Some(p.GPIOA));
        P_GPIOD.borrow(cs).replace(Some(p.GPIOD));
        P_GPIOE.borrow(cs).replace(Some(p.GPIOE));
        P_SPI1.borrow(cs).replace(Some(p.SPI1));
    });

    cortex_m::interrupt::free(|cs| {
        let gpioe = P_GPIOE.borrow(cs).borrow();
        let spi1 = P_SPI1.borrow(cs).borrow();

        gpioe.as_ref().unwrap().bsrr.write(|w| w.br3().set_bit());
        spi::write(
            &spi1.as_ref().unwrap(),
            lis302dl::REG_CTRL_REG1,
            lis302dl::ON,
        );
        gpioe.as_ref().unwrap().bsrr.write(|w| w.bs3().set_bit());
    });

    loop {
        let (x, y, z) = cortex_m::interrupt::free(|cs| {
            let gpioe = P_GPIOE.borrow(cs).borrow();
            let spi1 = P_SPI1.borrow(cs).borrow();

            gpioe.as_ref().unwrap().bsrr.write(|w| w.br3().set_bit());
            let x = spi::read(&spi1.as_ref().unwrap(), lis302dl::REG_OUT_X);
            gpioe.as_ref().unwrap().bsrr.write(|w| w.bs3().set_bit());

            gpioe.as_ref().unwrap().bsrr.write(|w| w.br3().set_bit());
            let y = spi::read(&spi1.as_ref().unwrap(), lis302dl::REG_OUT_Y);
            gpioe.as_ref().unwrap().bsrr.write(|w| w.bs3().set_bit());

            gpioe.as_ref().unwrap().bsrr.write(|w| w.br3().set_bit());
            let z = spi::read(&spi1.as_ref().unwrap(), lis302dl::REG_OUT_Z);
            gpioe.as_ref().unwrap().bsrr.write(|w| w.bs3().set_bit());

            (x, y, z)
        });

        cortex_m::interrupt::free(|cs| {
            let gpiod = P_GPIOD.borrow(cs).borrow();
            gpiod.as_ref().unwrap().bsrr.write(|w| {
                if x > 0x00 && x < 0x40 {
                    w.br13().set_bit();
                    w.bs15().set_bit();
                } else if x > 0xA0 && x < 0xFE {
                    w.bs13().set_bit();
                    w.br15().set_bit();
                } else {
                    w.br13().set_bit();
                    w.br15().set_bit();
                }
                if y > 0x00 && y < 0x40 {
                    w.br12().set_bit();
                    w.bs14().set_bit();
                } else if y > 0xA0 && y < 0xFE {
                    w.bs12().set_bit();
                    w.br14().set_bit();
                } else {
                    w.br12().set_bit();
                    w.br14().set_bit();
                }
                w
            });
        });

        delay(1_000_000);
    }
}
