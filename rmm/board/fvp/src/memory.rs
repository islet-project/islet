const FVP_DRAM0_REGION: core::ops::Range<usize> = core::ops::Range {
    start: 0x8000_0000,
    end: 0x8000_0000 + 0x7C00_0000 - 1,
};
const FVP_DRAM1_REGION: core::ops::Range<usize> = core::ops::Range {
    start: 0x8_8000_0000,
    end: 0x8_8000_0000 + 0x8000_0000 - 1,
};
const GRANULE_SIZE: usize = 4096;

#[derive(Debug)]
pub struct FVPGranuleMap;
impl FVPGranuleMap {
    pub fn new() -> &'static FVPGranuleMap {
        &FVPGranuleMap {}
    }
}

impl monitor::rmm::granule::GranuleMemoryMap for FVPGranuleMap {
    fn addr_to_idx(&self, phys: usize) -> Result<usize, ()> {
        if phys % GRANULE_SIZE != 0 {
            warn!("address need to be aligned 0x{:X}", phys);
            return Err(());
        }

        if FVP_DRAM0_REGION.contains(&phys) {
            Ok((phys - FVP_DRAM0_REGION.start) / GRANULE_SIZE)
        } else if FVP_DRAM1_REGION.contains(&phys) {
            let num_dram0 = (FVP_DRAM0_REGION.end - FVP_DRAM0_REGION.start + 1) / GRANULE_SIZE;
            Ok(((phys - FVP_DRAM1_REGION.start) / GRANULE_SIZE) + num_dram0)
        } else {
            warn!("address is strange 0x{:X}", phys);
            Err(())
        }
    }

    fn max_granules(&self) -> usize {
        let num_dram0 = (FVP_DRAM0_REGION.end - FVP_DRAM0_REGION.start + 1) / GRANULE_SIZE;
        let num_dram1 = (FVP_DRAM1_REGION.end - FVP_DRAM1_REGION.start + 1) / GRANULE_SIZE;
        num_dram0 + num_dram1
    }
}