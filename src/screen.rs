
#[macro_use]
#[allow(unused_imports)]
use debug;
use stm32;
use ssd1306;

use i2c::I2c;
use rtfm::{Resource, Threshold};
use tslib::gpio::{GpioPinDefault, Pin8, Pin9};
use tslib::afio::{AfioI2C1Peripheral, NotConfigured};

pub fn init_screen<'a>(
    i2c1: &'a stm32::I2C1,
    pinb8: GpioPinDefault<'a, stm32::GPIOB, Pin8>, 
    pinb9: GpioPinDefault<'a, stm32::GPIOB, Pin9>,
    afio_i2c1: AfioI2C1Peripheral<'a, NotConfigured>) {
    
    let pinb8 = pinb8.set_output_10MHz().set_alt_output_open_drain();
    let pinb9 = pinb9.set_output_10MHz().set_alt_output_open_drain();

    let afio_i2c1 = afio_i2c1.set_remapped();
    let i2c1 = I2c(i2c1);
    let r = i2c1.start_init();

    let bsm = r.0.set_fast_mode(10);
    let freq = r.1.set(8); // configure frequency
    let trise = r.2.set(4); // configure rise time
    let ports = r.3.set_ports_remapped(pinb8, pinb9, afio_i2c1);
    i2c1.complete_init(bsm, freq, trise, ports);

    ssd1306::sync_init(&i2c1);
}  

pub fn set_address_mode<'a, S>(
    t: &mut Threshold,
    i2c1: &'a S) 
where
    S : Resource<Data = stm32::I2C1> {
    ssd1306::write_control_2(t, i2c1, &[0x20, 0]);   
}

pub fn set_address<'a, S>(
    t: &mut Threshold,
    i2c1: &'a S,
    column: u8,
    page: u8) 
where
    S : Resource<Data = stm32::I2C1> {
    ssd1306::write_control_2(t, i2c1, &[0x21, column, 127, 0x22, page, 7]);   
}

pub fn write_digit<'a, S>(
    t: &mut Threshold,
    i2c1: &'a S,
    num: u8)
where
    S : Resource<Data = stm32::I2C1> 
{
    let num = &ssd1306::NUMBERS[num as usize];
    ssd1306::write_data(t, i2c1, num);
    ssd1306::write_data(t, i2c1, &[0, 0]);
}

pub fn write_dot<'a, S>(
    t: &mut Threshold,
    i2c1: &'a S)
where
    S : Resource<Data = stm32::I2C1> 
{
    ssd1306::write_data(t, i2c1, &[0, 1, 0]);
}

pub fn write_empty_digit<'a, S>(
    t: &mut Threshold,
    i2c1: &'a S)
where
    S : Resource<Data = stm32::I2C1> 
{
    ssd1306::write_data(t, i2c1, &[0; 7]);
}

pub fn write_number<'a, S>(
    t: &mut Threshold,
    i2c1: &'a S,
    num: u32)
where
    S : Resource<Data = stm32::I2C1>
{
    let digit = num % 10;
    let rem = num / 10;

    if rem > 0 {
        write_number(t, i2c1, rem);
    }

    write_digit(t, i2c1, digit as u8);
} 