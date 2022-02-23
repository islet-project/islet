.global context_save
context_save:
	stp lr, xzr, [SP, #-16]!

	mov x0, x29
	mrs x1, ESR_EL2
	bl handle_exception
	// fall through

.global context_restore
context_restore:
	ldp lr, xzr, [SP], #16
	ret

.macro HANDLER source, kind
	.align 7
	stp x29, lr, [SP, #-16]!

	mov x29, \source
	movk x29, \kind, LSL #16
	bl context_save

	ldp x29, lr, [SP], #16
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
