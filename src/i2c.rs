//! I2C Bus
//! 
//! # I2C1
//! 
//! - SCL = PB6 (remapped: PB8)
//! - SDA = PB7 (remapped: PB9)
//! 
//! # I2C2
//! 
//! - SCL = PB10
//! - SDA = PB11

use core::any::{Any, TypeId};
use core::fmt::{Display, Formatter, Result};
use core::ops::Deref;

use blue_pill::stm32f103xx::{AFIO, GPIOB, I2C1, I2C2, i2c1, RCC};


pub unsafe trait I2C: Deref<Target = i2c1::RegisterBlock> {
}

unsafe impl I2C for I2C1 {

}

unsafe impl I2C for I2C2 {
}

pub struct I2c<'a, S>(pub &'a S)
where 
    S: Any + I2C;

pub enum I2CState {
    Ok,
    /// The I2C module is still busy, so wait for a further response
    Busy,
    /// The module has encountered the error `I2CError`
    Error(I2CError)
}

type I2CWriteState = I2CState;

impl I2CWriteState {
    pub fn cont<F>(&self, mut f : F) -> I2CWriteState 
    where F : FnMut() -> I2CWriteState {
        match *self {
            I2CState::Ok => f(),
            I2CState::Busy => I2CState::Busy,
            I2CState::Error(err) => I2CState::Error(err)
        }
    }
}

type I2CReadState = I2CState;

/* impl<T> I2CReadState<T> {
    pub fn cont<F,T2>(&self, f : F) -> I2CReadState<T2> 
    where F : Fn(T) -> I2CReadState<T2> {
        match self.0 {
            I2CState::Ok => f(self.1.unwrap()),
            I2CState::Busy => I2CReadState(I2CState::Busy, None),
            I2CState::Error(err) => I2CReadState(I2CState::Error(err), None)
        }
    }
} */

impl I2CState {
    #[inline(always)]
    pub fn is_ok(&self) -> bool {
        match *self {
            I2CState::Ok => true,
            _ => false
        }
    }

    #[inline(always)]
    pub fn is_busy(&self) -> bool {
        match *self {
            I2CState::Busy => true,
            _ => false
        }
    }

    /* 
    #[inline(always)]
    pub fn cont<T>(&self, fun : T) -> I2CState
        where T : Fn() -> I2CState {
            if self.is_ok() {
                fun()
            } else {
                *self
            }
        } */
}

impl Display for I2CState {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match *self {
            I2CState::Ok => write!(f, "Ok"),
            I2CState::Busy => write!(f, "Busy"),
            I2CState::Error(ref err) => write!(f, "Error<{}>", err)
        }
    }
}

pub struct I2C_SR2_P(pub i2c1::sr2::R);

impl Display for I2C_SR2_P {
    fn fmt(&self, f: &mut Formatter) -> Result {
        let reg = &self.0;
        write!(f, "TRA: {}, BUSY: {}, MSL: {}", reg.tra().bit_is_set(), reg.busy().bit_is_set(), reg.msl().bit_is_set())
    }

}

#[derive(Copy, Clone)]
pub enum I2CError {
    /// No known error has occured
    None,
    /// Timeout Failure
    /// 
    /// - SCL remained low for 25 ms
    /// - Master cumulative clock low extend time more than 10ms
    /// - Slave cumulative clock low extend time more than 25ms
    Timeout,
    /// Acknowledge Failure.
    /// 
    /// No acknowledge returned.
    AF,
    /// Arbitration Lost
    /// 
    /// Arbritation to the bus is lost to another master
    ARLO,
    /// Overrun / Underrun
    /// 
    /// - During reception a new byte is received even though the DR has not been read.
    /// - During transmission when a new byte should be sent, but the DR register has not been written to.
    OVR,
    /// Bus Error
    BERR,
}


impl I2CError {
    pub fn if_no_err<F>(&self, fun : F) -> I2CState
        where F: Fn() -> I2CState {
        match *self {
            I2CError::None => fun(),
            ref err => I2CState::Error(*err)
        }
    }
}

