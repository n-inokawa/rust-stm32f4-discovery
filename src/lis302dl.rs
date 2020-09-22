const EXPECTED_DEVICE_ID: u16 = 0x3B;

pub const REG_WHO_AM_I: u16 = 0x0F;
pub const REG_CTRL_REG1: u16 = 0x20;
pub const REG_OUT_X: u16 = 0x29;
pub const REG_OUT_Y: u16 = 0x2B;
pub const REG_OUT_Z: u16 = 0x2D;

// CTRL_REG1
// default 100Hz
const DATA_RATE_400_HZ: u16 = 0x80;
// default power down
const ACTIVE_MODE: u16 = 0x40;
// default +-2G
const SCALE_PLUS_MINUS_8G: u16 = 0x20;
const Z_ENABLE: u16 = 0x04;
const Y_ENABLE: u16 = 0x02;
const X_ENABLE: u16 = 0x01;

pub const ON: u16 = X_ENABLE | Y_ENABLE | Z_ENABLE | ACTIVE_MODE | SCALE_PLUS_MINUS_8G;
