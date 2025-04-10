#![no_main]

use islet_rmm::rec::Rec;
use islet_rmm::rmi::rec::run::Run;
use islet_rmm::rmi::{
    DATA_CREATE_UNKNOWN, DATA_DESTROY, GRANULE_DELEGATE, GRANULE_UNDELEGATE, REALM_ACTIVATE,
    REC_ENTER, RTT_INIT_RIPAS, RTT_READ_ENTRY, SUCCESS,
};
use islet_rmm::rsi::{ATTEST_TOKEN_CONTINUE, ATTEST_TOKEN_INIT, HOST_CALL, REALM_CONFIG};
use islet_rmm::test_utils::{mock, *};

use libfuzzer_sys::{arbitrary, fuzz_target};

#[derive(Debug, arbitrary::Arbitrary)]
enum RSICommand {
    HOST_CALL,
    REALM_CONFIG,
    ATTESTATION,
}

#[derive(Debug, arbitrary::Arbitrary)]
struct InvalidIPACmdFuzz {
    ipa: u64,
    cmd: RSICommand,
}

fuzz_target!(|data: InvalidIPACmdFuzz| {
    let ipa = data.ipa as usize;

    let rd = mock::host::realm_setup();

    let (rec1, run1) = (alloc_granule(IDX_REC1), alloc_granule(IDX_REC1_RUN));

    match data.cmd {
        RSICommand::HOST_CALL => {
            let _ret = rmi::<REC_ENTER>(&[rec1, run1, HOST_CALL, ipa]);
        }
        RSICommand::REALM_CONFIG => {
            let _ret = rmi::<REC_ENTER>(&[rec1, run1, REALM_CONFIG, ipa]);
        }
        RSICommand::ATTESTATION => {
            let _ret = rmi::<REC_ENTER>(&[rec1, run1, ATTEST_TOKEN_INIT, 0, 0, 0, 0, 0, 0, 0, 0]);

            let _ret = rmi::<REC_ENTER>(&[rec1, run1, ATTEST_TOKEN_CONTINUE, ipa, 0, 0]);
        }
    }

    mock::host::realm_teardown(rd);
});
