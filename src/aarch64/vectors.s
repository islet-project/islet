.global context_save
context_save:
	// Save all general purpose registers
	stp x26, x27, [SP, #-16]!
	stp x24, x25, [SP, #-16]!
	stp x22, x23, [SP, #-16]!
	stp x20, x21, [SP, #-16]!
	stp x18, x19, [SP, #-16]!
	stp x16, x17, [SP, #-16]!
	stp x14, x15, [SP, #-16]!
	stp x12, x13, [SP, #-16]!
	stp x10, x11, [SP, #-16]!
	stp x8, x9, [SP, #-16]!
	stp x6, x7, [SP, #-16]!
	stp x4, x5, [SP, #-16]!
	stp x2, x3, [SP, #-16]!
	stp x0, x1, [SP, #-16]!

// Save all SIMD/FP registers
	stp q30, q31, [SP, #-32]!
	stp q28, q29, [SP, #-32]!
	stp q26, q27, [SP, #-32]!
	stp q24, q25, [SP, #-32]!
	stp q22, q23, [SP, #-32]!
	stp q20, q21, [SP, #-32]!
	stp q18, q19, [SP, #-32]!
	stp q16, q17, [SP, #-32]!
	stp q14, q15, [SP, #-32]!
	stp q12, q13, [SP, #-32]!
	stp q10, q11, [SP, #-32]!
	stp q8, q9, [SP, #-32]!
	stp q6, q7, [SP, #-32]!
	stp q4, q5, [SP, #-32]!
	stp q2, q3, [SP, #-32]!
	stp q0, q1, [SP, #-32]!

	mrs x1, TPIDR_EL1
	mrs x0, SP_EL1
	stp x0, x1, [SP, #-16]!

	mrs x1, TPIDR_EL0
	mrs x0, SP_EL0
	stp x0, x1, [SP, #-16]!

	mrs x1, SPSR_EL2
	mrs x0, ELR_EL2
	stp x0, x1, [SP, #-16]!

	// Setup arguments to exception handler
	mov x0, x29
	mrs x1, ESR_EL2
	mov x2, SP

	stp lr, xzr, [SP, #-16]!
	bl handle_exception
	// fall through

.global context_restore
context_restore:
	ldp lr, xzr, [SP], #16

	ldp x0, x1, [SP], #16
	msr ELR_EL2, x0
	msr SPSR_EL2, x1

	ldp x0, x1, [SP], #16
	msr SP_EL0, x0
	msr TPIDR_EL0, x0

	ldp x0, x1, [SP], #16
	msr SP_EL1, x0
	msr TPIDR_EL1, x1

	ldp q0, q1, [SP], #32
	ldp q2, q3, [SP], #32
	ldp q4, q5, [SP], #32
	ldp q6, q7, [SP], #32
	ldp q8, q9, [SP], #32
	ldp q10, q11, [SP], #32
	ldp q12, q13, [SP], #32
	ldp q14, q15, [SP], #32
	ldp q16, q17, [SP], #32
	ldp q18, q19, [SP], #32
	ldp q20, q21, [SP], #32
	ldp q22, q23, [SP], #32
	ldp q24, q25, [SP], #32
	ldp q26, q27, [SP], #32
	ldp q28, q29, [SP], #32
	ldp q30, q31, [SP], #32

	ldp x0, x1, [SP], #16
	ldp x2, x3, [SP], #16
	ldp x4, x5, [SP], #16
	ldp x6, x7, [SP], #16
	ldp x8, x9, [SP], #16
	ldp x10, x11, [SP], #16
	ldp x12, x13, [SP], #16
	ldp x14, x15, [SP], #16
	ldp x16, x17, [SP], #16
	ldp x18, x19, [SP], #16
	ldp x20, x21, [SP], #16
	ldp x22, x23, [SP], #16
	ldp x24, x25, [SP], #16
	ldp x26, x27, [SP], #16

	ret

.macro HANDLER source, kind
	.align 7
	stp lr, xzr, [SP, #-16]!
	stp x28, x29, [SP, #-16]!

	mov x29, \source
	movk x29, \kind, LSL #16
	bl context_save

	ldp x28, x29, [SP], #16
	ldp lr, xzr, [SP], #16
	eret
.endm

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
	HANDLER 2, 0
	HANDLER 2, 1
	HANDLER 2, 2
	HANDLER 2, 3
	HANDLER 3, 0
	HANDLER 3, 1
	HANDLER 3, 2
	HANDLER 3, 3
