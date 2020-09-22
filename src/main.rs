#![no_std]
#![no_main]

use core::cell::RefCell;

use panic_halt as _; // you can put a breakpoint on `rust_begin_unwind` to catch panics

use cortex_m::{self, asm, interrupt::Mutex, peripheral::syst::SystClkSource};
use cortex_m_rt::{entry, exception};
use stm32f4::stm32f407;

use rust_stm32f4_discovery::{lis302dl, spi};

static P_GPIOA: Mutex<RefCell<Option<stm32f407::GPIOA>>> = Mutex::new(RefCell::new(None));
static P_GPIOD: Mutex<RefCell<Option<stm32f407::GPIOD>>> = Mutex::new(RefCell::new(None));
static P_GPIOE: Mutex<RefCell<Option<stm32f407::GPIOE>>> = Mutex::new(RefCell::new(None));
static P_SPI1: Mutex<RefCell<Option<stm32f407::SPI1>>> = Mutex::new(RefCell::new(None));

static COUNT_MAX: u8 = 60;

#[entry]
fn main() -> ! {
    let c_p = cortex_m::Peripherals::take().unwrap();
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

        // Setup SysTick for interrupt
        let mut syst = c_p.SYST;
        syst.set_clock_source(SystClkSource::Core);
        // HSI used for system clock is 16MHz
        syst.set_reload(16_000_000 / COUNT_MAX as u32 - 1);
        syst.enable_interrupt();
        syst.enable_counter();
    }

    // Share peripherals with mutex
    cortex_m::interrupt::free(|cs| {
        P_GPIOA.borrow(cs).replace(Some(p.GPIOA));
        P_GPIOD.borrow(cs).replace(Some(p.GPIOD));
        P_GPIOE.borrow(cs).replace(Some(p.GPIOE));
        P_SPI1.borrow(cs).replace(Some(p.SPI1));
    });

    // Initialize LIS302DL
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
        asm::nop();
    }
}

#[exception]
fn SysTick() {
    static mut COUNT_G: u8 = 0;
    static mut COUNT_R: u8 = 0;
    static mut COUNT_B: u8 = 0;
    static mut COUNT_O: u8 = 0;

    let (x, y, z) = cortex_m::interrupt::free(|cs| {
        let gpioe = P_GPIOE.borrow(cs).borrow();
        let spi1 = P_SPI1.borrow(cs).borrow();

        gpioe.as_ref().unwrap().bsrr.write(|w| w.br3().set_bit());
        let x = spi::read(&spi1.as_ref().unwrap(), lis302dl::REG_OUT_X) as u8;
        gpioe.as_ref().unwrap().bsrr.write(|w| w.bs3().set_bit());

        gpioe.as_ref().unwrap().bsrr.write(|w| w.br3().set_bit());
        let y = spi::read(&spi1.as_ref().unwrap(), lis302dl::REG_OUT_Y) as u8;
        gpioe.as_ref().unwrap().bsrr.write(|w| w.bs3().set_bit());

        gpioe.as_ref().unwrap().bsrr.write(|w| w.br3().set_bit());
        let z = spi::read(&spi1.as_ref().unwrap(), lis302dl::REG_OUT_Z) as u8;
        gpioe.as_ref().unwrap().bsrr.write(|w| w.bs3().set_bit());

        (x, y, z)
    });

    match x {
        0x00...0x7F => {
            *COUNT_G += 1 + (COUNT_MAX - 1) * ((0x7F - x) / 0x7F);
            *COUNT_R += 1;
        }
        0x80...0xFF => {
            *COUNT_G += 1;
            *COUNT_R += 1 + (COUNT_MAX - 1) * ((x - 0x80) / 0x7F);
        }
    }
    match y {
        0x00...0x7F => {
            *COUNT_B += 1 + (COUNT_MAX - 1) * ((0x7F - y) / 0x7F);
            *COUNT_O += 1;
        }
        0x80...0xFF => {
            *COUNT_B += 1;
            *COUNT_O += 1 + (COUNT_MAX - 1) * ((y - 0x80) / 0x7F);
        }
    }

    if *COUNT_G >= COUNT_MAX {
        *COUNT_G = 0;
        cortex_m::interrupt::free(|cs| {
            let gpiod = P_GPIOD.borrow(cs).borrow();
            let gpiod_ref = gpiod.as_ref().unwrap();
            if gpiod_ref.idr.read().idr15().bit_is_set() {
                gpiod_ref.bsrr.write(|w| w.br15().set_bit());
            } else {
                gpiod_ref.bsrr.write(|w| w.bs15().set_bit());
            }
        });
    }
    if *COUNT_R >= COUNT_MAX {
        *COUNT_R = 0;
        cortex_m::interrupt::free(|cs| {
            let gpiod = P_GPIOD.borrow(cs).borrow();
            let gpiod_ref = gpiod.as_ref().unwrap();
            if gpiod_ref.idr.read().idr13().bit_is_set() {
                gpiod_ref.bsrr.write(|w| w.br13().set_bit());
            } else {
                gpiod_ref.bsrr.write(|w| w.bs13().set_bit());
            }
        });
    }
    if *COUNT_B >= COUNT_MAX {
        *COUNT_B = 0;
        cortex_m::interrupt::free(|cs| {
            let gpiod = P_GPIOD.borrow(cs).borrow();
            let gpiod_ref = gpiod.as_ref().unwrap();
            if gpiod_ref.idr.read().idr14().bit_is_set() {
                gpiod_ref.bsrr.write(|w| w.br14().set_bit());
            } else {
                gpiod_ref.bsrr.write(|w| w.bs14().set_bit());
            }
        });
    }
    if *COUNT_O >= COUNT_MAX {
        *COUNT_O = 0;
        cortex_m::interrupt::free(|cs| {
            let gpiod = P_GPIOD.borrow(cs).borrow();
            let gpiod_ref = gpiod.as_ref().unwrap();
            if gpiod_ref.idr.read().idr12().bit_is_set() {
                gpiod_ref.bsrr.write(|w| w.br12().set_bit());
            } else {
                gpiod_ref.bsrr.write(|w| w.bs12().set_bit());
            }
        });
    }
}
