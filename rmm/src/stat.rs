use crate::allocator;
use crate::rmi;
use crate::rsi;
use spin::mutex::Mutex;

//NOTE: RMI, RSI_CMD_MAX are should be updated whenever there is a new command
//      which is bigger than the current max value of the commands.
//      But if RMI, RSI commands are handled by 'Enum', then it can be fixed
//      by using the max enum value like MAX_KIND
const RMI_CMD_MIN: usize = rmi::VERSION;
const RMI_CMD_MAX: usize = rmi::RTT_SET_RIPAS;
const RMI_CMD_CNT: usize = RMI_CMD_MAX - RMI_CMD_MIN + 1;

const RSI_CMD_MIN: usize = rsi::ABI_VERSION;
const RSI_CMD_MAX: usize = rsi::HOST_CALL;
const RSI_CMD_CNT: usize = RSI_CMD_MAX - RSI_CMD_MIN + 1;

const MAX_CMD_CNT: usize = max(RMI_CMD_CNT, RSI_CMD_CNT);
const MAX_KIND: usize = Kind::Undefined as usize;

static mut COLLECTED_STAT_CNT: u64 = 0;

const fn max(a: usize, b: usize) -> usize {
    if a >= b {
        a
    } else {
        b
    }
}

lazy_static! {
    pub static ref STATS: Mutex<Stats> = Mutex::new(Stats::new());
}

#[inline(always)]
fn is_rmi_cmd(cmd: usize) -> bool {
    RMI_CMD_MIN <= cmd && cmd <= RMI_CMD_MAX
}

