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
#[macro_use]
extern crate cortex_m;
extern crate cortex_m_rtfm as rtfm;

use rtfm::{app, Threshold};


pub mod bh1750;
pub mod i2c;

use bh1750::OpCode;
use i2c::{I2c, I2CState};
use blue_pill::Timer;
use blue_pill::prelude::*;
use blue_pill::time::Hertz;

const FREQUENCY: Hertz = Hertz(2);

const BH1750_ADDR : u8 = 
        0b0100011; 
        //0b1011100;

app! {
    device: blue_pill::stm32f103xx,

    idle: {
        resources: [TIM1,ITM, I2C1]
    }
}

fn init(p: init::Peripherals) {
    let stim = &p.ITM.stim[0];
    iprintln!(stim, "I2C Example");

    let timer = Timer(p.TIM1);

    timer.init(FREQUENCY.invert(), p.RCC);
    timer.resume();

    let i2c = I2c(p.I2C1);
    i2c.init(false, p.AFIO, p.GPIOB, p.RCC);
    

    let state = i2c.start_write_polling(BH1750_ADDR)
        .cont(|| {i2c.write_data(OpCode::PowerOn as u8)})
        .cont(|| {i2c.stop()});
    iprintln!(stim, "Power on: {}", state);

    let state = state.cont(|| {i2c.start_write_polling(BH1750_ADDR)})
        .cont(|| {i2c.write_data(OpCode::ContinuousHResolutionMode2 as u8)})
        .cont(|| {i2c.stop()})
        .cont(|| { iprintln!(stim, "Set resolution mode: {}", state); I2CState::Ok });

    /* let mut low = 0;
    let mut high = 0;
    let state = state.cont(|| { i2c.start_read_polling(BH1750_ADDR)})
        .cont(|| { iprintln!(stim, "{}", i2c::I2C_SR2_P(i2c.0.sr2.read())); I2CState::Ok })
        .cont(|| { i2c.read_data(&mut low) })
        .cont(|| { i2c.read_data(&mut high) })
        .cont(|| { i2c.stop() });
    
    if state.is_ok() {
        iprintln!(stim, "result: {}", (high as u32) << 8 + (low as u32));
    } else {
        iprintln!(stim, "error fetching data: {}", state);
    } */

    //i2c.write_start_polling(0b0100011);

    // iprintln!(stim, "status: {}", state);

    iprintln!(&p.ITM.stim[0], "End init");

}

fn idle(_t: &mut Threshold, r:idle::Resources) -> ! {
    let stim = &r.ITM.stim[0];
    let timer = Timer(&*r.TIM1);
    let i2c = I2c(&*r.I2C1);

    iprintln!(stim, "Loop:");

    loop {
        let mut low = 0;
        let mut high = 0;
        let state =  i2c.start_read_polling(BH1750_ADDR)
                .cont(|| { i2c.read_data(&mut high) })
                .cont(|| { i2c.stop() })
                .cont(|| { i2c.read_last_data(&mut low ) });
    
        if state.is_ok() {
            iprintln!(stim, "result: {}", ((high as u32) << 8) + (low as u32));
        } else {
            iprintln!(stim, "error fetching data: {}", state);
        } 

        while timer.wait().is_err() {}


        // rtfm::wfi();
    }
}