impl Display for I2CError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match *self {
            I2CError::None => write!(f, "None"),
            I2CError::Timeout => write!(f,"Timeout"),
            I2CError::AF => write!(f,"AF"),
            I2CError::ARLO => write!(f, "ARLO"),
            I2CError::OVR => write!(f, "OVR"),
            I2CError::BERR => write!(f, "BERR")
        }
    }

}

impl<'a, S> I2c<'a, S>
where
    S: Any + I2C,
{
    /*
    /// By default I2C1 uses PB6 (SCL) and PB7 (SDA).
    /// This function allows us to remap it to PB8 (SCL) and PB9 (SDA).
    /// I2C2 only uses PB10 (SCL) and PB11 (SDA)
    fn use_remap(&self, afio: &AFIO) {
        let i2c = self.0;

        if i2c.get_type_id() == TypeId::of::<I2C1>() {
        }
    }*/

    /// Initialize the I2C port.
    /// 
    /// Initializes the GPIO ports of required by the I2C module an then starts it up using Fast Mode (400 kHz).
    /// 
    /// * `remap` - Specifies if the I2C module should use the alternative ports (only available for I2C1)
    pub fn init(&self, remap: bool, afio: &AFIO, gpio: &GPIOB, rcc: &RCC) {
        let i2c = self.0;

        // enable alternate function IO and IO port B
        rcc.apb2enr.modify(|_, w| {w.afioen().enabled().iopben().enabled()});

        if i2c.get_type_id() == TypeId::of::<I2C1>() {
            rcc.apb1enr.modify(|_, w| {w.i2c1en().enabled()});

            if remap {
                afio.mapr.modify(|_, w| {w.i2c1_remap().set_bit()});
                gpio.crh.modify(|_, w| {
                    w.mode8().output2().cnf8().alt_open().
                    mode9().output2().cnf9().alt_open()}); 
            } else {
                afio.mapr.modify(|_, w| {w.i2c1_remap().clear_bit()});

                // set RB6 (SCL) and RB7 (SDA) to alternative push pull and 
                // output 2 MHz
                gpio.crl.modify(|_, w| {
                    w.mode6().output2().cnf6().alt_open().
                    mode7().output2().cnf7().alt_open()});
            }
        }


        // set the apb frequency to 8MHz
        i2c.cr2.modify(|_, w| unsafe {w.freq().bits(8)});

        // enable FM mode (400KHz) set duty cycle to 1:1
        // ccr is calculated as T_PLCK1 = 125ns (because 8MHz frequency)
        // so 2500ns / 2 / 125ns = 10
        i2c.ccr.modify(|_, w| unsafe {w.f_s().set_bit().ccr().bits(10)});

        // alternative using 16:9 frequency by setting the duty bit
        //i2c.ccr.modify(|_, w| unsafe {w.f_s().set_bit().duty().set_bit().ccr().bits(1)});

        // set TRISE rise time
        // for SM mode it is 1000ns for FM mode it is 300ns
        // assuming T_PLCK1 = 125ns, 300ns / 125 ns ~ 2.4, round up to 3 and then +1
        i2c.trise.modify(|_, w| unsafe {w.trise().bits(4)});

        // enable the peripheral
        i2c.cr1.modify(|_, w| {w.pe().set_bit()});
    }

    fn get_error(&self, sr1: i2c1::sr1::R) -> I2CError {
        if sr1.timeout().bit_is_set() {
            return I2CError::Timeout
        } else if sr1.af().bit_is_set() {
            return I2CError::AF
        } else if sr1.arlo().bit_is_set() {
            return I2CError::ARLO
        } else if sr1.ovr().bit_is_set() {
            return I2CError::OVR
        } else if sr1.berr().bit_is_set() {
            return I2CError::BERR
        }

        I2CError::None
    }

    // reads the SB (Start Bit) of the Status 1 register
    pub fn is_start_flag_set(&self) -> I2CState {
        let state = self.0.sr1.read();
        let sb = state.sb().bit_is_set();
        self.get_error(state).if_no_err(|| {
            if sb { I2CState::Ok } else { I2CState::Busy }
        })
    }

    /// read the master mode flag
    pub fn is_msl_flag_set(&self) -> bool {
        self.0.sr2.read().msl().bit_is_set()
    }

    /// Determine if slave address matched
    pub fn is_addr_flag_set(&self) -> I2CState {
        let state = self.0.sr1.read();
        let addr = state.addr().bit_is_set();
        self.get_error(state).if_no_err(|| {
            if addr {
                I2CState::Ok
            } else {
                I2CState::Busy
            }
        }) 
    }

    /// Determine if byte transfer finished (BTF)
    pub fn is_byte_transfer_finished(&self) -> I2CState {
        let state = self.0.sr1.read();
        let btf = state.btf().bit_is_set();
        self.get_error(state).if_no_err(|| {
            if btf {I2CState::Ok} else {I2CState::Busy}
        })
    }

    /// Determine if data register is empty (TxE)
    pub fn is_data_register_empty(&self) -> I2CState {
        let state = self.0.sr1.read();
        self.get_error(state).if_no_err(|| {
            I2CState::Ok
        })
    }

    /// Determine if a byte has been received which can be read from the data register (RxNE)
    pub fn is_data_register_not_empty(&self) -> I2CState {
        let state = self.0.sr1.read();
        let rxne = state.rx_ne().bit_is_set();
        self.get_error(state).if_no_err(|| {
            if rxne { I2CState::Ok } else { I2CState::Busy }
        })
    }

    #[inline(always)]
    pub fn write_data(&self, dat : u8) -> I2CWriteState {
        self.0.dr.write(|w| unsafe { w.bits(dat as u32) });
        self.poll_loop(|| {self.is_data_register_empty()})
    }

    #[inline(always)]
    pub fn read_data(&self, out: &mut u8) -> I2CReadState {
        let state = self.poll_loop(|| { self.is_data_register_not_empty() });
        if state.is_ok() {
            *out = self.0.dr.read().bits() as u8;
        }
        state
    }

    #[inline(always)]
    pub fn read_last_data(&self, out: &mut u8) -> I2CReadState {
        self.0.cr1.modify(|_,w| {w.ack().clear_bit()});
        self.read_data(out)
    }

    pub fn poll_loop<T>(&self, fun: T) -> I2CState 
        where T : Fn() -> I2CState {
        loop {
            let state = fun();
            if !state.is_busy() { return state }
        }
    }

    /// Send the start signal and write the `addr` to the bus.
    /// 
    /// `read` specifies if it is a read request (`true`) or a write request (`false`)
    pub fn start_polling(&self, addr : u8, read : bool) -> I2CState {
        self.enable_start();

        self.poll_loop(|| { self.is_start_flag_set() }).cont(|| {
            self.write_data((addr << 1) + (if read { 1 } else { 0 }));
            if read {
                self.0.cr1.modify(|_,w| {w.pos().set_bit().ack().set_bit()});
            }
            self.poll_loop(|| {self.is_addr_flag_set()})
        }).cont(|| {
            self.0.sr2.read();
            I2CState::Ok
        })
    }

    #[inline(always)]
    pub fn start_write_polling(&self, addr : u8) -> I2CWriteState {
        self.start_polling(addr, false)
    }

    #[inline(always)]
    pub fn start_read_polling(&self, addr: u8) -> I2CReadState {
        let state = self.start_polling(addr, true);
        state
    }

    pub fn stop(&self) -> I2CState {
        self.enable_stop();
        I2CState::Ok
    }

/*
    pub fn write_data_polling(&self, dat : u8) -> I2CState {
        self.write_data(dat);
        self.poll_loop(|| {self.is_byte_transfer_finished()});
    } */

    #[inline(always)]
    fn is_busy(&self) -> bool {
        let i2c = self.0;
        i2c.sr2.read().busy().bit_is_set()
    }

    #[inline(always)]
    fn enable_start(&self) {
        let i2c = self. 0;
        i2c.cr1.modify(|_, w| {w.start().set_bit()});
    }

    #[inline(always)]
    fn enable_stop(&self) {
        let i2c = self.0;
        i2c.cr1.modify(|_, w| {w.stop().set_bit()});
    }
}