use crate::event::Mainloop;
use crate::granule::{set_granule, GranuleState};
use crate::listen;
use crate::rmi;
use crate::rmi::error::Error;
use crate::rmi::rtt::validate_ipa;
use crate::rmi::Rd;
use crate::{get_granule, get_granule_if};

pub fn set_event_handler(mainloop: &mut Mainloop) {
    // Sets up a local channel between the client and server realm.
    listen!(mainloop, rmi::LOCAL_CHANNEL_SETUP, |arg, _ret, rmm| {
        use crate::asm::{smc, SMC_SUCCESS};
        use crate::granule::{is_granule_aligned, GRANULE_SIZE};
        use vmsa::error::Error as MmError;

        let mut realm_id_list: [Option<usize>; 2] = [None; 2];
        let rd_list: [usize; 2] = [arg[0], arg[1]];
        let lc_pa_start = arg[2];
        let lc_ipa_start = arg[3];
        let lc_size = arg[4];
        let lc_ipa_end = lc_ipa_start + lc_size;
        let lc_pa_end = lc_pa_start + lc_size;

        trace!(
            "LOCAL_CHANNEL_SETUP: client_rd 0x{:X}, server_rd 0x{:X}, lc_pa: 0x{:X}, lc_ipa: 0x{:X}, lc_size: 0x{:X}",
            rd_list[0],
            rd_list[1],
            lc_pa_start,
            lc_ipa_start,
            lc_size,
        );

        if !is_granule_aligned(lc_pa_start)
            || !is_granule_aligned(lc_ipa_start)
            || !is_granule_aligned(lc_size)
        {
            error!("Wrong alignment");
            return Err(Error::RmiErrorInput);
        }

        for (idx, rd) in rd_list.iter().enumerate() {
            let mut rd_granule = get_granule_if!(*rd, GranuleState::RD)?;
            let rd = rd_granule.content_mut::<Rd>();
            realm_id_list[idx] = Some(rd.id());

            validate_ipa(lc_ipa_start, rd.ipa_bits())?;
            if lc_size > GRANULE_SIZE {
                validate_ipa(lc_ipa_end - GRANULE_SIZE, rd.ipa_bits())?;
            }
        }

        trace!(
            "LOCAL_CHANNEL_SETUP: Set local channel granule to GranuleState::Data with stage 2 mapping in both realms"
        );
        for cur_lc_pa in (lc_pa_start..lc_pa_end).step_by(GRANULE_SIZE) {
            let offset = cur_lc_pa - lc_pa_start;

            let mut lc_g = match get_granule_if!(cur_lc_pa, GranuleState::Undelegated) {
                Err(MmError::MmNoEntry) => {
                    set_state_and_get_granule!(cur_lc_pa, GranuleState::Undelegated)
                }
                other => other,
            }?;

            if smc(rmi::gpt::MARK_REALM, &[cur_lc_pa])[0] != SMC_SUCCESS {
                return Err(Error::RmiErrorInput);
            }

            rmm.page_table.map(cur_lc_pa, true);
            set_granule(&mut lc_g, GranuleState::Delegated).map_err(|e| {
                rmm.page_table.unmap(cur_lc_pa);
                e
            })?;

            for realm_id in realm_id_list {
                let realm_id = match realm_id {
                    Some(r_id) => r_id,
                    None => return Err(Error::RmiErrorInput),
                };
                crate::rtt::data_create(realm_id, lc_ipa_start + offset, cur_lc_pa)?;
            }
            set_granule(&mut lc_g, GranuleState::Data)?;
        }

        Ok(())
    });
}
