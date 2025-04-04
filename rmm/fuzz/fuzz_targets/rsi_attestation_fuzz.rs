#![no_main]

use islet_rmm::granule::GRANULE_SIZE;
use islet_rmm::rec::Rec;
use islet_rmm::rmi::rec::run::Run;
use islet_rmm::rmi::{
    DATA_CREATE_UNKNOWN, DATA_DESTROY, GRANULE_DELEGATE, GRANULE_UNDELEGATE, REALM_ACTIVATE,
    REC_ENTER, RTT_INIT_RIPAS, RTT_READ_ENTRY, SUCCESS,
};
use islet_rmm::rsi::{ATTEST_TOKEN_CONTINUE, ATTEST_TOKEN_INIT};
use islet_rmm::test_utils::{mock, *};

use libfuzzer_sys::{arbitrary, fuzz_target, Corpus};

#[derive(Debug, arbitrary::Arbitrary)]
struct TokenContinueFuzz {
    offset: u64,
    size: u64,
}

#[derive(Debug, arbitrary::Arbitrary)]
struct AttestFuzz {
    ipa: u64,
    challenge: [u64; 8],

    /* Fuzz multiple ATTEST_TOKEN_CONTINUE calls */
    tokens: Vec<TokenContinueFuzz>,
}

fuzz_target!(|data: AttestFuzz| -> Corpus {
    let ipa = data.ipa as usize;
    let challenge = &data.challenge;
    let top = match ipa.checked_add(L3_SIZE) {
        Some(x) => x,
        None => {
            return Corpus::Reject;
        }
    };

    let rd = mock::host::realm_unactivated_setup();
    let (rec1, run1) = (alloc_granule(IDX_REC1), alloc_granule(IDX_REC1_RUN));

    let data_granule = alloc_granule(IDX_DATA1);
    let mut fuzz_ret = Corpus::Keep;

    /* Reject IPAs which cannot be mapped */
    let ret = rmi::<RTT_READ_ENTRY>(&[rd, ipa, MAP_LEVEL]);
    if ret[0] != SUCCESS {
        mock::host::realm_teardown(rd);
        return Corpus::Reject;
    }

    mock::host::map(rd, ipa);

    let ret = rmi::<RTT_INIT_RIPAS>(&[rd, ipa, top]);
    if ret[0] != SUCCESS {
        mock::host::unmap(rd, ipa, false);
        mock::host::realm_teardown(rd);
        return Corpus::Reject;
    }

    let ret = rmi::<REALM_ACTIVATE>(&[rd]);
    assert_eq!(ret[0], SUCCESS);

    let _ret = rmi::<GRANULE_DELEGATE>(&[data_granule]);

    let ret = rmi::<DATA_CREATE_UNKNOWN>(&[rd, data_granule, ipa]);
    if ret[0] == SUCCESS {
        let challenges = &[
            challenge[0] as usize,
            challenge[1] as usize,
            challenge[2] as usize,
            challenge[3] as usize,
            challenge[4] as usize,
            challenge[5] as usize,
            challenge[6] as usize,
            challenge[7] as usize,
        ];

        let rsi_call = RecEnterFuzzCall {
            cmd: ATTEST_TOKEN_INIT,
            args: challenges,
        };

        let rsi_call_ptr = (&rsi_call as *const RecEnterFuzzCall) as usize;

        let _ret = rmi::<REC_ENTER>(&[rec1, run1, rsi_call_ptr]);

        for token in data.tokens {
            let offset = token.offset as usize;
            let size = token.size as usize;

            let rsi_call = RecEnterFuzzCall {
                cmd: ATTEST_TOKEN_CONTINUE,
                args: &[ipa, offset, size],
            };

            let rsi_call_ptr = (&rsi_call as *const RecEnterFuzzCall) as usize;

            let _ret = rmi::<REC_ENTER>(&[rec1, run1, rsi_call_ptr]);
        }

        let ret = rmi::<DATA_DESTROY>(&[rd, ipa]);
        assert_eq!(ret[0], SUCCESS);
    } else {
        fuzz_ret = Corpus::Reject;
    }

    let _ret = rmi::<GRANULE_UNDELEGATE>(&[data_granule]);

    mock::host::unmap(rd, ipa, false);

    mock::host::realm_teardown(rd);

    fuzz_ret
});
