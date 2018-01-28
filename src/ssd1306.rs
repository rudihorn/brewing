
use core::any::Any;
use tslib::i2c::{I2c, I2C, I2CState, I2cState, I2cStateOptions, Write};
use stm32::I2C1;

#[macro_use]
use debug;
use cortex_m;

use cyclicbuffer::CyclicBuffer;

const CMD_DISPLAYOFF : u8 = 0xAE;
const CMD_SETDISPLAYCLOCKDIV : u8 = 0xD5;
const CMD_SETMULTIPLEX : u8 = 0xA8;
const CMD_SETDISPLAYOFFSET : u8 = 0xD3;
const CMD_CHARGEPUMP : u8 = 0x8D;
const CMD_SETCOMPINS : u8 = 0xDA;
const CMD_SETCONTRAST : u8 = 0x81;
const CMD_SETSEGREMAP : u8 = 0xA0;
const CMD_SETPRECHARGE : u8 = 0xD9;
const CMD_SETVCOMDETECT : u8 = 0xDB;
const CMD_SETDISPLAYALLON_RESUME : u8 = 0xA4;
const CMD_NORMALDISPLAY : u8 = 0xA6;
const CMD_DEACTIVATE_SCROLL : u8 = 0x2E;
const CMD_DISPLAYON : u8 = 0xAF;
const CMD_LOWERCOL : u8 = 0x00;
const CMD_SETSTARTLINE : u8 = 0x40;
const CMD_MEMORYMODE : u8 = 0x20;
const CMD_INVERTDISPLAY : u8 = 0xA6;
pub static NUMBERS : [[u8;5];10] = [
    [ // 0
        0b01111110,
        0b10000001,
        0b10000001,
        0b10000001,
        0b01111110,
    ], // 1
    [
        0b00000000,
        0b01000001,
        0b11111111,
        0b00000001,
        0b00000000,
    ], // 2
    [
        0b01000011,
        0b10000101,
        0b10001001,
        0b10010001,
        0b01100001,
    ], // 3
    [
        0b01000010,
        0b10000001,
        0b10010001,
        0b10010001,
        0b01101110,
    ], // 4
    [
        0b00011000,
        0b00101000,
        0b01001000,
        0b11111111,
        0b00001000,
    ], // 5
    [
        0b11110001,
        0b10010001,
        0b10010001,
        0b10010001,
        0b10001110,
    ], // 6
    [
        0b01111110,
        0b10010001,
        0b10010001,
        0b10010001,
        0b10001110,
    ], // 7
    [
        0b11000000,
        0b10001111,
        0b10010000,
        0b10100000,
        0b11000000,
    ], // 8
    [
        0b01101110,
        0b10010001,
        0b10010001,
        0b10010001,
        0b11101110,
    ], // 9
    [
        0b01100000,
        0b10010001,
        0b10010001,
        0b10010001,
        0b01111110,
    ]
];


const LCD_HEIGHT : u8 = 32;


// https://cdn-shop.adafruit.com/datasheets/SSD1306.pdf
// could also be ...010 for SA0, last bit is R/W#
// adafruit says 7 bit 0x3C, so SA0 = 0
static ADDRESS : u8 = 0b0111100;

pub enum ModuleState {
    Starting,
    PowerOff,

}

static mut MOD_STATE : ModuleState = ModuleState::Starting;

#[inline(always)]
pub fn write_control<'a, S>(i2c: &I2c<'a, S>, b : u8) -> I2CState where S : 'static + I2C {
    i2c.start_write_polling(ADDRESS).cont(|| {i2c.write_data(0x00)}).cont(|| {i2c.write_data(b)}).cont(|| {i2c.stop()})
}