#[inline(always)]
fn is_rsi_cmd(cmd: usize) -> bool {
    RSI_CMD_MIN <= cmd && cmd <= RSI_CMD_MAX
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Stats {
    list: [Stat; MAX_KIND],
}

impl Stats {
    fn new() -> Self {
        let mut stats = Stats {
            list: [Stat::default(); MAX_KIND],
        };

        for i in 0..MAX_KIND {
            stats.list[i] = Stat::new(Kind::from(i));
        }
        stats
    }

    fn get_stat(&mut self, cmd: usize) -> Result<&mut Stat, Error> {
        let kind = Kind::get_kind(cmd)?;
        Ok(&mut self.list[kind as usize])
    }

    pub fn measure<F: FnMut()>(&mut self, cmd: usize, mut handler: F) {
        if let Err(e) = self.start(cmd) {
            info!("stats.start is failed: {:?} with cmd {:x}", e, cmd);
        }

        handler();

        if let Err(e) = self.end(cmd) {
            info!("stats.end is failed: {:?} with cmd {:x}", e, cmd);
        }
    }

    fn start(&mut self, cmd: usize) -> Result<(), Error> {
        let stat = self.get_stat(cmd)?;
        trace!("start Stat with command {}", stat.cmd_to_str(cmd));
        stat.start(cmd)
    }

    fn end(&mut self, cmd: usize) -> Result<(), Error> {
        let stat = self.get_stat(cmd)?;

        if stat.overflowed {
            return Err(Error::IntegerOverflow);
        }

        if let Err(Error::IntegerOverflow) = stat.update(cmd) {
            stat.overflowed = true;
            return Err(Error::IntegerOverflow);
        }

        unsafe {
            COLLECTED_STAT_CNT = COLLECTED_STAT_CNT.wrapping_add(1);
            if COLLECTED_STAT_CNT % 10 == 0 {
                self.print();
            }
        };

        Ok(())
    }

    fn print(&self) {
        info!("=============================================== STATS::PRINT() START =============================================");
        for i in 0..MAX_KIND {
            if let Err(e) = self.list[i].print_all() {
                info!("Failed to print Stats: {:?}", e);
            }
        }

        info!(
            "TOTAL MemUsed in RMM HEAP: {: >12} byte",
            allocator::get_used_size()
        );
        info!("=============================================== STATS::PRINT() END ===============================================");
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Default)]
struct Stat {
    kind: Kind,
    overflowed: bool,
    cur_cmd: Option<usize>,
    call_cnt: [u64; MAX_CMD_CNT],

    mem_used_before: Option<i64>,
    total_mem_used: [i64; MAX_CMD_CNT],
}

impl Stat {
    fn new(kind: Kind) -> Self {
        Stat {
            kind,
            ..Default::default()
        }
    }
    fn cmd_to_str(&self, cmd: usize) -> &'static str {
        match self.kind {
            Kind::RMI => rmi::to_str(cmd),
            Kind::RSI => rsi::to_str(cmd),
            _ => "Undefined",
        }
    }

    fn print_mem_stat(&self, idx: usize, cmd: usize, avg_mem_used: i64) {
        info!(
            "{:?}::{: <25} TOTAL MemUsed: {: >12} byte, AVG MemUsed: {: >10} byte, CallCnt: {: >8}",
            self.kind,
            self.cmd_to_str(cmd),
            self.total_mem_used[idx],
            avg_mem_used,
            self.call_cnt[idx]
        );
    }

    fn print(&self, cmd: usize) -> Result<(), Error> {
        let idx = self.get_idx(cmd)?;

        if self.call_cnt[idx] == 0 {
            return Ok(());
        }

        if self.total_mem_used[idx] == 0 {
            self.print_mem_stat(idx, cmd, 0);
            return Ok(());
        }

        let avg_mem_used = self.total_mem_used[idx].checked_div(self.call_cnt[idx] as i64);
        if avg_mem_used.is_none() {
            return Err(Error::IntegerOverflow);
        }

        self.print_mem_stat(idx, cmd, avg_mem_used.unwrap());

        Ok(())
    }

    fn print_all(&self) -> Result<(), Error> {
        let cmd_min = match self.kind {
            Kind::RMI => RMI_CMD_MIN,
            Kind::RSI => RSI_CMD_MIN,
            _ => return Ok(()),
        };

        for i in 0..MAX_CMD_CNT {
            self.print(cmd_min + i)?;
        }
        Ok(())
    }

    fn start(&mut self, cmd: usize) -> Result<(), Error> {
        if self.mem_used_before.is_some() || self.cur_cmd.is_some() {
            return Err(Error::MismatchedSequence);
        }
        self.mem_used_before = Some(allocator::get_used_size() as i64);
        self.cur_cmd = Some(cmd);

        Ok(())
    }

    fn get_idx(&self, cmd: usize) -> Result<usize, Error> {
        let idx = match self.kind {
            Kind::RMI => cmd.checked_sub(RMI_CMD_MIN),
            Kind::RSI => cmd.checked_sub(RSI_CMD_MIN),
            _ => return Err(Error::Unknown),
        };
        if idx.is_none() {
            error!("get_idx is failed with: {:?}", Error::IntegerOverflow);
            return Err(Error::IntegerOverflow);
        }

        Ok(idx.unwrap())
    }

    fn update(&mut self, cmd: usize) -> Result<(), Error> {
        if self.mem_used_before.is_none() || self.cur_cmd.is_none() {
            return Err(Error::MismatchedSequence);
        }
        if cmd != self.cur_cmd.unwrap() {
            return Err(Error::MismatchedCommand);
        }

        let idx = self.get_idx(cmd)?;
        let mem_used_after = allocator::get_used_size() as i64;
        let mem_used_before = self.mem_used_before.unwrap();
        if mem_used_after.checked_sub(mem_used_before).is_none() {
            error!(
                "Failed to subtract mem_used_before {:x} from mem_used_after {:x}",
                mem_used_before, mem_used_after
            );
            return Err(Error::IntegerOverflow);
        }
        let cur_mem_used = mem_used_after - mem_used_before;

        if self.total_mem_used[idx].checked_add(cur_mem_used).is_none() {
            error!(
                "Failed to add {:x} to total_mem_used[{}] {:x}",
                cur_mem_used, idx, self.total_mem_used[idx]
            );
            return Err(Error::IntegerOverflow);
        }
        self.total_mem_used[idx] += cur_mem_used;

        if self.call_cnt[idx].checked_add(1).is_none() {
            error!(
                "Failed to add 1 to call_cnt[{}] {:x}",
                idx, self.call_cnt[idx]
            );
            return Err(Error::IntegerOverflow);
        }
        self.call_cnt[idx] += 1;

        if let Err(e) = self.print(cmd) {
            error!("error to print Stat {:?}, {}", e, self.cmd_to_str(cmd));
        }

        // clean up after updating
        self.mem_used_before = None;
        self.cur_cmd = None;

        Ok(())
    }
}

#[derive(Copy, Clone, Debug)]
pub enum Kind {
    RMI = 0,
    RSI = 1,
    Undefined,
}

impl Default for Kind {
    fn default() -> Self {
        Kind::Undefined
    }
}

impl From<usize> for Kind {
    fn from(num: usize) -> Self {
        match num {
            0 => Kind::RMI,
            1 => Kind::RSI,
            _ => Kind::Undefined,
        }
    }
}

impl Kind {
    fn get_kind(cmd: usize) -> Result<Self, Error> {
        if is_rmi_cmd(cmd) {
            return Ok(Kind::RMI);
        } else if is_rsi_cmd(cmd) {
            return Ok(Kind::RSI);
        }

        Err(Error::InvalidCommand)
    }
}

#[derive(Debug)]
pub enum Error {
    IntegerOverflow = 1,
    InvalidCommand = 2,
    MismatchedCommand = 3,
    MismatchedSequence = 4,
    Unknown,
}
