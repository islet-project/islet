/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Copyright (C) 2023 ARM Ltd.
 */

#ifndef __SMC_RSI_H_
#define __SMC_RSI_H_

/*
 * This file describes the Realm Services Interface (RSI) Application Binary
 * Interface (ABI) for SMC calls made from within the Realm to the RMM and
 * serviced by the RMM.
 */

#define SMC_RSI_CALL_BASE		0xC4000000

/*
 * The major version number of the RSI implementation.  Increase this whenever
 * the binary format or semantics of the SMC calls change.
 */
#define RSI_ABI_VERSION_MAJOR		1

/*
 * The minor version number of the RSI implementation.  Increase this when
 * a bug is fixed, or a feature is added without breaking binary compatibility.
 */
#define RSI_ABI_VERSION_MINOR		0

#define RSI_ABI_VERSION			((RSI_ABI_VERSION_MAJOR << 16) | \
					 RSI_ABI_VERSION_MINOR)

#define RSI_LEGACY_ABI_VERSION		0xc0000

#define RSI_ABI_VERSION_GET_MAJOR(_version) ((_version) >> 16)
#define RSI_ABI_VERSION_GET_MINOR(_version) ((_version) & 0xFFFF)

#define RSI_SUCCESS			0
#define RSI_ERROR_INPUT			1
#define RSI_ERROR_STATE			2
#define RSI_INCOMPLETE			3

#define SMC_RSI_FID(_x)			(SMC_RSI_CALL_BASE + (_x))

#define SMC_RSI_ABI_VERSION			SMC_RSI_FID(0x190)

/*
 * arg1 == The IPA of token buffer
 * arg2 == Challenge value, bytes:  0 -  7
 * arg3 == Challenge value, bytes:  7 - 15
 * arg4 == Challenge value, bytes: 16 - 23
 * arg5 == Challenge value, bytes: 24 - 31
 * arg6 == Challenge value, bytes: 32 - 39
 * arg7 == Challenge value, bytes: 40 - 47
 * arg8 == Challenge value, bytes: 48 - 55
 * arg9 == Challenge value, bytes: 56 - 63
 * ret0 == Status / error
 */
#define SMC_RSI_ATTESTATION_TOKEN_INIT		SMC_RSI_FID(0x194)

/*
 * arg1 == The IPA of token buffer
 * ret0 == Status / error
 * ret1 == Size of completed token in bytes
 */
#define SMC_RSI_ATTESTATION_TOKEN_CONTINUE	SMC_RSI_FID(0x195)

/*
 * arg1  == Index, which measurements slot to extend
 * arg2  == Size of realm measurement in bytes, max 64 bytes
 * arg3  == Measurement value, bytes:  0 -  7
 * arg4  == Measurement value, bytes:  7 - 15
 * arg5  == Measurement value, bytes: 16 - 23
 * arg6  == Measurement value, bytes: 24 - 31
 * arg7  == Measurement value, bytes: 32 - 39
 * arg8  == Measurement value, bytes: 40 - 47
 * arg9  == Measurement value, bytes: 48 - 55
 * arg10 == Measurement value, bytes: 56 - 63
 * ret0  == Status / error
 */
#define SMC_RSI_MEASUREMENT_EXTEND		SMC_RSI_FID(0x193)

/*
 * arg1 == Index, which measurements slot to read
 * ret0 == Status / error
 * ret1 == Measurement value, bytes:  0 -  7
 * ret2 == Measurement value, bytes:  7 - 15
 * ret3 == Measurement value, bytes: 16 - 23
 * ret4 == Measurement value, bytes: 24 - 31
 * ret5 == Measurement value, bytes: 32 - 39
 * ret6 == Measurement value, bytes: 40 - 47
 * ret7 == Measurement value, bytes: 48 - 55
 * ret8 == Measurement value, bytes: 56 - 63
 */
#define SMC_RSI_MEASUREMENT_READ		SMC_RSI_FID(0x192)

#ifndef __ASSEMBLY__

struct realm_config {
	unsigned long ipa_bits; /* Width of IPA in bits */
};

#endif /* __ASSEMBLY__ */

/*
 * arg1 == struct realm_config addr
 * ret0 == Status / error
 */
#define SMC_RSI_REALM_CONFIG			SMC_RSI_FID(0x196)

/*
 * arg1 == IPA address of target region
 * arg2 == size of target region in bytes
 * arg3 == RIPAS value
 * ret0 == Status / error
 * ret1 == Top of modified IPA range
 */
#define SMC_RSI_IPA_STATE_SET			SMC_RSI_FID(0x197)

/*
 * arg1 == IPA of target page
 * ret0 == Status / error
 * ret1 == RIPAS value
 */
#define SMC_RSI_IPA_STATE_GET			SMC_RSI_FID(0x198)

/*
 * arg1 == IPA of host call structure
 * ret0 == Status / error
 */
#define SMC_RSI_HOST_CALL			SMC_RSI_FID(0x199)

#endif /* __SMC_RSI_H_ */
