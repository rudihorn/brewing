#![no_std]
#![feature(proc_macro)]

pub extern crate tslib;
pub extern crate cortex_m;
pub extern crate cortex_m_rtfm as rtfm;
pub extern crate panic_abort;

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
    iprintln!("SPI Example");

    /* let timer = Timer(p.TIM1);

    timer.init(FREQUENCY.invert(), p.RCC);
    timer.resume(); */

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

    // initialize the temperature sensor
    tempsensor::init_temp(&p.device.SPI2, pinsb.12, pinsb.13, pinsb.14, pinsb.15);

    // initialize the screen
    screen::init_screen(&p.device.I2C1, pinsb.8, pinsb.9, afio_periph.i2c1);
    
    iprintln!("Finished initialization");

    // initialize external timer interrupt
    unsafe {
        // set line 5 to A0
        p.device.AFIO.exticr3.modify(|_, w| w.exti8().bits(0));
        // enable interrupt on line 8
        p.device.EXTI.imr.modify(|_, w| w.mr8().set_bit());
        // enable fall trigger on line 8
        p.device.EXTI.ftsr.modify(|_, w| w.tr8().set_bit());
    }

    let tim2 = rcc_periph.tim2.enable_tim2().reset();


    {
        // initialize timer
        let tim = &p.device.TIM2;
        // set prescaler to f_apb / 800
        tim.psc.modify(|_, w| w.psc().bits(799));
        // set auto reset register to 10
        tim.arr.modify(|_, w| w.arr().bits(10));
        // enable interrupt
        tim.dier.modify(|_, w| w.uie().set_bit());
        // enable counter
        tim.cr1.modify(|_, w| w.cen().set_bit());
    }

    init::LateResources {
        I2C1: p.device.I2C1,
        SPI2_REG: p.device.SPI2,
        EXTI: p.device.EXTI,
        GPIOA: p.device.GPIOA,
        TIM2_R: p.device.TIM2,
    }
}


fn idle(t: &mut Threshold, r: idle::Resources) -> ! {
    screen::set_address_mode(t, &r.I2C1);
    screen::set_address(t, &r.I2C1, 0, 0);


    for _i in 0..64 {
        ssd1306::write_data(t, &r.I2C1, &[0; 8]);
        ssd1306::wait_buffer();
    } 

    screen::set_address(t, &r.I2C1, 0, 0);
    screen::write_number(t, &r.I2C1, 10);

    r.SPI2_REG.claim(t, |spi, _t| {
        let spi = Spi(&*spi);
        unsafe {
           SPI_RES.start_write(0x80, 0b11010001, &spi);
           //SPI_RES.start_read(0x0, &spi);
        }
    });

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
    let i2c = r.I2C1;
    i2c.claim(t, |i2c1, _t| {
        let i2c = I2c(i2c1);
        ssd1306::event_interrupt(&i2c);
    });
}

fn i2c_er_interrupt(_t: &mut Threshold, r: I2C1_ER::Resources) {
    let a = (r.I2C1).sr1.read();

    iprintln!("er {} / AF {}", a.bits(), a.af().bit_is_set());
    rtfm::bkpt();
}

static mut READ_STATE : ReadState = ReadState::Conf;
static mut LAST_TEMP : u16 = 0;
static mut LAST_READ : u64 = 0;

fn spi_interrupt(t: &mut Threshold, r: SPI2::Resources) {
    let spi_res = unsafe { &mut SPI_RES };
    let spi = Spi(&*r.SPI2_REG);

    spi_res.process_int(&spi);

    unsafe {
        match SPI_RES.state {
            SpiState::Finished =>
            {
                match READ_STATE {
                    ReadState::Conf => {
                        SPI_RES.start_read(0x02, &spi);
                        READ_STATE = ReadState::Lsb;
                    }
                    ReadState::Lsb => {
                        SPI_RES.start_read(0x01, &spi);
                        READ_STATE = ReadState::Msb(SPI_RES.result);
                    }
                    ReadState::Msb(lsb) => {
                        let val : u16 = ((SPI_RES.result as u16) << 8) | (lsb as u16);
                        if val != LAST_TEMP {
                            LAST_TEMP = val;
                            iprint!("val: {} ", val);
                            let conv = ((val >> 1) as u32 * 43234) >> 15;
                            let temp = temp_conversion::lookup_temperature(conv as u16);

                            screen::set_address(t, &r.I2C1, 0, 0);
                            // ensure the number is completely covered by making sure 
                            // we always print 5 digits
                            if temp < 10000 {
                                screen::write_empty_digit(t, &r.I2C1);
                            }
                            screen::write_number(t, &r.I2C1, temp / 100);
                            screen::write_dot(t, &r.I2C1);
                            screen::write_number(t, &r.I2C1, temp % 100);
                            iprintln!("-> {}", temp);
                        }

                        // read next value
                        READ_STATE = ReadState::Lsb;
                        // SPI_RES.start_read(0x02, &spi);
                    }
                }
                // SPI_RES.state = SpiState::Idle;  
            }
            _ => {}
        }
    }
}

static mut CNTR : u64 = 0;

fn timer2_interrupt(t: &mut Threshold, r: TIM2::Resources) {
    let tim2 = &*r.TIM2_R;
    tim2.sr.modify(|_, w| w.uif().clear_bit());

    unsafe {
        CNTR += 1;
    }

    if unsafe { CNTR } % 1000 == 0 {
        iprintln!("ext {}", tim2.sr.read().bits());
        screen::set_address(t, &r.I2C1, 0, 1);
    }
    if unsafe { CNTR } % 2000 == 0 {
        screen::write_dot(t, &r.I2C1);
    } else if unsafe { CNTR } % 2000 == 1000 {
        screen::write_empty_digit(t, &r.I2C1);
    }

}

fn external_interrupt(_t: &mut Threshold, r: EXTI9_5::Resources) {
    unsafe {
        let c = CNTR;
        let took = c - LAST_READ;
        iprintln!("hz {} {}", 1000 / took, took);
        LAST_READ = c;
    }

    let spi = Spi(&*r.SPI2_REG);

    if r.EXTI.pr.read().pr8().bit_is_set() {
        unsafe { SPI_RES.start_read(0x02, &spi); }

        r.EXTI.pr.modify(|_, w| w.pr8().set_bit());
    }
}