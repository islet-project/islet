# How to debug when console is not enabled in realm yet

This document is for debugging a realm code when we cannot use any debugging
tools or print* functions in realm. If it is stuck at some point in realm,
realm would never return control to RMM. In this case, you cannot figure out
upto what point the realm has been executing. This is where the debugging method
to be described comes into play. Note, description below is not a sophisticated
method. Please update this document with better solution.

## 1. Make an exception take place
Insert an instruction which raises an exception taken to EL2 RMM
in your code where you want to inspect.
For example, write an unused instruction 'smc #0xF0'.
Since the current implementation of RMM exection handler for the SMC trap
does not forward the exception to the normal world, using SMC instructions
are useful for debugging. (refer rmm/armv9a/src/exception/trap.rs)

## 2. Inspect realm context at RMM exception handler
Edit the handle_lower_exception() function in rmm/armv9a/src/exception/trap.rs
to inspect realm context (i.e., cpu registers).
You may want to add lines below:
```
 debug!("{:?}\nESR: {:X}\n{:#X?}", info, esr, tf);
```
### How to identify code location of realm
If you want to inspect multiple code locations in realm, you need to identify
each location where exception is raised.

#### Distinguishing by immediate values in SMC instruction
You may use different immediate value
for each SMC instruction and individually identify them at RMM by reading
the ISS encoding(bits[15:0]) in the ESR_EL2 register.
```
let esr_el2: u64 = ESR_EL2.get();
let smc_imm: u64 = esr_els & 0xFF;
```

#### Distinguishing by the realm pc (==ELR_EL2) reigster
You can distinguish them by the pc register, but you may not able to identify
the corresponding address of each instructions that you inserted until you look
into the instruction in the address.
1. Identify its Intermediate Physical Address(IPA) using instructions below:

    rmm/armv9a/src/helper/regs.rs
    ```
    + define_sys_register!(PAR_EL1);
    ```

    rmm/armv9a/src/exception/trap.rs
    ```
     79 pub extern "C" fn handle_lower_exception(
     80     info: Info,
     81     esr: u32,
     82     vcpu: &mut VCPU<Context>,
     83     tf: &mut TrapFrame,
     84 ) -> u64 {
    + 85     unsafe {
    + 86         let elr: u64 = vcpu.context.elr;
    + 87         let ipa;
    + 88         let pa;
    + 89         llvm_asm! { "at s1e1r, $0": : "r"(elr) };
    + 90         ipa = PAR_EL1.get() & (((1 << 48) - 1) & !0xFFF);
    + 91         llvm_asm! { "at s12e1r, $0": : "r"(elr) };
    + 92         pa = PAR_EL1.get() & (((1 << 48) - 1) & !0xFFF);
    + 93         debug!("ELR_EL2:{:#X} -> IPA:{:#X} -> PA:{:#X}",
    + 94                 elr, ipa, pa);
    + 95     };
    ```

    example:
    ```
    [DEBUG]armv9a::exception::trap -- ELR_EL2: ffffffc0086a10e8 -> IPA: 408A10E8 -> PA: 8872670E8
    ```

2. Inspect the address using the qemu monitor

    2.1 Start over the qemu with '-S' option which stops VM after creating a VM
    and setting vcpu contexts for it.

    ```
    ../qemu-system-aarch64 \
            -kernel Image_realmvm \
            -initrd initramfs-busybox-aarch64.cpio.gz \
            -append "console=ttyAMA0" \
            --enable-kvm \
            -cpu host \
            -smp 2 \
            -M virt,gic-version=3 \
            -m 256M \
            -S \
            -nographic
    ```

    2.2 Enter <Ctrl-a c> to switch from the serial console to the qemu monitor.
    Then dump the code with the IPA to inspect.

    ```
    xp/{number-of-linues}i {IPA-you-want-to-inspect}
    ```
    example:
    ```
    (qemu) xp/20i 0x408A10CC
    ffffffc0086a10cc:   94000bbf    bl  ffffffc0086a3fc8 <time_init>
    ffffffc0086a10d0:   d40008c3    smc #0x46
    ...
    ffffffc0086a10e4:   940031ca    bl  ffffffc0086ad80c <call_function_init>
    ffffffc0086a10e8:   d4000923    smc #0x49
    ffffffc0086a10ec:   97e5cc99    bl  ffffffc008014350 <arch_local_save_flags>
    ...
    ```
    FYI. The default start guest physical address(==IPA) of DRAM is 0x40000000
    which is determined by qemu. The address contains the code for bootloader
    that qemu writes if we don't give the bootloader option to qemu and run kernel
    directly.
    ```
    (qemu) xp/10i 0x40000000
    x0000000040000000: 580000c0 ldr x0, pc+24 (addr 0x40000018)
    0x0000000040000004: aa1f03e1 mov x1, xzr
    0x0000000040000008: aa1f03e2 mov x2, xzr
    0x000000004000000c: aa1f03e3 mov x3, xzr
    0x0000000040000010: 58000084 ldr x4, pc+16 (addr 0x40000020)
    0x0000000040000014: d61f0080 br x4
    ```
    Refer the code at qemu/hw/arm/boot.c
    ```
      static const ARMInsnFixup bootloader_aarch64[] = {
          { 0x580000c0 }, /* ldr x0, arg ; Load the lower 32-bits of DTB */
          { 0xaa1f03e1 }, /* mov x1, xzr */
          { 0xaa1f03e2 }, /* mov x2, xzr */
          { 0xaa1f03e3 }, /* mov x3, xzr */
          { 0x58000084 }, /* ldr x4, entry ; Load the lower 32-bits of kernel entry */
          { 0xd61f0080 }, /* br x4      ; Jump to the kernel entry point */
          { 0, FIXUP_ARGPTR_LO }, /* arg: .word @DTB Lower 32-bits */
          { 0, FIXUP_ARGPTR_HI}, /* .word @DTB Higher 32-bits */
          { 0, FIXUP_ENTRYPOINT_LO }, /* entry: .word @Kernel Entry Lower 32-bits */
          { 0, FIXUP_ENTRYPOINT_HI }, /* .word @Kernel Entry Higher 32-bits */
          { 0, FIXUP_TERMINATOR }
    ```

