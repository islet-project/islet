.equ VCPU_GP_REGS, 0
.equ VCPU_SYS_REGS, 264
.equ VCPU_FP_REGS, 472

.macro save_volatile_to_stack
	stp x29, x30, [SP, #-16]!
	stp x17, x18, [SP, #-(8*12)]!
	stp x15, x16, [SP, #-16]!
	stp x13, x14, [SP, #-16]!
	stp x11, x12, [SP, #-16]!
	stp x9, x10, [SP, #-16]!
	stp x7, x8, [SP, #-16]!
	stp x5, x6, [SP, #-16]!
	stp x3, x4, [SP, #-16]!
	stp x1, x2, [SP, #-16]!

	mrs x18, spsr_el2
	stp x18, x0, [SP, #-16]!

	mrs x18, elr_el2
	stp xzr, x18, [SP, #-16]!
.endm

.macro save_volatile_to_vcpu
	/* Save all general purpose registers */

	str x18, [sp, #-16]!

	mrs x18, tpidr_el2
	stp x0, x1, [x18, #VCPU_GP_REGS + 8 * 0]
	stp x2, x3, [x18, #VCPU_GP_REGS + 8 * 2]
	stp x4, x5, [x18, #VCPU_GP_REGS + 8 * 4]
	stp x6, x7, [x18, #VCPU_GP_REGS + 8 * 6]
	stp x8, x9, [x18, #VCPU_GP_REGS + 8 * 8]
	stp x10, x11, [x18, #VCPU_GP_REGS + 8 * 10]
	stp x12, x13, [x18, #VCPU_GP_REGS + 8 * 12]
	stp x14, x15, [x18, #VCPU_GP_REGS + 8 * 14]
	stp x16, x17, [x18, #VCPU_GP_REGS + 8 * 16]
	stp x19, x20, [x18, #VCPU_GP_REGS + 8 * 19]
	stp x21, x22, [x18, #VCPU_GP_REGS + 8 * 21]
	stp x23, x24, [x18, #VCPU_GP_REGS + 8 * 23]
	stp x25, x26, [x18, #VCPU_GP_REGS + 8 * 25]
	stp x27, x28, [x18, #VCPU_GP_REGS + 8 * 27]
	stp x29, x30, [x18, #VCPU_GP_REGS + 8 * 29]

	ldr x0, [sp], #16
	str x0, [x18, #VCPU_GP_REGS + 8 * 18]

	/* Save return address & mode */
	mrs x1, elr_el2
	mrs x2, spsr_el2
	stp x1, x2, [x18, #VCPU_GP_REGS + 8 * 31]
.endm

.global restore_all_from_vcpu_and_run
restore_all_from_vcpu_and_run:
	mrs x0, tpidr_el2

	/* Restore system registers */
	/* Use x28 as the base */
	add x28, x0, #VCPU_SYS_REGS

	ldr x3, [x28], #8
	/* msr sctlr_el2, x2 */
	msr sp_el1, x3

	ldp x2, x3, [x28], #16
	msr sp_el0, x2
	msr esr_el1, x3

	ldp x2, x3, [x28], #16
	msr vbar_el1, x2
	msr ttbr0_el1, x3

	ldp x2, x3, [x28], #16
	msr ttbr1_el1, x2
	msr mair_el1, x3

	ldp x2, x3, [x28], #16
	msr amair_el1, x2
	msr tcr_el1, x3

	ldp x2, x3, [x28], #16
	msr tpidr_el1, x2
	msr tpidr_el0, x3

	ldp x2, x3, [x28], #16
	msr tpidrro_el0, x2
	msr actlr_el1, x3

	ldp x2, x3, [x28], #16
	msr vmpidr_el2, x2
	msr csselr_el1, x3

	ldp x2, x3, [x28], #16
	msr cpacr_el1, x2
	msr afsr0_el1, x3

	ldp x2, x3, [x28], #16
	msr afsr1_el1, x2
	msr far_el1, x3

	ldp x2, x3, [x28], #16
	msr contextidr_el1, x2
	msr cntkctl_el1, x3

	ldp x2, x3, [x28], #16
	msr par_el1, x2
	msr vttbr_el2, x3

	ldp x2, x3, [x28], #16
	msr esr_el2, x2
	msr hpfar_el2, x3

	ldr x3, [x28], #8
	msr sctlr_el1, x3
	/* TODO: invalidate TLB */

	/* Intentional fallthrough */
.global restore_nonvolatile_from_vcpu_and_run
restore_nonvolatile_from_vcpu_and_run:
	/* Restore non-volatile registers. */
	ldp x19, x20, [x0, #VCPU_GP_REGS + 8 * 19]
	ldp x21, x22, [x0, #VCPU_GP_REGS + 8 * 21]
	ldp x23, x24, [x0, #VCPU_GP_REGS + 8 * 23]
	ldp x25, x26, [x0, #VCPU_GP_REGS + 8 * 25]
	ldp x27, x28, [x0, #VCPU_GP_REGS + 8 * 27]

	/* Intentional fallthrough */
.global restore_volatile_from_vcpu_and_run
restore_volatile_from_vcpu_and_run:
	ldp x4, x5, [x0, #VCPU_GP_REGS + 8 * 4]
	ldp x6, x7, [x0, #VCPU_GP_REGS + 8 * 6]
	ldp x8, x9, [x0, #VCPU_GP_REGS + 8 * 8]
	ldp x10, x11, [x0, #VCPU_GP_REGS + 8 * 10]
	ldp x12, x13, [x0, #VCPU_GP_REGS + 8 * 12]
	ldp x14, x15, [x0, #VCPU_GP_REGS + 8 * 14]
	ldp x16, x17, [x0, #VCPU_GP_REGS + 8 * 16]
	ldr x18, [x0, #VCPU_GP_REGS + 8 * 18]
	ldp x29, x30, [x0, #VCPU_GP_REGS + 8 * 29]

	ldp x1, x2, [x0, #VCPU_GP_REGS + 8 * 31]
	msr elr_el2, x1
	msr spsr_el2, x2

	ldp x2, x3, [x0, #VCPU_GP_REGS + 8 * 2]
	ldp x0, x1, [x0, #VCPU_GP_REGS + 8 * 0]
	eret
	dsb nsh
	isb

.global restore_volatile_from_stack_and_return
restore_volatile_from_stack_and_return:
	ldp xzr, x18, [SP], #16
	msr ELR_EL2, x18

	ldp x18, x0, [SP], #16
	msr SPSR_EL2, x18

	ldp x1, x2, [SP], #16
	ldp x3, x4, [SP], #16
	ldp x5, x6, [SP], #16
	ldp x7, x8, [SP], #16
	ldp x9, x10, [SP], #16
	ldp x11, x12, [SP], #16
	ldp x13, x14, [SP], #16
	ldp x15, x16, [SP], #16
	ldp x17, x18, [SP], #(8*12)
	ldp x29, x30, [SP], #16
	eret

.macro HANDLER source, kind
	.align 7
	save_volatile_to_stack

	mov x0, \source
	movk x0, \kind, LSL #16
	mrs x1, ESR_EL2
	mov x2, SP

	bl handle_exception
	b restore_volatile_from_stack_and_return
.endm

.macro HANDLER_LOWER source, kind
	.align 7
	save_volatile_to_vcpu

	/* Setup arguments to exception handler */
	mov x0, \source
	movk x0, \kind, LSL #16
	mrs x1, ESR_EL2
	mrs x2, TPIDR_EL2
	mov x3, SP

	bl handle_lower_exception

	/* Enter to rmm */
	/* vcpu will be switched by rmm if needed */
	cbnz x0, rmm_enter

	mrs x0, tpidr_el2
	b restore_nonvolatile_from_vcpu_and_run
.endm

.global rmm_enter
rmm_enter:
	/* Save non-volatile registers */
	mrs x1, tpidr_el2
	stp x19, x20, [x1, #VCPU_GP_REGS + 8 * 19]
	stp x21, x22, [x1, #VCPU_GP_REGS + 8 * 21]
	stp x23, x24, [x1, #VCPU_GP_REGS + 8 * 23]
	stp x25, x26, [x1, #VCPU_GP_REGS + 8 * 25]
	stp x27, x28, [x1, #VCPU_GP_REGS + 8 * 27]

	/* Save system registers */
	/* Use x28 as the base */
	add x28, x1, #VCPU_SYS_REGS

	/* mrs x2, sctlr_el2 */
	mrs x3, sp_el1
	str x3, [x28], #8

	mrs x2, sp_el0
	mrs x3, esr_el1
	stp x2, x3, [x28], #16

	mrs x2, vbar_el1
	mrs x3, ttbr0_el1
	stp x2, x3, [x28], #16

	mrs x2, ttbr1_el1
	mrs x3, mair_el1
	stp x2, x3, [x28], #16

	mrs x2, amair_el1
	mrs x3, tcr_el1
	stp x2, x3, [x28], #16

	mrs x2, tpidr_el1
	mrs x3, tpidr_el0
	stp x2, x3, [x28], #16

	mrs x2, tpidrro_el0
	mrs x3, actlr_el1
	stp x2, x3, [x28], #24

	mrs x3, csselr_el1
	str x3, [x28], #8

	mrs x2, cpacr_el1
	mrs x3, afsr0_el1
	stp x2, x3, [x28], #16

	mrs x2, afsr1_el1
	mrs x3, far_el1
	stp x2, x3, [x28], #16

	mrs x2, contextidr_el1
	mrs x3, cntkctl_el1
	stp x2, x3, [x28], #16

	mrs x2, par_el1
	mrs x3, vttbr_el2
	stp x2, x3, [x28], #16

	mrs x2, esr_el2
	mrs x3, hpfar_el2
	stp x2, x3, [x28], #16

	mrs x3, sctlr_el1
	str x3, [x28], #8
	/* TODO: FP_REGS */

	/* load three more registers to match with TrapFrame */
	ldr xzr, [SP], #8
	ldr xzr, [SP], #8
	ldr xzr, [SP], #8

	ldr x0, [SP], #8
	ldp x1, x2, [SP], #16
	ldp x3, x4, [SP], #16
	ldp x5, x6, [SP], #16
	ldp x7, x8, [SP], #16
	ldp x9, x10, [SP], #16
	ldp x11, x12, [SP], #16
	ldp x13, x14, [SP], #16
	ldp x15, x16, [SP], #16
	ldp x17, x18, [SP], #16
	ldp x19, x20, [SP], #16
	ldp x21, x22, [SP], #16
	ldp x23, x24, [SP], #16
	ldp x25, x26, [SP], #16
	ldp x27, x28, [SP], #16
	ldp x29, x30, [SP], #16
	ret

.global rmm_exit
rmm_exit:
	stp x29, x30, [SP, #-16]!
	stp x27, x28, [SP, #-16]!
	stp x25, x26, [SP, #-16]!
	stp x23, x24, [SP, #-16]!
	stp x21, x22, [SP, #-16]!
	stp x19, x20, [SP, #-16]!
	stp x17, x18, [SP, #-16]!
	stp x15, x16, [SP, #-16]!
	stp x13, x14, [SP, #-16]!
	stp x11, x12, [SP, #-16]!
	stp x9, x10, [SP, #-16]!
	stp x7, x8, [SP, #-16]!
	stp x5, x6, [SP, #-16]!
	stp x3, x4, [SP, #-16]!
	stp x1, x2, [SP, #-16]!
	str x0, [SP, #-8]!

	/* store three more registers to match with TrapFrame */
	str xzr, [SP, #-8]!
	str xzr, [SP, #-8]!
	str xzr, [SP, #-8]!

	b restore_all_from_vcpu_and_run

.align 11
.global vectors
vectors:
	HANDLER 0, 0
	HANDLER 0, 1
	HANDLER 0, 2
	HANDLER 0, 3
	HANDLER 1, 0
	HANDLER 1, 1
	HANDLER 1, 2
	HANDLER 1, 3
	HANDLER_LOWER 2, 0
	HANDLER_LOWER 2, 1
	HANDLER_LOWER 2, 2
	HANDLER_LOWER 2, 3
	HANDLER_LOWER 3, 0
	HANDLER_LOWER 3, 1
	HANDLER_LOWER 3, 2
	HANDLER_LOWER 3, 3
