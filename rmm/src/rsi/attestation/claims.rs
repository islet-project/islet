use core::fmt::Debug;

use alloc::{string::String, vec::Vec};
use ciborium::Value;
use tinyvec::ArrayVec;

use crate::measurement::{Measurement, MEASUREMENTS_SLOT_NR, MEASUREMENTS_SLOT_RIM};

pub const CHALLENGE_LABEL: u64 = 10;
pub const PERSONALIZATION_VALUE_LABEL: u64 = 44235;
pub const INITIAL_MEASUREMENT_LABEL: u64 = 44238;
pub const EXTENSIBLE_MEASUREMENTS_LABEL: u64 = 44239;
pub const HASH_ALGO_ID_LABEL: u64 = 44236;
pub const PUBLIC_KEY_LABEL: u64 = 44237;
pub const PUBLIC_KEY_HASH_ALOG_ID_LABEL: u64 = 44240;

#[derive(Clone, Copy, Debug, Default)]
pub struct MeasurementEntry(Measurement, usize);

#[derive(Clone, Debug)]
pub struct Data<T: Default, const N: usize>(ArrayVec<[T; N]>);

impl<T: Copy + Default, const N: usize> Data<T, N> {
    pub fn from_slice(slice: &[T]) -> Self {
        Data(slice.iter().cloned().collect())
    }
}

pub const REM_SLOT_NR: usize = MEASUREMENTS_SLOT_NR - 1;

pub type Challenge = Data<u8, 64>;
pub type PersonalizationValue = Data<u8, 64>;
pub type REMs = Data<MeasurementEntry, REM_SLOT_NR>;
pub type RIM = MeasurementEntry;
pub type HashAlgo = String;
pub type RAKPubKey = Data<u8, 97>;

#[derive(Clone, Debug)]
pub struct Claim<T> {
    value: T,
    label: u64,
}

impl<T: Into<Value>> From<Claim<T>> for (Value, Value) {
    fn from(value: Claim<T>) -> Self {
        (Value::Integer(value.label.into()), value.value.into())
    }
}

impl From<MeasurementEntry> for Value {
    fn from(value: MeasurementEntry) -> Self {
        Value::Bytes(value.0.as_ref()[..value.1].to_vec())
    }
}

impl<const N: usize> From<Data<u8, N>> for Value {
    fn from(value: Data<u8, N>) -> Self {
        Value::Bytes(value.0.to_vec())
    }
}

impl<T: Into<Value>, const N: usize> From<Data<T, N>> for Value
where
    T: Default,
{
    default fn from(value: Data<T, N>) -> Self {
        let mut array = Vec::new();
        for el in value.0.into_iter() {
            array.push(el.into());
        }
        array.into()
    }
}

#[derive(Clone, Debug)]
pub struct RealmClaims {
    pub challenge: Claim<Challenge>,
    pub personalization_value: Claim<PersonalizationValue>,
    pub rim: Claim<RIM>,
    pub rems: Claim<REMs>,
    pub measurement_hash_algo: Claim<HashAlgo>,
    pub rak_pub: Claim<RAKPubKey>,
    pub rak_pub_hash_algo: Claim<HashAlgo>,
}

impl RealmClaims {
    pub fn init(
        challenge: &[u8],
        personalization_val: &[u8],
        measurements: &[Measurement],
        measurement_hash_algo: String,
        key_pub: &[u8],
        key_pub_hash_algo: String,
    ) -> RealmClaims {
        let challenge_claim: Claim<Challenge> = Claim {
            label: CHALLENGE_LABEL,
            value: Data::from_slice(challenge),
        };

        let personalization_value: Claim<PersonalizationValue> = Claim {
            label: PERSONALIZATION_VALUE_LABEL,
            value: Data::from_slice(personalization_val),
        };

        let measurement_size = match measurement_hash_algo.as_str() {
            "sha-256" => 32,
            "sha-512" => 64,
            _ => panic!("Unexpected hash algo id {}", measurement_hash_algo),
        };

        let rim: Claim<RIM> = Claim {
            label: INITIAL_MEASUREMENT_LABEL,
            value: MeasurementEntry(measurements[MEASUREMENTS_SLOT_RIM], measurement_size),
        };

        let mut rems_data = [MeasurementEntry::default(); REM_SLOT_NR];
        for i in 0..REM_SLOT_NR {
            rems_data[i] = MeasurementEntry(measurements[i + 1], measurement_size);
        }

        let rems: Claim<REMs> = Claim {
            label: EXTENSIBLE_MEASUREMENTS_LABEL,
            value: Data::from_slice(&rems_data),
        };

        let hash_algo_id: Claim<HashAlgo> = Claim {
            label: HASH_ALGO_ID_LABEL,
            value: measurement_hash_algo,
        };

        let rak_pub: Claim<RAKPubKey> = Claim {
            label: PUBLIC_KEY_LABEL,
            value: Data::from_slice(key_pub),
        };

        let rak_pub_hash_algo: Claim<HashAlgo> = Claim {
            label: PUBLIC_KEY_HASH_ALOG_ID_LABEL,
            value: key_pub_hash_algo,
        };

        Self {
            challenge: challenge_claim,
            personalization_value,
            rim,
            rems,
            measurement_hash_algo: hash_algo_id,
            rak_pub,
            rak_pub_hash_algo,
        }
    }
}
