use crate::event::{Context, RsiHandle};
use crate::rmi::error::Error;
use crate::rmi::rec::run::Run;
use crate::rmi::rec::Rec;
use crate::Monitor;
use crate::{rmi, rsi};

#[derive(Debug)]
pub enum Kind {
    RSI = 0,
    IRQ = 1,
    FIQ = 2,
    SERROR = 3,
    InstAbort = 4,
    DataAbort = 5,
    UndefinedSync,
}

impl Into<u64> for Kind {
    fn into(self) -> u64 {
        self as u64
    }
}

impl From<usize> for Kind {
    fn from(num: usize) -> Self {
        match num {
            0 => Kind::RSI,
            1 => Kind::IRQ,
            2 => Kind::FIQ,
            3 => Kind::SERROR,
            4 => Kind::InstAbort,
            5 => Kind::DataAbort,
            _ => Kind::UndefinedSync,
        }
    }
}

pub fn handle_realm_exit(
    realm_exit_res: [usize; 4],
    rmm: &Monitor,
    rec: &mut Rec,
    run: &mut Run,
    realm_id: usize,
) -> Result<(bool, usize), Error> {
    let rmi = rmm.rmi;
    let mut return_to_ns = true;
    let ret = match Kind::from(realm_exit_res[0]) {
        Kind::RSI => {
            trace!("REC_ENTER ret: {:#X?}", realm_exit_res);
            let rsi = &rmm.rsi;
            let cmd = realm_exit_res[1];
            let mut ret = rmi::SUCCESS;

            rsi::constraint::validate(cmd, |_, ret_num| {
                let mut rsi_ctx = Context::new(cmd);
                rsi_ctx.resize_ret(ret_num);

                // set default value
                if rsi.dispatch(&mut rsi_ctx, rmm, rec, run) == RsiHandle::RET_SUCCESS {
                    if rsi_ctx.ret_slice()[0] == rmi::SUCCESS_REC_ENTER {
                        return_to_ns = false;
                    }
                    ret = rsi_ctx.ret_slice()[0];
                } else {
                    return_to_ns = false;
                }
            });
            ret
        }
        Kind::InstAbort | Kind::DataAbort => unsafe {
            run.set_exit_reason(rmi::EXIT_SYNC);
            run.set_esr(realm_exit_res[1] as u64);
            run.set_hpfar(realm_exit_res[2] as u64);
            run.set_far(realm_exit_res[3] as u64);
            let _ = rmi.send_mmio_write(realm_id, rec.id(), run);
            rmi::SUCCESS
        },
        Kind::IRQ => unsafe {
            run.set_exit_reason(rmi::EXIT_IRQ);
            run.set_esr(realm_exit_res[1] as u64);
            run.set_hpfar(realm_exit_res[2] as u64);
            run.set_far(realm_exit_res[3] as u64);
            rmi::SUCCESS
        },
        Kind::UndefinedSync => unsafe {
            run.set_exit_reason(rmi::EXIT_SYNC);
            run.set_esr(realm_exit_res[1] as u64);
            run.set_hpfar(realm_exit_res[2] as u64);
            run.set_far(realm_exit_res[3] as u64);
            rmi::SUCCESS
        },
        _ => rmi::SUCCESS,
    };
    Ok((return_to_ns, ret))
}
