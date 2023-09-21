use crate::measurement::MeasurementError;

#[derive(Debug)]
pub enum Error {
    RealmDoesNotExists,
    InvalidMeasurementIndex,
    MeasurementError(MeasurementError),
}

impl From<MeasurementError> for Error {
    fn from(value: MeasurementError) -> Self {
        Self::MeasurementError(value)
    }
}
