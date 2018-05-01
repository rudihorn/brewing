
#[macro_use]
#[allow(unused_imports)]
use debug;
use stm32;
use spi;
use cortex_m;

use spi::{Spi, SPIResult, SpiStateOptions};
//use rtfm::{Resource, Threshold};
use tslib::gpio::{GpioPinDefault, Pin12, Pin13, Pin14, Pin15};


pub fn init_temp<'a>(
    spi2: &'a stm32::SPI2,
    pinb12: GpioPinDefault<'a, stm32::GPIOB, Pin12>, 
    pinb13: GpioPinDefault<'a, stm32::GPIOB, Pin13>,
    pinb14: GpioPinDefault<'a, stm32::GPIOB, Pin14>, 
    pinb15: GpioPinDefault<'a, stm32::GPIOB, Pin15>) {

    let pinb12 = pinb12.set_output_10MHz().set_alt_output_push_pull(); // NSS
    let pinb13 = pinb13.set_output_10MHz().set_alt_output_push_pull(); // SCK
    let pinb14 = pinb14.set_input().set_floating_input(); // MISO
    let pinb15 = pinb15.set_output_10MHz().set_alt_output_push_pull(); // MISO

    let spi2 = Spi(spi2);
    let r = spi2.start_init();

    let ports = r.set_ports(pinb12, pinb13, pinb14, pinb15);

    spi2.complete_init(ports);

    spi2.listen(false, true);
}

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


pub static mut SPI_RES: SpiResource = SpiResource { 
    action: SpiAction::Read,
    state: SpiState::Idle,

    result: 0,
}; 

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

    pub fn read<'a, S : spi::SPI + 'static>(&mut self, spi : &Spi<'a, S>) -> Option<u8> {
        let b = spi.read();
        match b {
            SPIResult::Success(a) => {
                Some(a)
            },
            SPIResult::Error(e) =>  {
                iprintln!("read error: {}", e);
                None
            }
        }
    }

    pub fn process_int<'a, S : spi::SPI + 'static>(&mut self, spi : &Spi<'a, S>) {
        let state = spi.get_state();

        match state {
            SpiStateOptions::CanRead(read) => {
                let b = read.read();
                match self.state {
                    SpiState::ReadFirst => {
                        match self.action {
                            SpiAction::Read => spi.send(0),
                            SpiAction::Write(a) => spi.send(a),
                        };
                        self.state = SpiState::ReadSecond
                    }
                    SpiState::ReadSecond => {
                        self.result = b;
                        spi.disable();
                        self.state = SpiState::Finished
                    }
                    _ => {}
                }
            }
            SpiStateOptions::Error(_error, code) => {
                iprintln!("Error: {}", code);
            }
            _ => {}
        }
    }
}