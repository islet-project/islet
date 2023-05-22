/* SPDX-License-Identifier: GPL-2.0 */
/*
 * Copyright (C) 2022 Arm Limited.
 * All rights reserved.
 */

#ifndef __TOKEN_VERIFIER_H__
#define __TOKEN_VERIFIER_H__

#include <qcbor/qcbor_decode.h>

#define TOKEN_VERIFICATION_ERR_SUCCESS                 0
#define TOKEN_VERIFICATION_ERR_INIT_ERROR              1
#define TOKEN_VERIFICATION_ERR_MISSING_MANDATORY_CLAIM 2
#define TOKEN_VERIFICATION_ERR_INVALID_COSE_TAG        3
#define TOKEN_VERIFICATION_ERR_INVALID_CLAIM_LEN       4
#define TOKEN_VERIFICATION_ERR_INTERNAL_ERROR          5
#define TOKEN_VERIFICATION_ERR_QCBOR(qcbor_err)        (1000 + qcbor_err)

/* Number of realm extensible measurements (REM) */
#define REM_COUNT 4

#define MAX_SW_COMPONENT_COUNT 16

#define CLAIM_COUNT_REALM_TOKEN 6
#define CLAIM_COUNT_COSE_SIGN1_WRAPPER 3
#define CLAIM_COUNT_PLATFORM_TOKEN 8
#define CLAIM_COUNT_REALM_EXTENSIBLE_MEASUREMENTS REM_COUNT
#define CLAIM_COUNT_SW_COMPONENT 4

/* This tells how the data should be interpreted in the claim_t struct, and not
 * necessarily is the same as the item's major type in the token.
 */
enum claim_data_type {
	CLAIM_INT64,
	CLAIM_BOOL,
	CLAIM_BSTR,
	CLAIM_TEXT,
};

struct claim_t {
	/* 'static' */
	bool mandatory;
	enum claim_data_type type;
	int64_t key;
	const char *title;

	/* filled during verification */
	bool present;
	union {
		int64_t int_data;
		bool bool_data;
		/* Used for text and bytestream as well */
		/* TODO: Add expected length check as well? */
		struct q_useful_buf_c buffer_data;
	};
};

struct sw_component_t {
	bool present;
	struct claim_t claims[CLAIM_COUNT_SW_COMPONENT];
};

struct attestation_claims {
	struct claim_t realm_cose_sign1_wrapper[CLAIM_COUNT_COSE_SIGN1_WRAPPER];
	struct claim_t realm_token_claims[CLAIM_COUNT_REALM_TOKEN];
	struct claim_t realm_measurement_claims[CLAIM_COUNT_REALM_EXTENSIBLE_MEASUREMENTS];
	struct claim_t plat_cose_sign1_wrapper[CLAIM_COUNT_COSE_SIGN1_WRAPPER];
	struct claim_t plat_token_claims[CLAIM_COUNT_PLATFORM_TOKEN];
	struct sw_component_t sw_component_claims[MAX_SW_COMPONENT_COUNT];
};

/* Returns TOKEN_VERIFICATION_ERR* */
int verify_token(const char *token, size_t size,
	struct attestation_claims *attest_claims);

#endif /* __TOKEN_VERIFIER_H__ */
