#[derive(Debug)]
pub enum MeasurementError {
    InvalidHashAlgorithmValue(u8),
    OutputBufferTooSmall,
}
