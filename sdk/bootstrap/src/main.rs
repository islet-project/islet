#![no_std]
#![no_main]
#![feature(llvm_asm)]
#![feature(naked_functions)]
#![warn(rust_2018_idioms)]
pub mod panic;

#[naked]
#[link_section = ".head.text"]
#[no_mangle]
unsafe extern "C" fn bootstrap_entry() {
    #![allow(unsupported_naked_functions)]
    //vm_create
    llvm_asm! {
        "
        smc 0x0
        "
        : : "{x0}"(0xc000_0003 as usize), "{x1}"(1) : : "volatile"
    }
    //vm_set_reg
    llvm_asm! {
        "
        smc 0x0
        "
        : : "{x0}"(0xc000_0009 as usize),
            "{x1}"(0), "{x2}"(0), "{x3}"(31), "{x4}"(0x4008_1000): : "volatile"
    }
    //vm_switch
    llvm_asm! {
        "
        smc 0x0
        "
        : : "{x0}"(0xc000_0004 as usize), "{x1}"(0), "{x2}"(0) : : "volatile"
    }

    //vm_map_memory
    llvm_asm! {
        "
        smc 0x0
        "
        : : "{x0}"(0xc000_0007 as usize),
            "{x1}"(0), "{x2}"(0x1c0a_0000), "{x3}"(0x1c0a_0000), "{x4}"(0x1000) : : "volatile"
    }

    loop {
        let ret0: usize;
        let ret1: usize;
        //vm_run
        llvm_asm! {
            "
            smc 0x0
            "
            : "={x0}"(ret0), "={x1}"(ret1) : "{x0}"(0xc000_000b as usize) : : "volatile"
        }

        if ret0 == 1 {
            //RET_PAGE_FAULT
            //vm_map_memory
            llvm_asm! {
                "
                smc 0x0
                "
                : : "{x0}"(0xc000_0007 as usize),
                    "{x1}"(0), "{x2}"(ret1 & !(0xfffusize)) : : "volatile"
            }
        } else if ret0 != 0 {
            break;
        }
    }

    //To prevent optimization to remove realm_test
    llvm_asm! {
        "
        b .
        b realm_test
        "
        : : "{x0}"(0xc000_0005 as usize) : : "volatile"
    }
}

#[naked]
#[no_mangle]
pub unsafe extern "C" fn realm_test() -> ! {
    #![allow(unsupported_naked_functions)]
    loop {
        llvm_asm! {
            "
            hvc #0
            "
        }
    }
}
