pub const CORRECT_MILLIS: f64 = 1e-3;
pub const CORRECT_MICRO: f64 = 1e-6;
pub const CORRECT_NANO: f64 = 1e-9;
pub const CORRECT_PICO: f64 = 1e-12;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum CalibrationObject {
    Gain,
    Offset,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum CalibrationKind {
    CurrentAdc,
    VoltageAdc,
    ShuntResistance,
    RsCorrection,
    CurrentDac,
    VoltageDac,
}
