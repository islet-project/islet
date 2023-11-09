use crate::measurement::{Measurement, MeasurementError};
use crate::realm::Realm;

use crate::event::RsiHandle;
use crate::realm::context::Context;
use crate::rmm_el3::{plat_token, realm_attest_key};
use crate::rsi::attestation::Attestation;
use crate::rsi::error::Error as RsiError;

use alloc::collections::BTreeMap;
use alloc::sync::Arc;
use spin::mutex::Mutex;
use spinning_top::Spinlock;

type RealmMutex = Arc<Mutex<Realm<Context>>>;
type RealmMap = BTreeMap<usize, RealmMutex>;
pub static RMS: Spinlock<(usize, RealmMap)> = Spinlock::new((0, BTreeMap::new()));

pub fn get_realm(id: usize) -> Option<RealmMutex> {
    RMS.lock().1.get(&id).map(|realm| Arc::clone(realm))
}

impl crate::rsi::Interface for RsiHandle {
    fn measurement_read(
        &self,
        realmid: usize,
        index: usize,
        out: &mut crate::measurement::Measurement,
    ) -> Result<(), crate::rsi::error::Error> {
        let realm_lock = get_realm(realmid).ok_or(RsiError::RealmDoesNotExists)?;

        let mut realm = realm_lock.lock();

        let measurement = realm
            .measurements
            .iter_mut()
            .nth(index)
            .ok_or(RsiError::InvalidMeasurementIndex)?;

        out.as_mut_slice().copy_from_slice(measurement.as_slice());
        Ok(())
    }

    fn measurement_extend(
        &self,
        realmid: usize,
        index: usize,
        f: impl Fn(&mut crate::measurement::Measurement) -> Result<(), MeasurementError>,
    ) -> Result<(), crate::rsi::error::Error> {
        let realm_lock = get_realm(realmid).ok_or(RsiError::RealmDoesNotExists)?;

        let mut realm = realm_lock.lock();

        let measurement = realm
            .measurements
            .iter_mut()
            .nth(index)
            .ok_or(RsiError::InvalidMeasurementIndex)?;

        f(measurement)?;
        Ok(())
    }

    fn get_attestation_token(
        &self,
        attest_pa: usize,
        challenge: &[u8],
        measurements: &[Measurement],
        hash_algo: u8,
    ) -> usize {
        // TODO: consider storing attestation object somewhere,
        // as RAK and token do not change during rmm lifetime.
        let token = Attestation::new(&plat_token(), &realm_attest_key()).create_attestation_token(
            challenge,
            measurements,
            hash_algo,
        );

        unsafe {
            let pa_ptr = attest_pa as *mut u8;
            core::ptr::copy(token.as_ptr(), pa_ptr, token.len());
        }

        token.len()
    }
}
