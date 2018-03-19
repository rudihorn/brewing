#![no_std]
#![feature(proc_macro)]

pub extern crate tslib;
pub extern crate cortex_m;
pub extern crate cortex_m_rtfm as rtfm;

pub use tslib::stm32f103xx_hal as hal;
pub use hal::stm32f103xx as stm32;

use rtfm::{app, Resource, Threshold};

#[macro_use]
pub mod debug;
pub mod cyclicbuffer;
pub mod screen;
pub mod tempsensor;
pub mod bh1750;
pub mod ssd1306;
pub mod temp_conversion;

use tslib::{rcc, afio, spi, gpio, i2c};

use rcc::{Rcc};
use afio::Afio;
use gpio::{Gpio};
use spi::{Spi};
use i2c::{I2c};
use tempsensor::{SPI_RES, SpiState};

use stm32::{GPIOA, I2C1, EXTI, SPI2 as SPI2_reg, TIM2 as TIM2_R};

app! {
    device: stm32,

    resources: {
        static COUNTER: u64 = 0;
        static I2C1: I2C1;
        static SPI2_REG: SPI2_reg;
        static EXTI: EXTI;
        static GPIOA: GPIOA;
        static TIM2_R: TIM2_R;
    },

    idle: {
        resources: [I2C1, SPI2_REG, EXTI, GPIOA, TIM2_R],
    }, 

    tasks: {
        I2C1_EV: {
            path: i2c_ev_interrupt,
            priority: 1,
            resources: [I2C1]
        },
        I2C1_ER: {
            path: i2c_er_interrupt,
            priority: 1,
            resources: [I2C1]
        },
        SPI2: {
            path: spi_interrupt,
            priority: 1,
            resources: [I2C1, SPI2_REG]
        },
        EXTI9_5: {
            path: external_interrupt,
            priority: 1,
            resources: [EXTI, SPI2_REG]
        },
        TIM2: {
            path: timer2_interrupt,
            priority: 1,
            resources: [I2C1, COUNTER, TIM2_R]
        }
    },
}


#[inline(never)]
fn init(p: init::Peripherals, _r : init::Resources) -> init::LateResources {
    iprintln!("Skeleton");

    let rcc = Rcc(&p.device.RCC);
    let rcc_periph = rcc.get_peripherals();

    rcc_periph.afio.enable();
    rcc_periph.spi2.enable_spi2();
    let rcc_io_a = rcc_periph.iopa.enable_gpioa();
    let rcc_io_b = rcc_periph.iopb.enable_gpiob();
    rcc_periph.i2c1.enable_i2c1();

    // get the gpio b pins
    let gpiob = Gpio(&p.device.GPIOB);
    let pinsb = gpiob.get_pins(rcc_io_b);

    let afio = Afio(&p.device.AFIO);
    let afio_periph = afio.get_peripherals();

    init::LateResources {
        I2C1: p.device.I2C1,
        SPI2_REG: p.device.SPI2,
        EXTI: p.device.EXTI,
        GPIOA: p.device.GPIOA,
        TIM2_R: p.device.TIM2,
    }
}


fn idle(t: &mut Threshold, r: idle::Resources) -> ! {
    iprintln!("Entering idle loop...");

    loop {
        rtfm::wfi();
    }
}

pub enum ReadState {
    Conf,
    Lsb,
    Msb(u8)
}

fn i2c_ev_interrupt(t: &mut Threshold, r: I2C1_EV::Resources) {
}

fn i2c_er_interrupt(_t: &mut Threshold, r: I2C1_ER::Resources) {
}


fn spi_interrupt(t: &mut Threshold, r: SPI2::Resources) {
}


fn timer2_interrupt(t: &mut Threshold, r: TIM2::Resources) {}

fn external_interrupt(_t: &mut Threshold, r: EXTI9_5::Resources) {
}