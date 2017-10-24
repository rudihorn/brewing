
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
