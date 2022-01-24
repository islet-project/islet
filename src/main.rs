/*
 * Copyright (c) 2020 Samsung Electronics Co., Ltd. All Rights Reserved.
 *
 * PROPRIETARY/CONFIDENTIAL
 * This software is the confidential and proprietary information of
 * Samsung Electronics Co., Ltd. ("Confidential Information").
 * You shall not disclose such Confidential Information and
 * shall use it only in accordance with the terms of the license agreement
 * you entered into with Samsung Electronics Co., Ltd. (“SAMSUNG”).
 * SAMSUNG MAKES NO REPRESENTATIONS OR WARRANTIES ABOUT
 * THE SUITABILITY OF THE SOFTWARE, EITHER EXPRESS OR IMPLIED,
 * INCLUDING BUT NOT LIMITED TO THE IMPLIED WARRANTIES OF
 * MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE,
 * OR NON-INFRINGEMENT. SAMSUNG SHALL NOT BE LIABLE
 * FOR ANY DAMAGES SUFFERED BY LICENSEE AS A RESULT OF USING,
 * MODIFYING OR DISTRIBUTING THIS SOFTWARE OR ITS DERIVATIVES.
 */

#![no_std]
#![no_main]
#![feature(const_fn)]
#![feature(const_fn_fn_ptr_basics)]
#![feature(const_mut_refs)]
#![feature(llvm_asm)]
#![feature(alloc_error_handler)]
#![warn(rust_2018_idioms)]

pub mod config;
pub mod driver;
pub mod io;
pub mod panic;
pub mod rmi;

use crate::config::RMM_STACK_SIZE;
use crate::io::{stdout, Write};

#[no_mangle]
static mut COLD_BOOT: bool = true;

#[no_mangle]
#[link_section = ".stack"]
static mut RMM_STACK: [u8; RMM_STACK_SIZE] = [0; RMM_STACK_SIZE];

#[link_section = ".head.text"]
#[no_mangle]
unsafe extern "C" fn rmm_entry() -> ! {
    llvm_asm! {
        "
		ldr x0, =__RMM_STACK_END__
		mov sp, x0

		bl setup

		1:
		bl main
		b 1b
        "
        : : : : "volatile"
    }

    loop {}
}

extern "C" {
    static __BSS_START__: usize;
    static __BSS_SIZE__: usize;
}

#[no_mangle]
#[allow(unused)]
unsafe fn clear_bss() {
    let bss = core::slice::from_raw_parts_mut(
        &__BSS_START__ as *const usize as *mut u64,
        &__BSS_SIZE__ as *const usize as usize / core::mem::size_of::<u64>(),
    );
    bss.fill(0);
}

unsafe fn init_console() {
    let _ = stdout().attach(driver::uart::pl011::device());

    let _ = stdout().write_all("RMM: initialized the console!\n".as_bytes());
}

pub fn rmm_exit() {
    unsafe {
        rmi::smc(rmi::RMM_REQ_COMPLETE, 0);
    }
}

#[no_mangle]
#[allow(unused)]
unsafe fn setup() {
    if COLD_BOOT {
        clear_bss();
        init_console();

        COLD_BOOT = false;
    }
}

#[no_mangle]
#[allow(unused)]
unsafe fn main() -> ! {
    let _ = stdout().write_all("RMM: booted on core!\n".as_bytes());

    loop {
        rmm_exit();
        let _ = stdout().write_all("RMM: invoked!\n".as_bytes());
    }
}
