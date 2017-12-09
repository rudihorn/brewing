//! Sends "Hello, world!" through the ITM port 0
//!
//! **IMPORTANT** Not all Cortex-M chips support ITM. You'll have to connect the
//! microcontroller's SWO pin to the SWD interface. Note that some development
//! boards don't provide this option.
//!
//! ITM is much faster than semihosting. Like 4 orders of magnitude or so.
//!
//! You'll need [`itmdump`] to receive the message on the host plus you'll need
//! to uncomment the `monitor` commands in the `.gdbinit` file.
//!
//! [`itmdump`]: https://docs.rs/itm/0.1.1/itm/
//!
//! ---

#![feature(get_type_id)]
#![feature(proc_macro)]
#![no_std]

extern crate blue_pill;
#[allow(unused_imports)]
#[macro_use(iprint, iprintln)]
extern crate cortex_m;
extern crate cortex_m_rtfm as rtfm;
extern crate tslib;

use rtfm::{app, Threshold};

pub mod cyclicbuffer;
pub mod bh1750;
pub mod ssd1306;
pub mod temp_conversion;

use tslib::{rcc, afio, spi, gpio, i2c};

use rcc::{Rcc};
use afio::Afio;
use gpio::{Gpio};
use spi::{Spi, SPIResult};
use i2c::{I2c};
use blue_pill::Timer;
use blue_pill::prelude::*;
use blue_pill::time::Hertz;
use cortex_m::peripheral::Stim;

const FREQUENCY: Hertz = Hertz(2);


pub enum SpiState {
    Idle,
    ReadFirst,
    ReadSecond,
    WriteFirst,
    WriteSecond,
    Finished
}

pub enum SpiAction {
    Read,
    Write(u8),
}

pub struct SpiResource {
    pub action : SpiAction,
    pub state : SpiState,

    pub result : u8,
}


static mut SPI_RES: SpiResource = SpiResource { 
    action: SpiAction::Read,
    state: SpiState::Idle,

    result: 0,
}; 

app! {
    device: blue_pill::stm32f103xx,

    idle: {
        resources: [TIM1, ITM, I2C1, SPI2],
    }, 

    tasks: {
        I2C1_EV: {
            path: i2c_ev_interrupt,
            priority: 1,
            resources: [ITM, I2C1]
        },
        I2C1_ER: {
            path: i2c_er_interrupt,
            priority: 1,
            resources: [ITM, I2C1]
        },
        SPI2: {
            path: spi_interrupt,
            priority: 1,
            resources: [ITM, SPI2]
        },
    },
}

impl SpiResource {
    pub fn start_read<'a, S : spi::SPI + 'static>(&mut self, reg: u8, spi : &Spi<'a, S>) {
        spi.enable();
        //rtfm::bkpt();
        spi.send(reg);

        self.state = SpiState::ReadFirst;
        self.action = SpiAction::Read;
    }

    pub fn start_write<'a, S : spi::SPI + 'static>(&mut self, reg: u8, val: u8, spi : &Spi<'a, S>) {
        spi.enable();
        spi.send(reg);

        self.state = SpiState::ReadFirst;
        self.action = SpiAction::Write(val);
    }

    pub fn read<'a, S : spi::SPI + 'static>(&mut self, stim : &Stim, spi : &Spi<'a, S>) -> Option<u8> {
        let b = spi.read();
        match b {
            SPIResult::Success(a) => {
                Some(a)
            },
            SPIResult::Error(e) =>  {
                // iprintln!(stim, "read error: {}", e);
                None
            }
        }
    }

    pub fn process_int<'a, S : spi::SPI + 'static>(&mut self, stim : &Stim, spi : &Spi<'a, S>) {
        match self.state {
        SpiState::ReadFirst => 
            {   
                let b = self.read(stim, spi);
                if let Some(a) = b {
                    match self.action {
                        SpiAction::Read => spi.send(0),
                        SpiAction::Write(a) => spi.send(a),
                    };
                    self.state = SpiState::ReadSecond
                }
            }
        SpiState::ReadSecond => 
            {
                let b = self.read(stim, spi);
                if let Some(a) = b {
                    self.result = a;
                    spi.disable();
                    self.state = SpiState::Finished
                }
            },
            _ => {}
        }
    }
}

