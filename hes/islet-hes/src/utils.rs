use alloc::vec::Vec;
use ciborium::Value;

use crate::Measurement;

/// Keeps all platform token tag values based on RSS
pub mod token_tag {
    /* Claims */
    pub const CCA_PLAT_CHALLENGE: u32 = 10;
    pub const CCA_PLAT_INSTANCE_ID: u32 = 256;
    pub const CCA_PLAT_PROFILE: u32 = 265;
    pub const CCA_PLAT_SECURITY_LIFECYCLE: u32 = 2395;
    pub const CCA_PLAT_IMPLEMENTATION_ID: u32 = 2396;
    pub const CCA_PLAT_SW_COMPONENTS: u32 = 2399;
    pub const CCA_PLAT_VERIFICATION_SERVICE: u32 = 2400;
    pub const CCA_PLAT_CONFIGURATION: u32 = 2401;
    pub const CCA_PLAT_HASH_ALGO_DESC: u32 = 2402;

    /* Software components */
    pub const CCA_SW_COMP_TITLE: u32 = 1;
    pub const CCA_SW_COMP_MEASUREMENT_VALUE: u32 = 2;
    pub const CCA_SW_COMP_VERSION: u32 = 4;
    pub const CCA_SW_COMP_SIGNER_ID: u32 = 5;
    pub const CCA_SW_COMP_HASH_ALGORITHM: u32 = 6;
}

fn encode_measurement(measurement: &Measurement) -> Value {
    let mut map: Vec<(Value, Value)> = Vec::with_capacity(5);

    map.push((
        Value::Integer(token_tag::CCA_SW_COMP_TITLE.into()),
        Value::Text(measurement.metadata.sw_type.clone()),
    ));
    map.push((
        Value::Integer(token_tag::CCA_SW_COMP_HASH_ALGORITHM.into()),
        Value::Text(measurement.metadata.algorithm.into()),
    ));
    map.push((
        Value::Integer(token_tag::CCA_SW_COMP_MEASUREMENT_VALUE.into()),
        Value::Bytes(measurement.value.to_vec()),
    ));
    map.push((
        Value::Integer(token_tag::CCA_SW_COMP_VERSION.into()),
        Value::Text(measurement.metadata.sw_version.clone()),
    ));
    map.push((
        Value::Integer(token_tag::CCA_SW_COMP_SIGNER_ID.into()),
        Value::Bytes(measurement.metadata.signer_id.to_vec()),
    ));

    Value::Map(map)
}

pub fn encode_measurements(measurements: &[Measurement]) -> Value {
    let mut array: Vec<Value> = Vec::with_capacity(measurements.len());

    for measurement in measurements {
        array.push(encode_measurement(measurement));
    }

    Value::Array(array)
}
