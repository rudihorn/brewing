use spi;
use spi::{Spi, SPIResult};
use cortex_m;

pub enum OpCode {
    PowerDown = 0b00000000,
    PowerOn = 0b00000001,
    Reset = 0b00000111,

    /// Continous High Resolution Mode
    /// 
    /// Start measurement at 1 lx resolution.
    /// Measurement Time is typically 120 ms.
    ContinuousHResolutionMode = 0b00010000,

    /// Continous High Resolution Mode 2
    /// 
    /// Start measurement at 0.5 lx resolution.
    /// Measurement Time is typically 120 ms.
    ContinuousHResolutionMode2 = 0b00010001,
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
        match self.state {
        SpiState::ReadFirst => 
            {   
                let b = self.read(spi);
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
                let b = self.read(spi);
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