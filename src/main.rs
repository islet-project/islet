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
#[link_section = ".stack"]
static mut RMM_STACK: [u8; RMM_STACK_SIZE] = [0; RMM_STACK_SIZE];

#[link_section = ".head.text"]
#[no_mangle]
unsafe extern "C" fn rmm_entry() -> ! {
    llvm_asm! {
        "
		ldr x0, =__RMM_STACK_END__
		mov sp, x0
        bl main
        "
        : : : : "volatile"
    }
    loop {}
}

extern "C" {
    static __BSS_START__: usize;
    static __BSS_SIZE__: usize;
}

pub unsafe fn clear_bss() {
	let bss = core::slice::from_raw_parts_mut(
		&__BSS_START__ as *const usize as *mut u64,
		&__BSS_SIZE__  as *const usize as usize / core::mem::size_of::<u64>(),
	);
	bss.fill(0);
}

pub fn rmm_exit() {
    unsafe {
        rmi::smc(rmi::RMM_REQ_COMPLETE, 0);
    }
}

#[allow(dead_code)]
#[no_mangle]
fn main() -> ! {
	unsafe {
		clear_bss();
	}

    unsafe {
        let _ = stdout().write_all("Hello World!\n".as_bytes());
    }

    loop {
        rmm_exit();
    }
}
