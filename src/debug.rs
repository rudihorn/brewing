
#[macro_export]
macro_rules! iprint {
    ($s:expr) => {
        #[allow(unused_unsafe)]
        let stim = unsafe { &mut cortex_m::peripheral::Peripherals::steal().ITM.stim[0] };
        cortex_m::itm::write_str(stim, $s);
    };

    ($($arg:tt)*) => {
        #[allow(unused_unsafe)]
        let stim = unsafe { &mut cortex_m::peripheral::Peripherals::steal().ITM.stim[0] };
        cortex_m::itm::write_fmt(stim, format_args!($($arg)*));
    };
}

/// Macro for sending a formatted string through an ITM channel, with a newline.
#[macro_export]
macro_rules! iprintln {
    () => {
        iprint!("\n");
    };

    ($fmt:expr) => {
        iprint!(concat!($fmt, "\n"));
    };

    ($fmt:expr, $($arg:tt)*) => {
        iprint!(concat!($fmt, "\n"), $($arg)*);
    };
}