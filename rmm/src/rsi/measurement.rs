use crate::measurement::MeasurementError;
use crate::realm::registry::get_realm;
use crate::rsi::error::Error;

pub fn read(
    realmid: usize,
    index: usize,
    out: &mut crate::measurement::Measurement,
) -> Result<(), crate::rsi::error::Error> {
    let realm_lock = get_realm(realmid).ok_or(Error::RealmDoesNotExists)?;

    let mut realm = realm_lock.lock();

    let measurement = realm
        .measurements
        .iter_mut()
        .nth(index)
        .ok_or(Error::InvalidMeasurementIndex)?;

    out.as_mut_slice().copy_from_slice(measurement.as_slice());
    Ok(())
}

pub fn write(
    realmid: usize,
    index: usize,
    val: &crate::measurement::Measurement,
) -> Result<(), crate::rsi::error::Error> {
    let realm_lock = get_realm(realmid).ok_or(Error::RealmDoesNotExists)?;

    let mut realm = realm_lock.lock();

    let measurement = realm
        .measurements
        .iter_mut()
        .nth(index)
        .ok_or(Error::InvalidMeasurementIndex)?;

    measurement.as_mut_slice().copy_from_slice(val.as_slice());
    Ok(())
}

pub fn extend(
    realmid: usize,
    index: usize,
    f: impl Fn(&mut crate::measurement::Measurement) -> Result<(), MeasurementError>,
) -> Result<(), crate::rsi::error::Error> {
    let realm_lock = get_realm(realmid).ok_or(Error::RealmDoesNotExists)?;

    let mut realm = realm_lock.lock();

    let measurement = realm
        .measurements
        .iter_mut()
        .nth(index)
        .ok_or(Error::InvalidMeasurementIndex)?;

    f(measurement)?;
    Ok(())
}
