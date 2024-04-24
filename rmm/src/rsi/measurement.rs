use crate::measurement::MeasurementError;
use crate::rsi::error::Error;
use crate::rsi::Rd;

pub fn read(
    rd: &Rd,
    index: usize,
    out: &mut crate::measurement::Measurement,
) -> Result<(), crate::rsi::error::Error> {
    let measurement = rd
        .measurements
        .iter()
        .nth(index)
        .ok_or(Error::InvalidMeasurementIndex)?;

    out.as_mut_slice().copy_from_slice(measurement.as_slice());
    Ok(())
}

pub fn extend(
    rd: &mut Rd,
    index: usize,
    f: impl Fn(&mut crate::measurement::Measurement) -> Result<(), MeasurementError>,
) -> Result<(), crate::rsi::error::Error> {
    let measurement = rd
        .measurements
        .iter_mut()
        .nth(index)
        .ok_or(Error::InvalidMeasurementIndex)?;

    f(measurement)?;
    Ok(())
}
