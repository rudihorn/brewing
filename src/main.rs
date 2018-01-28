#![no_std]
#![feature(proc_macro)]

pub extern crate tslib;
pub extern crate cortex_m;
pub extern crate cortex_m_rtfm as rtfm;
pub extern crate cortex_m_semihosting as sh;

pub use tslib::stm32f103xx_hal as hal;
pub use hal::stm32f103xx as stm32;

use rtfm::{app, Resource, Threshold};

#[macro_use]
pub mod debug;
pub mod cyclicbuffer;
pub mod screen;
pub mod bh1750;
pub mod ssd1306;
pub mod temp_conversion;

use tslib::{rcc, afio, spi, gpio, i2c};

use rcc::{Rcc};
use afio::Afio;
use gpio::{Gpio};
use spi::{Spi, SPIResult};
use i2c::{I2c};
use bh1750::{SPI_RES, SpiState};

use cortex_m::peripheral::{Peripherals, ITM};
use stm32::{I2C1, SPI2 as SPI2_reg};

app! {
    device: stm32,

    resources: {
        static I2C1: I2C1;
        static SPI2_reg: SPI2_reg;
    },

    idle: {
        resources: [I2C1, SPI2_reg],
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
            resources: [I2C1, SPI2_reg]
        },
    },
}


#[inline(never)]
fn init(mut p: init::Peripherals) -> init::LateResources {
    iprintln!("SPI Example");

    /* let timer = Timer(p.TIM1);

    timer.init(FREQUENCY.invert(), p.RCC);
    timer.resume(); */

    let rcc = Rcc(&p.device.RCC);
    let rcc_periph = rcc.get_peripherals();

    rcc_periph.afio.enable();
    rcc_periph.spi1.enable_spi1();
    rcc_periph.spi2.enable_spi2();
    rcc_periph.iopa.enable_gpioa();
    rcc_periph.iopb.enable_gpiob();
    rcc_periph.iopc.enable_gpioc();
    rcc_periph.i2c1.enable_i2c1();

    /* our code */
    let gpiob = Gpio(&p.device.GPIOB);
    let pinsb = gpiob.get_pins();

    // setup SPI pins
    let gpioa = Gpio(&p.device.GPIOA);
    let pinsa = gpioa.get_pins();
    
    let pina4 = pinsa.4.set_output_10MHz().set_alt_output_push_pull();
    let pina5 = pinsa.5.set_output_10MHz().set_alt_output_push_pull();
    let pina6 = pinsa.6.set_input().set_floating_input();
    let pina7 = pinsa.7.set_output_10MHz().set_alt_output_push_pull();

    let pinb12 = pinsb.12.set_output_10MHz_h().set_alt_output_push_pull_h(); // NSS
    let pinb13 = pinsb.13.set_output_10MHz_h().set_alt_output_push_pull_h(); // SCK
    let pinb14 = pinsb.14.set_input_h().set_floating_input_h(); // MISO
    let pinb15 = pinsb.15.set_output_10MHz_h().set_alt_output_push_pull_h(); // MISO

    let afio = Afio(&p.device.AFIO);
    let afio_periph = afio.get_peripherals();

    // configure afio so it doesn't remap spi
    let afio_spi1 = afio_periph.spi1.set_not_remapped_spi1();

    {
        let spi2 = Spi(&p.device.SPI2);
        let r = spi2.start_init();

        //let ports = r.set_ports_normal(pina4, pina5, pina6, pina7, afio_spi1);
        let ports = r.set_ports(pinb12, pinb13, pinb14, pinb15);

        spi2.complete_init(ports);

        spi2.listen();
    }    

    screen::init_screen(&p.device.I2C1, pinsb.8, pinsb.9, afio_periph.i2c1);
    
    init::LateResources {
        I2C1: p.device.I2C1,
        SPI2_reg: p.device.SPI2,
    }
}


fn idle(t: &mut Threshold, r: idle::Resources) -> ! {
    screen::set_address_mode(t, &r.I2C1);
    screen::set_address(t, &r.I2C1, 0, 0);


    for i in 0..64 {
        ssd1306::write_data(t, &r.I2C1, &[0; 8]);
        ssd1306::wait_buffer();
    } 

    /* screen::set_address(t, &r.I2C1, 0, 1);
    screen::write_number(t, &r.I2C1, 69);
    screen::set_address(t, &r.I2C1, 0, 0);
    screen::write_number(t, &r.I2C1, 10); */

    let s = r.SPI2_reg.claim(t, |spi, _t| {
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
    let a = (r.I2C1).sr1.read().bits();
    //iprintln!(itm, "ev {}", a);
    let i2c = r.I2C1;
    i2c.claim(t, |i2c1, t| {
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

fn spi_interrupt(t: &mut Threshold, r: SPI2::Resources) {
    let spi_res = unsafe { &mut SPI_RES };
    let spi = Spi(&*r.SPI2_reg);

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
                            screen::write_number(t, &r.I2C1, temp / 100);
                            screen::write_dot(t, &r.I2C1);
                            screen::write_number(t, &r.I2C1, temp % 100);
                            iprintln!("-> {}", temp);
                        }

                        // read next value
                        READ_STATE = ReadState::Lsb;
                        SPI_RES.start_read(0x02, &spi);
                    }
                }
                // SPI_RES.state = SpiState::Idle;  
            }
            _ => {}
        }
    }
}
