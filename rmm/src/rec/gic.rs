use super::Rec;
use crate::gic::*;
use crate::rmi::error::Error;
use crate::rmi::rec::run::Run;

use aarch64_cpu::registers::*;

pub fn init_gic(rec: &mut Rec<'_>) {
    let gic_state = &mut rec.context.gic_state;
    gic_state.ich_hcr_el2 = ICH_HCR_EL2_INIT;
}

pub fn restore_state(rec: &Rec<'_>) {
    let gic_state = &rec.context.gic_state;
    let nr_lrs = nr_lrs();
    let nr_aprs = nr_aprs();

    for i in 0..=nr_lrs {
        set_lr(i, gic_state.ich_lr_el2[i]);
    }
    for i in 0..=nr_aprs {
        set_ap0r(i, gic_state.ich_ap0r_el2[i]);
        set_ap1r(i, gic_state.ich_ap1r_el2[i]);
    }
    ICH_VMCR_EL2.set(gic_state.ich_vmcr_el2);
    ICH_HCR_EL2.set(gic_state.ich_hcr_el2);
}

pub fn save_state(rec: &mut Rec<'_>) {
    let gic_state = &mut rec.context.gic_state;
    let nr_lrs = nr_lrs();
    let nr_aprs = nr_aprs();

    for i in 0..=nr_lrs {
        gic_state.ich_lr_el2[i] = get_lr(i);
    }
    for i in 0..=nr_aprs {
        gic_state.ich_ap0r_el2[i] = get_ap0r(i);
        gic_state.ich_ap1r_el2[i] = get_ap1r(i);
    }

    gic_state.ich_vmcr_el2 = ICH_VMCR_EL2.get();
    gic_state.ich_hcr_el2 = ICH_HCR_EL2.get();
    gic_state.ich_misr_el2 = ICH_MISR_EL2.get();

    ICH_HCR_EL2.set(gic_state.ich_hcr_el2 & !(ICH_HCR_EL2::En.mask << ICH_HCR_EL2::En.shift));
}

pub fn receive_state_from_host(rec: &mut Rec<'_>, run: &Run) -> Result<(), Error> {
    let gic_state = &mut rec.context.gic_state;
    let nr_lrs = nr_lrs();

    gic_state.ich_lr_el2[..nr_lrs].copy_from_slice(&run.entry_gic_lrs()[..nr_lrs]);
    gic_state.ich_hcr_el2 &= !ICH_HCR_EL2_NS_MASK;
    gic_state.ich_hcr_el2 |= run.entry_gic_hcr() & ICH_HCR_EL2_NS_MASK;
    Ok(())
}

pub fn send_state_to_host(rec: &Rec<'_>, run: &mut Run) -> Result<(), Error> {
    let gic_state = &rec.context.gic_state;
    let nr_lrs = nr_lrs();

    run.exit_gic_lrs_mut()[..nr_lrs].copy_from_slice(&gic_state.ich_lr_el2[..nr_lrs]);
    run.set_gic_misr(gic_state.ich_misr_el2);
    run.set_gic_vmcr(gic_state.ich_vmcr_el2);
    run.set_gic_hcr(gic_state.ich_hcr_el2 & (ICH_HCR_EL2_EOI_COUNT_MASK | ICH_HCR_EL2_NS_MASK));
    Ok(())
}

pub fn validate_state(run: &Run) -> bool {
    let hcr = run.entry_gic_hcr();

    /* Validate rec_entry.gicv3_hcr MBZ bits */
    if (hcr & !ICH_HCR_EL2_NS_MASK) != 0 {
        return false;
    }

    for i in 0..nr_lrs() {
        let lrs = run.entry_gic_lrs();
        let lr = lrs[i];

        let state = (lr >> ICH_LR0_EL2::State.shift) & ICH_LR0_EL2::State.mask;
        let vintid = (lr >> ICH_LR0_EL2::vINTID.shift) & ICH_LR0_EL2::vINTID.mask;
        let priority = (lr >> ICH_LR0_EL2::Priority.shift) & ICH_LR0_EL2::Priority.mask;

        let pintid_mask = ICH_LR0_EL2::pINTID.mask << ICH_LR0_EL2::pINTID.shift;
        let eoi_mask = ICH_LR0_EL2::EOI.mask << ICH_LR0_EL2::EOI.shift;
        let only_eoi = pintid_mask & !eoi_mask;

        let hw = (lr >> ICH_LR0_EL2::HW.shift) & ICH_LR0_EL2::HW.mask;

        if state == ICH_LR0_EL2::State::Invalid.into() {
            continue;
        }

        /* The RMM Specification imposes the constraint that HW == '0' */
        if hw != 0
            /* Check RES0 bits in the Priority field */
            || priority & pri_res0_mask() != 0
            /* Only the EOI bit in the pINTID is allowed to be set */
            || lr & only_eoi != 0
            /* Check if vINTID is in the valid range */
            || !valid_vintid(vintid)
        {
            return false;
        }

        /*
         * Behavior is UNPREDICTABLE if two or more List Registers
         * specify the same vINTID.
         */
        for j in i + 1..=nr_lrs() {
            let lrs = run.entry_gic_lrs();
            let lr = lrs[j];

            let vintid_2 = (lr >> ICH_LR0_EL2::vINTID.shift) & ICH_LR0_EL2::vINTID.mask;
            let state = (lr >> ICH_LR0_EL2::State.shift) & ICH_LR0_EL2::State.mask;

            if state == ICH_LR0_EL2::State::Invalid.into() {
                continue;
            }

            if vintid == vintid_2 {
                return false;
            }
        }
    }

    true
}