#[inline(never)]
fn init(p: init::Peripherals) {
    let stim = &p.ITM.stim[0];
    iprintln!(stim, "SPI Example");

    /* let timer = Timer(p.TIM1);

    timer.init(FREQUENCY.invert(), p.RCC);
    timer.resume(); */

    let rcc = Rcc(p.RCC);
    let rcc_periph = rcc.get_peripherals();

    rcc_periph.afio.enable();
    rcc_periph.spi1.enable_spi1();
    rcc_periph.spi2.enable_spi2();
    rcc_periph.iopa.enable_gpioa();
    rcc_periph.iopb.enable_gpiob();
    rcc_periph.iopc.enable_gpioc();
    rcc_periph.i2c1.enable_i2c1();

    let gpiob = Gpio(p.GPIOB);
    let pinsb = gpiob.get_pins();

    // setup 
    let pinb8 = pinsb.8.set_output_2MHz_h().set_alt_output_open_drain_h();
    let pinb9 = pinsb.9.set_output_2MHz_h().set_alt_output_open_drain_h();

    // setup SPI pins
    let gpioa = Gpio(p.GPIOA);
    let pinsa = gpioa.get_pins();
    
    let pina4 = pinsa.4.set_output_10MHz().set_alt_output_push_pull();
    let pina5 = pinsa.5.set_output_10MHz().set_alt_output_push_pull();
    let pina6 = pinsa.6.set_input().set_floating_input();
    let pina7 = pinsa.7.set_output_10MHz().set_alt_output_push_pull();

    let pinb12 = pinsb.12.set_output_10MHz_h().set_alt_output_push_pull_h(); // NSS
    let pinb13 = pinsb.13.set_output_10MHz_h().set_alt_output_push_pull_h(); // SCK
    let pinb14 = pinsb.14.set_input_h().set_floating_input_h(); // MISO
    let pinb15 = pinsb.15.set_output_10MHz_h().set_alt_output_push_pull_h(); // MISO

    let afio = Afio(p.AFIO);
    let afio_periph = afio.get_peripherals();

    // configure afio so it doesn't remap spi
    let afio_spi1 = afio_periph.spi1.set_not_remapped_spi1();

    let spi2 = Spi(p.SPI2);
    {
        let r = spi2.start_init();

        //let ports = r.set_ports_normal(pina4, pina5, pina6, pina7, afio_spi1);
        let ports = r.set_ports(pinb12, pinb13, pinb14, pinb15);

        spi2.complete_init(ports);

        spi2.listen();
    }    

    let afio_i2c1 = afio_periph.i2c1.set_remapped();
    let i2c1 = I2c(p.I2C1);
    {
        let r = i2c1.start_init();

        let bsm = r.0.set_fast_mode(10);
        let freq = r.1.set(8);
        let trise = r.2.set(4);
        let ports = r.3.set_ports_remapped(pinb8, pinb9, afio_i2c1);
        i2c1.complete_init(bsm, freq, trise, ports);


    }    


    ssd1306::sync_init(&i2c1, stim);

    // i2c1.listen();
}


fn idle(t: &mut Threshold, r: idle::Resources) -> ! {
    use rtfm::Resource;
    use core::ops::Deref;
    let s = r.SPI2.claim(t, |spi, _t| {
        let spi = Spi(spi.deref());

        unsafe { 
            SPI_RES.start_write(0x80, 0b11010001, &spi);
            // SPI_RES.start_read(0x0, &spi);
        }
    }); 

    loop {
        rtfm::wfi();

    }
}

pub enum ReadState {
    Conf,
    Lsb,
    Msb(u8)
}

fn i2c_ev_interrupt(_t: &mut Threshold, r: I2C1_EV::Resources) {
}

fn i2c_er_interrupt(_t: &mut Threshold, r: I2C1_ER::Resources) {
    rtfm::bkpt();
}

static mut READ_STATE : ReadState = ReadState::Conf;
static mut LAST_TEMP : u16 = 0;

fn spi_interrupt(_t: &mut Threshold, r: SPI2::Resources) {
    let spi_res = unsafe { &mut SPI_RES };
    let stim = &r.ITM.stim[0];
    let spi = Spi(&**r.SPI2);

    spi_res.process_int(stim, &spi);


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
                            iprint!(stim, "val: {} ", val);
                            let conv = ((val >> 1) as u32 * 43234) >> 15;
                            let temp = temp_conversion::lookup_temperature(conv as u16);
                            iprintln!(stim, "-> {}", temp);
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
