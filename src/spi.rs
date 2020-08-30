use stm32f4::stm32f407;

const READ_FLAG: u16 = 0x80;
const MS_FLAG: u16 = 0x40;

const DUMMY: u16 = 0x00;

pub fn write(spi1: &stm32f407::SPI1, address: u16, data: u16) {
    send_bytes(&spi1, address);
    send_bytes(&spi1, data);
}

pub fn read(spi1: &stm32f407::SPI1, address: u16) -> u32 {
    send_bytes(&spi1, address | READ_FLAG);
    send_bytes(&spi1, DUMMY)
}

fn send_bytes(spi1: &stm32f407::SPI1, data: u16) -> u32 {
    while spi1.sr.read().txe().is_not_empty() {
        cortex_m::asm::nop();
    }
    spi1.dr.write(|w| w.dr().bits(data));
    while spi1.sr.read().rxne().is_empty() {
        cortex_m::asm::nop();
    }
    spi1.dr.read().bits()
}