pub fn sync_init<'a, S>(i2c: &I2c<'a, S>) where S : 'static + I2C {
    let a = i2c.start_write_polling(ADDRESS)
        .cont(|| {i2c.write_data(0x00)})
        .cont(|| {i2c.write_data(CMD_DISPLAYOFF)})
        .cont(|| {i2c.stop()})
        .cont(|| {write_control(&i2c, CMD_DISPLAYOFF)})
        .cont(|| {write_control(&i2c, CMD_SETDISPLAYCLOCKDIV)})
        .cont(|| {write_control(&i2c, 0x80)})
        .cont(|| {write_control(&i2c, CMD_SETMULTIPLEX)})
        .cont(|| {write_control(&i2c, LCD_HEIGHT - 1)})
        .cont(|| {write_control(&i2c, CMD_SETDISPLAYOFFSET)})
        .cont(|| {write_control(&i2c, 0x00)})
        .cont(|| {write_control(&i2c, CMD_SETSTARTLINE)})
        .cont(|| {write_control(&i2c, CMD_MEMORYMODE)})
        .cont(|| {write_control(&i2c, 0x00)})
        .cont(|| {write_control(&i2c, CMD_CHARGEPUMP)})
        .cont(|| {write_control(&i2c, 0x14)})
        .cont(|| {write_control(&i2c, CMD_SETSEGREMAP | 0x01)})
        .cont(|| {write_control(&i2c, CMD_SETCOMPINS)})
        .cont(|| {write_control(&i2c, 0x2)})
        .cont(|| {write_control(&i2c, CMD_SETCONTRAST)})
        .cont(|| {write_control(&i2c, 0x8F)})
        .cont(|| {write_control(&i2c, CMD_SETPRECHARGE)})
        .cont(|| {write_control(&i2c, 0xF1)})
        .cont(|| {write_control(&i2c, CMD_SETVCOMDETECT)})
        .cont(|| {write_control(&i2c, 0x40)})
        .cont(|| {write_control(&i2c, CMD_SETDISPLAYALLON_RESUME)})
        .cont(|| {write_control(&i2c, CMD_NORMALDISPLAY)})
        .cont(|| {write_control(&i2c, CMD_DEACTIVATE_SCROLL)})
        .cont(|| {write_control(&i2c, CMD_DISPLAYON)})
        .cont(|| {i2c.stop()});

    iprintln!("state {}", a);

    let a = a.cont(|| { i2c.start_write_polling(ADDRESS)})
        .cont(|| {i2c.write_data(0x80)})
        .cont(|| {i2c.write_data(0x21)})
        .cont(|| {i2c.write_data(0x80)})
        .cont(|| {i2c.write_data(0x00)})
        .cont(|| {i2c.write_data(0x80)})
        .cont(|| {i2c.write_data(127)})
        .cont(|| {i2c.write_data(0x40)})
        .cont(|| {i2c.write_data(0x00)})
        .cont(|| {i2c.write_data(0x00)})
        .cont(|| {i2c.write_data(0x00)})
        .cont(|| {i2c.write_data(0x00)})
        .cont(|| {i2c.write_data(0x00)})
        .cont(|| {i2c.write_data(0x00)})
        .cont(|| {i2c.write_data(0x00)})
        .cont(|| {i2c.write_data(0x00)})
        .cont(|| {i2c.write_data(0x00)})
        .cont(|| {i2c.write_data(0x00)})
        .cont(|| {i2c.write_data(0x00)})
        .cont(|| {i2c.write_data(0x00)})
        .cont(|| {i2c.write_data(0x00)})
        .cont(|| {i2c.write_data(0x00)})
        .cont(|| {i2c.write_data(0x00)})
        .cont(|| {i2c.write_data(0x00)})
        .cont(|| {i2c.write_data(0x00)})
        .cont(|| {i2c.write_data(0x00)})
        .cont(|| {i2c.write_data(0x00)})
        .cont(|| {i2c.write_data(0x00)})
        .cont(|| {i2c.write_data(0x00)})
        .cont(|| {i2c.write_data(0x00)});

    let mut st = a;

    for s in NUMBERS[0].iter() {
        st = st.cont(|| {i2c.write_data(*s)});
    }

    for i in 0..10 {
        st = st.cont(|| {i2c.write_data(0x00)});
    }

    for s in NUMBERS[1].iter() {
        st = st.cont(|| {i2c.write_data(*s)});
    }

    for i in 0..10 {
        st = st.cont(|| {i2c.write_data(0x00)});
    }

    st = st.cont(|| {i2c.stop()});
    let cr1 = i2c.0.cr1.read().bits();

    if let I2CState::Error(e) = st {
        ::rtfm::bkpt();
    }
}


const _BUF_LEN : usize = 512; 
static mut _BUFFER : [u8;_BUF_LEN] = [0; _BUF_LEN];
static mut BUFFER : CyclicBuffer<u8> = unsafe { CyclicBuffer { data: &mut _BUFFER, ptr: 0, len: 0} };

pub enum LcdState {
    Stopped,
    Idle,
    Data(u8),
    Control(u8),
    Control2(u8),
}

