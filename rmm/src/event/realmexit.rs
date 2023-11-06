const REC_EXIT_REASON_MASK: usize = 15; // 0b1111

const EXIT_SYNC_TYPE_SHIFT: usize = 4;
const EXIT_SYNC_TYPE_MASK: usize = 15 << EXIT_SYNC_TYPE_SHIFT; // 0b1111_0000

/// Handles 'RET_TO_RMM' cases from trap::handle_lower_exception()
///
/// It can't cover all of RmiRecExitReason types.
/// Because the following types of RmiRecExitReason
/// are set to ExitReason using RSI:
/// - RMI_EXIT_RIPAS_CHANGE
/// - RMI_EXIT_HOST_CALL
#[derive(Debug)]
#[repr(usize)]
pub enum RecExitReason {
    Sync(ExitSyncType),
    IRQ = 1,
    FIQ = 2,
    PSCI = 3,
    SError = 4,
    Undefined = REC_EXIT_REASON_MASK, // fixed, 0b1111
}

impl Into<u64> for RecExitReason {
    fn into(self) -> u64 {
        match self {
            RecExitReason::Sync(exit_sync_type) => exit_sync_type.into(),
            RecExitReason::IRQ => 1,
            RecExitReason::FIQ => 2,
            RecExitReason::PSCI => 3,
            RecExitReason::SError => 4,
            RecExitReason::Undefined => 7,
        }
    }
}

impl From<usize> for RecExitReason {
    fn from(num: usize) -> Self {
        match num & REC_EXIT_REASON_MASK {
            0 => RecExitReason::Sync(ExitSyncType::from(num)),
            1 => RecExitReason::IRQ,
            2 => RecExitReason::FIQ,
            3 => RecExitReason::PSCI,
            4 => RecExitReason::SError,
            _ => RecExitReason::Undefined,
        }
    }
}

/// Fault Status Code only for the 'RecExitReason::Sync'
/// it's a different from trap::Syndrome
#[derive(Debug)]
#[repr(usize)]
pub enum ExitSyncType {
    RSI = 1 << EXIT_SYNC_TYPE_SHIFT,
    DataAbort = 2 << EXIT_SYNC_TYPE_SHIFT,
    InstAbort = 3 << EXIT_SYNC_TYPE_SHIFT,
    Undefined = EXIT_SYNC_TYPE_MASK, // fixed, 0b1111_0000
}

impl Into<u64> for ExitSyncType {
    fn into(self) -> u64 {
        self as u64
    }
}

impl From<usize> for ExitSyncType {
    fn from(num: usize) -> Self {
        let masked_val = (num & EXIT_SYNC_TYPE_MASK) >> EXIT_SYNC_TYPE_SHIFT;
        match masked_val {
            1 => ExitSyncType::RSI,
            2 => ExitSyncType::DataAbort,
            3 => ExitSyncType::InstAbort,
            _ => ExitSyncType::Undefined,
        }
    }
}
