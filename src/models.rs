use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct Calibration {
    pub sampling_rates: Vec<SamplingRate>,
    pub boards: Vec<Board>,
}

#[derive(Debug, Deserialize)]
pub struct SamplingRate {
    pub name: String,
    pub id: u32,
    pub values: Vec<f64>,
    pub commlib_indexes: Vec<u32>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Board {
    pub board_number: u32,
    pub current_dac: Vec<RangeBlock>,
    pub voltage_adc: Vec<RangeBlock>,
    pub shunt_resistance: Vec<RangeBlock>,
    pub rs_correction: Vec<RangeBlock>,
    pub current_adc: Vec<RangeBlock>,
    pub voltage_dac: Vec<RangeBlock>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct RangeBlock {
    pub range_name: String,
    pub range_id: u32,
    pub sampling_rates: Vec<RangeSamplingRate>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct RangeSamplingRate {
    pub sr_id: u32,
    pub calibrations: Values,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Values {
    #[serde(default)]
    pub gains: Vec<f64>,
    #[serde(default)]
    pub offsets: Vec<f64>,
}

pub fn read_calibtations<I: AsRef<Path>>(
    path: I,
) -> Result<Calibration, Box<dyn std::error::Error>> {
    Ok(toml::from_str(&std::fs::read_to_string(path)?)?)
}

#[cfg(test)]
mod models_tests {
    use crate::models::read_calibtations;

    #[test]
    fn deserialize() -> Result<(), Box<dyn std::error::Error>> {
        let calib = read_calibtations("src\\assets\\syncropatch.toml");
        println!("{:#?}", calib);
        Ok(())
    }
}
