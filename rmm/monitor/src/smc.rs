use core::arch::asm;

pub fn call(command: usize, args: [usize; 4]) -> [usize; 8] {
    let mut ret: [usize; 8] = [0usize; 8];

    unsafe {
        asm!(
            "smc #0x0",
            inlateout("x0") command => ret[0],
            inlateout("x1") args[0] => ret[1],
            inlateout("x2") args[1] => ret[2],
            inlateout("x3") args[2] => ret[3],
            inlateout("x4") args[3] => ret[4],
            out("x5") ret[5],
            out("x6") ret[6],
            out("x7") ret[7],
        )
    }
    ret
}
