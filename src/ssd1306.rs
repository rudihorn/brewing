
use ::tslib::i2c::{I2c, I2C, I2CState};
#[macro_use(iprint, iprintln)]
use ::cortex_m;

use ::cyclicbuffer::CyclicBuffer;

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
static NUMBERS : [[u8;5];2] = [
    [
        0b00000000,
        0b01000001,
        0b11111111,
        0b00000001,
        0b00000000,
    ],
    [
        0b00100011,
        0b01000101,
        0b10001001,
        0b10010001,
        0b01100001,
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

pub fn sync_init<'a, S>(i2c: &I2c<'a, S>, stim: &::cortex_m::peripheral::Stim) where S : 'static + I2C {
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
        //.cont(|| {write_control(&i2c, CMD_MEMORYMODE)})
        //.cont(|| {write_control(&i2c, 0x00)})
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

    iprintln!(stim, "state {}", a);

    let a = a.cont(|| { i2c.start_write_polling(ADDRESS)})
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

    if let I2CState::Error(e) = st {
        ::rtfm::bkpt;
    }
}


static mut _BUFFER : [u8;256] = [0; 256];
static mut BUFFER : CyclicBuffer<u8> = unsafe { CyclicBuffer { data: &mut _BUFFER, ptr: 0, len: 0} };

pub enum LcdState {
    Idle,
    Data(u8),
    Control(u8),
}
static mut LCD_STATE : LcdState = LcdState::Idle;

pub fn event_interrupt<'a, S>(i2c: &I2c<'a, S>) where S : 'static + I2C {
    /* 
    let state = i2c.0.sr1.read();

    if state.sb().bit_is_set() {
        i2c.start_write_async(ADDRESS);
    } else if state.tx_e().bit_is_set() {
        match LCD_STATE {
            LcdState::Idle => {
                let what = BUFFER.read();
                if let Some(a) = what {
                    if a & 0x80 == 0 {
                        LCD_STATE = LcdState::Data(a);                    
                        i2c.write_async(0x40);
                    } else {
                        LCD_STATE = LcdState::Control(a & (!0x80));
                        i2c.write_async(0x00);
                    }
                }
            }
        }
    }

    match LCD_STATE {
        LcdState::Idle => {
            let what = BFFER.read();
            if let Some(a) = what {
                if a & 0x80 == 0 {
                    LCD_STATE = LcdState::Data(a)                    
                } else {
                    LCD_STATE = LcdState::Control(a)
                }
            }
            if i2c.is_start_flag_set_async() {
                i2c.start_write(ADDRESS);
                MOD_STATE = ModuleState::PowerOff
            }
        },

    } */
}