static mut LCD_STATE : LcdState = LcdState::Stopped;

#[inline(never)]
pub fn wait_buffer() {
    unsafe {
        while BUFFER.length() > _BUF_LEN / 2 {
            ::rtfm::wfi(); 
        }
    }
}

use ::rtfm::{Resource, Threshold};

pub fn write_data<'a, S>(t: &mut Threshold, i2c: &S, dat: &[u8]) 
where 
    S : Resource<Data = I2C1>
{
    unsafe {
        BUFFER.write(dat.len() as u8);

        for el in dat.iter() {
            while false == BUFFER.write(*el) { }
        }
    }

    unsafe{
        if let LcdState::Stopped = LCD_STATE {
            i2c.claim(t, |i2c,t| {
                I2c(&*i2c).enable_start();
            });
            LCD_STATE = LcdState::Idle;
        }
    }

    i2c.claim(t, |i2c,t| {
        I2c(&*i2c).listen();
    });
}

pub fn write_control_2<'a, S>(t: &mut Threshold, i2c: &S, dat: &[u8]) 
where
    S : Resource<Data = I2C1>
{
    unsafe {
        BUFFER.write(0x80 | dat.len() as u8);

        for el in dat.iter() {
            while false == BUFFER.write(*el) {::rtfm::wfi() }
        }
    }

    unsafe{
        if let LcdState::Stopped = LCD_STATE {
            i2c.claim(t, |i2c,t| {
                I2c(&*i2c).enable_start();
            });
            LCD_STATE = LcdState::Idle;
        }
    }

    i2c.claim(t, |i2c,t| {
        I2c(&*i2c).listen();
    });
}

pub fn start_next<'a, S: I2C + Any>(i2c: I2cState<'a, S, Write>) {
    unsafe {
        let a = BUFFER.read();
        if let Some(a) = a {
            if a & 0x80 == 0 {
                LCD_STATE = LcdState::Data(a);
                i2c.write(0x40);
                //iprintln!("dat {}", a);
            } else {
                let a = a & (!0x80);
                LCD_STATE = LcdState::Control2(a);
                i2c.write(0x80);
                //iprintln!("ctrl {}", a);
            }
        } else {
            ::rtfm::bkpt();
        }
    }
}

pub fn event_interrupt<'a, S>(i2c: &I2c<'a, S>) where S : 'static + I2C {
    unsafe {
        match i2c.get_state() {
            I2cStateOptions::Started(s) => {
                s.write_address(ADDRESS, false);

                //iprintln!("st");
            }, 
            I2cStateOptions::CanWrite(w) => {
                //iprintln!("wr");

                match LCD_STATE {
                    LcdState::Stopped => {
                        // this should not happen
                        ::rtfm::bkpt(); 
                    }
                    LcdState::Idle => {
                        start_next(w);
                        //iprintln!(itm, "idle {}", i2c.0.sr1.read().bits());
                    },
                    LcdState::Control(b) => {
                        if b > 0 {
                            w.write(0x80);
                            LCD_STATE = LcdState::Control2(b);
                        } else {
                            //iprintln!("ectrl");
                            start_next(w)
                        }
                    },
                    LcdState::Control2(b) => {
                        let d = BUFFER.read();
                        if let Some(d) = d {
                            //iprintln!("cb {}", d);
                            w.write(d);
                            LCD_STATE = LcdState::Control(b - 1);
                        } else {
                            //iprintln!("ctrle");
                            w.suspend();
                        }
                    },
                    LcdState::Data(b) => {
                        // there is still data to write
                        if b > 0 {
                            let d = BUFFER.read();
                            if let Some(d) = d {
                                //iprintln!("db {}", d);
                                w.write(d);
                                LCD_STATE = LcdState::Data(b - 1);
                            } else {
                                //iprintln!("dbe");
                                // data to write but buffer is empty
                                w.suspend()
                            }
                        } else {
                            //iprintln!("nmd");
                            if let Some(d) = BUFFER.peak() {
                                if d & 0x80 == 0 {
                                    // if we have further data just queue it
                                    BUFFER.read();
                                    LCD_STATE = LcdState::Data(d)
                                } else {
                                    i2c.enable_start();
                                    LCD_STATE = LcdState::Idle;
                                } 
                            } else {
                                //iprintln!("stopped");
                                w.stop();
                                LCD_STATE = LcdState::Stopped;
                            }
                        }
                    }
                }
            }
            _ => ()
        }
    }
}