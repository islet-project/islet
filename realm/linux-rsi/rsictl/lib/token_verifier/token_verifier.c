/* SPDX-License-Identifier: GPL-2.0 */
/*
 * Copyright (C) 2022 Arm Limited.
 * All rights reserved.
 */

#include <stdio.h>
#include <inttypes.h>
#include <qcbor/qcbor_decode.h>
#include <qcbor/qcbor_spiffy_decode.h>
#include "attest_defines.h"
#include "token_verifier.h"
#include "token_dumper.h"

#define SHA256_SIZE 32
#define SHA512_SIZE 64

#define ARRAY_SIZE(a) (sizeof(a)/sizeof((a)[0]))

#define RETURN_ON_DECODE_ERROR(p_context) \
	do { \
		QCBORError ret; \
		ret = QCBORDecode_GetError(p_context); \
		if (ret != QCBOR_SUCCESS) { \
			printf("QCBOR decode failed with error at %s:%d." \
				" err = %d\r\n", \
				__FILE__, (int)__LINE__, (int)ret); \
			return TOKEN_VERIFICATION_ERR_QCBOR(ret); \
		} \
	} while (0)

static void init_claim(struct claim_t *claim,
		       bool mandatory, enum claim_data_type type,
		       int64_t key, const char *title, bool present)
{
	claim->mandatory = mandatory;
	claim->type = type;
	claim->key = key;
	claim->title = title;
	claim->present = present;
}

static int init_cose_wrapper_claim(struct claim_t *cose_sign1_wrapper)
{
	struct claim_t *c;

	/* The cose wrapper looks like the following:
	 *  - Protected header (bytestring).
	 *  - Unprotected header: might contain 0 items. This is a map. Due to
	 *    the way this thing is implemented, it is not in the below list,
	 *    but is handled in the verify_token_cose_sign1_wrapping
	 *    function.
	 *  - Payload: Platform token (bytestring). The content is passed for
	 *    verify_platform_token.
	 *  - Signature.
	 */
	c = cose_sign1_wrapper;
	/* This structure is in an array, so the key is not used */
	init_claim(c++, true, CLAIM_BSTR, 0, "Protected header",  false);
	init_claim(c++, true, CLAIM_BSTR, 0, "Platform token payload", false);
	init_claim(c++, true, CLAIM_BSTR, 0, "Signature",  false);
	if (c > cose_sign1_wrapper + CLAIM_COUNT_COSE_SIGN1_WRAPPER) {
		return TOKEN_VERIFICATION_ERR_INIT_ERROR;
	}
	return 0;
}

static int init_claims(struct attestation_claims *attest_claims)
{
	int i;
	int ret;
	struct claim_t *c;
	/* TODO: All the buffer overwrite checks are happening too late.
	 * Either remove, or find a better way.
	 */
	c = attest_claims->realm_token_claims;
	init_claim(c++, true, CLAIM_BSTR, CCA_REALM_CHALLENGE,             "Realm challenge",                false);
	init_claim(c++, true, CLAIM_BSTR, CCA_REALM_PERSONALIZATION_VALUE, "Realm personalization value",    false);
	init_claim(c++, true, CLAIM_TEXT, CCA_REALM_HASH_ALGO_ID,          "Realm hash algo id",             false);
	init_claim(c++, true, CLAIM_TEXT, CCA_REALM_PUB_KEY_HASH_ALGO_ID,  "Realm public key hash algo id",  false);
	init_claim(c++, true, CLAIM_BSTR, CCA_REALM_PUB_KEY,               "Realm signing public key",       false);
	init_claim(c++, true, CLAIM_BSTR, CCA_REALM_INITIAL_MEASUREMENT,   "Realm initial measurement",      false);
	/* Realm extensible measurements are not present here as they are
	 * encoded as a CBOR array, and it is handled specially in
	 * verify_realm_token().
	 */
	if (c > attest_claims->realm_token_claims + CLAIM_COUNT_REALM_TOKEN) {
		return TOKEN_VERIFICATION_ERR_INIT_ERROR;
	}

	ret = init_cose_wrapper_claim(attest_claims->realm_cose_sign1_wrapper);
	if (ret != 0) {
		return ret;
	}
	ret = init_cose_wrapper_claim(attest_claims->plat_cose_sign1_wrapper);
	if (ret != 0) {
		return ret;
	}

	c = attest_claims->plat_token_claims;
	init_claim(c++, true,  CLAIM_BSTR,  CCA_PLAT_CHALLENGE,            "Challenge",            false);
	init_claim(c++, false, CLAIM_TEXT,  CCA_PLAT_VERIFICATION_SERVICE, "Verification service", false);
	init_claim(c++, true,  CLAIM_TEXT,  CCA_PLAT_PROFILE,              "Profile",              false);
	init_claim(c++, true,  CLAIM_BSTR,  CCA_PLAT_INSTANCE_ID,          "Instance ID",          false);
	init_claim(c++, true,  CLAIM_BSTR,  CCA_PLAT_IMPLEMENTATION_ID,    "Implementation ID",    false);
	init_claim(c++, true,  CLAIM_INT64, CCA_PLAT_SECURITY_LIFECYCLE,   "Lifecycle",            false);
	init_claim(c++, true,  CLAIM_BSTR,  CCA_PLAT_CONFIGURATION,        "Configuration",        false);
	init_claim(c++, true,  CLAIM_TEXT,  CCA_PLAT_HASH_ALGO_ID,         "Platform hash algo",   false);
	if (c > attest_claims->plat_token_claims +
		CLAIM_COUNT_PLATFORM_TOKEN) {
		return TOKEN_VERIFICATION_ERR_INIT_ERROR;
	}

	for (i = 0; i < CLAIM_COUNT_REALM_EXTENSIBLE_MEASUREMENTS; ++i) {
		c = attest_claims->realm_measurement_claims + i;
		init_claim(c, true, CLAIM_BSTR, i,
			"Realm extensible measurements", false);
	}

	for (i = 0; i < MAX_SW_COMPONENT_COUNT; ++i) {
		struct sw_component_t *component =
			attest_claims->sw_component_claims + i;

		component->present = false;
		c = component->claims;
		init_claim(c++, false, CLAIM_TEXT, CCA_SW_COMP_HASH_ALGORITHM,    "Hash algo.",  false);
		init_claim(c++, true,  CLAIM_BSTR, CCA_SW_COMP_MEASUREMENT_VALUE, "Meas. val.", false);
		init_claim(c++, false, CLAIM_TEXT, CCA_SW_COMP_VERSION,           "Version",    false);
		init_claim(c++, true,  CLAIM_BSTR, CCA_SW_COMP_SIGNER_ID,         "Signer ID",  false);
		if (c > component->claims + CLAIM_COUNT_SW_COMPONENT) {
			return TOKEN_VERIFICATION_ERR_INIT_ERROR;
		}
	}
	return TOKEN_VERIFICATION_ERR_SUCCESS;
}

static int handle_claim_decode_error(const struct claim_t *claim,
				     QCBORError err)
{
	if (err == QCBOR_ERR_LABEL_NOT_FOUND) {
		if (claim->mandatory) {
			printf("Mandatory claim with key %" PRId64 " (%s) is "
				"missing from token.\r\n", claim->key,
				claim->title);
			return TOKEN_VERIFICATION_ERR_MISSING_MANDATORY_CLAIM;
		}
	} else {
		printf("Decode failed with error at %s:%d. err = %d key = %"
			PRId64 " (%s).\r\n",  __FILE__, (int)__LINE__, err,
			claim->key, claim->title);
		return TOKEN_VERIFICATION_ERR_QCBOR(err);
	}
	return TOKEN_VERIFICATION_ERR_SUCCESS;
}

/* Consume claims from a map.
 *
 * This function iterates on the array 'claims', and looks up items with the
 * specified keys. If a claim flagged as mandatory is not found, an error is
 * returned. The function doesn't checks for extra items. So if the map contains
 * items with keys that are not in the claims array, no error is reported.
 *
 * The map needs to be 'entered' before calling this function, and be 'exited'
 * after it returns.
 */
static int get_claims_from_map(QCBORDecodeContext *p_context,
			       struct claim_t *claims,
			       size_t num_of_claims)
{
	QCBORError err;
	int token_verification_error;
	unsigned i;

	for (i = 0; i < num_of_claims; ++i) {
		struct claim_t *claim = claims + i;

		switch (claim->type) {
		case CLAIM_INT64:
			QCBORDecode_GetInt64InMapN(p_context, claim->key,
				&(claim->int_data));
			break;
		case CLAIM_BOOL:
			QCBORDecode_GetBoolInMapN(p_context, claim->key,
				&(claim->bool_data));
			break;
		case CLAIM_BSTR:
			QCBORDecode_GetByteStringInMapN(p_context, claim->key,
				&(claim->buffer_data));
			break;
		case CLAIM_TEXT:
			QCBORDecode_GetTextStringInMapN(p_context, claim->key,
				&(claim->buffer_data));
			break;
		default:
			printf("Internal error at  %s:%d.\r\n",
				__FILE__, (int)__LINE__);
			return TOKEN_VERIFICATION_ERR_INTERNAL_ERROR;
		}
		err = QCBORDecode_GetAndResetError(p_context);
		if (err == QCBOR_SUCCESS) {
			claim->present = true;
		} else {
			token_verification_error =
				handle_claim_decode_error(claim, err);
			if (token_verification_error !=
				TOKEN_VERIFICATION_ERR_SUCCESS) {
				return token_verification_error;
			}
		}
	}
	return TOKEN_VERIFICATION_ERR_SUCCESS;
}

/* Consume a single claim from an array and from the top level.
 *
 * The claim's 'key' and 'mandatory' attribute is not used in this function.
 * The claim is considered mandatory.
 */
static int get_claim(QCBORDecodeContext *p_context, struct claim_t *claim)
{
	QCBORError err;

	switch (claim->type) {
	case CLAIM_INT64:
		QCBORDecode_GetInt64(p_context, &(claim->int_data));
		break;
	case CLAIM_BOOL:
		QCBORDecode_GetBool(p_context, &(claim->bool_data));
		break;
	case CLAIM_BSTR:
		QCBORDecode_GetByteString(p_context, &(claim->buffer_data));
		break;
	case CLAIM_TEXT:
		QCBORDecode_GetTextString(p_context, &(claim->buffer_data));
		break;
	default:
		printf("Internal error at  %s:%d.\r\n",
			__FILE__, (int)__LINE__);
		break;
	}
	err = QCBORDecode_GetAndResetError(p_context);
	if (err == QCBOR_SUCCESS) {
		claim->present = true;
		return TOKEN_VERIFICATION_ERR_SUCCESS;
	}
	printf("Decode failed with error at %s:%d. err = %d claim: \"%s\".\r\n",
		__FILE__, (int)__LINE__, err, claim->title);
	return TOKEN_VERIFICATION_ERR_QCBOR(err);
}

/* Consume claims from an array and from the top level.
 *
 * This function iterates on the array 'claims', and gets an item for each
 * element. If the array or the cbor runs out of elements before reaching the
 * end of the 'claims' array, then error is returned.
 *
 * The claim's 'key' and 'mandatory' attribute is not used in this function.
 * All the elements considered mandatory.
 */
static int get_claims(QCBORDecodeContext *p_context, struct claim_t *claims,
		      size_t num_of_claims)
{
	QCBORError err;
	unsigned i;

	for (i = 0; i < num_of_claims; ++i) {
		struct claim_t *claim = claims + i;

		err = get_claim(p_context, claim);
		if (err != TOKEN_VERIFICATION_ERR_SUCCESS) {
			return err;
		}
	}
	return TOKEN_VERIFICATION_ERR_SUCCESS;
}

static int verify_platform_token(struct q_useful_buf_c buf,
				 struct attestation_claims *attest_claims)
{
	QCBORDecodeContext context;
	int err;
	int label, index;

	QCBORDecode_Init(&context, buf, QCBOR_DECODE_MODE_NORMAL);
	RETURN_ON_DECODE_ERROR(&context);

	QCBORDecode_EnterMap(&context, NULL);
	RETURN_ON_DECODE_ERROR(&context);

	err = get_claims_from_map(&context,
		attest_claims->plat_token_claims,
		CLAIM_COUNT_PLATFORM_TOKEN);
	if (err != TOKEN_VERIFICATION_ERR_SUCCESS) {
		return err;
	}

	label = CCA_PLAT_SW_COMPONENTS;
	QCBORDecode_EnterArrayFromMapN(&context, label);
	RETURN_ON_DECODE_ERROR(&context);

	index = 0;
	while (1) {
		QCBORDecode_EnterMap(&context, NULL);
		if (QCBORDecode_GetError(&context) == QCBOR_ERR_NO_MORE_ITEMS) {
			/* This is OK. We just reached the end of the array.
			 * Break from the loop.
			 */
			break;
		}

		if (index >= MAX_SW_COMPONENT_COUNT) {
			printf("Not enough slots in sw_component_claims.\r\n");
			printf("Increase MAX_SW_COMPONENT_COUNT in %s.\r\n",
				__FILE__);
			return TOKEN_VERIFICATION_ERR_INTERNAL_ERROR;
		}

		err = get_claims_from_map(&context,
			attest_claims->sw_component_claims[index].claims,
			CLAIM_COUNT_SW_COMPONENT);
		if (err != TOKEN_VERIFICATION_ERR_SUCCESS) {
			return err;
		}
		attest_claims->sw_component_claims[index].present = true;

		QCBORDecode_ExitMap(&context);
		RETURN_ON_DECODE_ERROR(&context);

		++index;
	}
	/* We only get here if the decode error code was a
	 * QCBOR_ERR_NO_MORE_ITEMS which is expected when the end of an array is
	 * reached. In this case the processing must be continued, so clear the
	 * error.
	 */
	QCBORDecode_GetAndResetError(&context);
	RETURN_ON_DECODE_ERROR(&context);

	QCBORDecode_ExitArray(&context);
	RETURN_ON_DECODE_ERROR(&context);

	QCBORDecode_ExitMap(&context);
	RETURN_ON_DECODE_ERROR(&context);

	QCBORDecode_Finish(&context);

	return TOKEN_VERIFICATION_ERR_SUCCESS;
}

static bool verify_length_of_measurement(size_t len)
{
	size_t allowed_lengths[] = {SHA256_SIZE, SHA512_SIZE};
	unsigned i;

	for (i = 0; i < ARRAY_SIZE(allowed_lengths); ++i) {
		if (len == allowed_lengths[i])
			return true;
	}

	return false;
}

static int verify_realm_token(struct q_useful_buf_c buf,
			     struct attestation_claims *attest_claims)
{
	QCBORDecodeContext context;
	int err;
	int i;

	QCBORDecode_Init(&context, buf, QCBOR_DECODE_MODE_NORMAL);
	RETURN_ON_DECODE_ERROR(&context);

	QCBORDecode_EnterMap(&context, NULL);
	RETURN_ON_DECODE_ERROR(&context);

	err = get_claims_from_map(&context, attest_claims->realm_token_claims,
		CLAIM_COUNT_REALM_TOKEN);
	if (err != TOKEN_VERIFICATION_ERR_SUCCESS) {
		return err;
	}

	/* Now get the realm extensible measurements */
	QCBORDecode_EnterArrayFromMapN(&context,
					CCA_REALM_EXTENSIBLE_MEASUREMENTS);
	RETURN_ON_DECODE_ERROR(&context);

	err = get_claims(&context,
		attest_claims->realm_measurement_claims,
		CLAIM_COUNT_REALM_EXTENSIBLE_MEASUREMENTS);
	if (err != TOKEN_VERIFICATION_ERR_SUCCESS) {
		return err;
	}

	for (i = 0; i < CLAIM_COUNT_REALM_EXTENSIBLE_MEASUREMENTS; ++i) {
		struct claim_t *claims =
			attest_claims->realm_measurement_claims;
		struct q_useful_buf_c buf = claims[i].buffer_data;

		if (!verify_length_of_measurement(buf.len)) {
			return TOKEN_VERIFICATION_ERR_INVALID_CLAIM_LEN;
		}
	}

	QCBORDecode_ExitArray(&context);
	RETURN_ON_DECODE_ERROR(&context);

	QCBORDecode_ExitMap(&context);
	QCBORDecode_Finish(&context);

	return TOKEN_VERIFICATION_ERR_SUCCESS;
}

/* Returns a pointer to the wrapped token in: 'token_payload'.
 * Returns the claims in the wrapper in cose_sign1_wrapper.
 */
static int verify_token_cose_sign1_wrapping(
				  struct q_useful_buf_c token,
				  struct q_useful_buf_c *token_payload,
				  struct claim_t *cose_sign1_wrapper)
{
	QCBORDecodeContext context;
	QCBORItem item;
	int err;

	QCBORDecode_Init(&context, token, QCBOR_DECODE_MODE_NORMAL);
	RETURN_ON_DECODE_ERROR(&context);

	/* Check COSE tag. */
	QCBORDecode_PeekNext(&context, &item);
	if (!QCBORDecode_IsTagged(&context, &item,
		TAG_COSE_SIGN1)) {
		return TOKEN_VERIFICATION_ERR_INVALID_COSE_TAG;
	}

	QCBORDecode_EnterArray(&context, NULL);
	RETURN_ON_DECODE_ERROR(&context);

	/* Protected header */
	err = get_claim(&context, cose_sign1_wrapper);
	if (err != TOKEN_VERIFICATION_ERR_SUCCESS) {
		return err;
	}

	/* Unprotected header. The map is always present, but may contain 0
	 * items.
	 */
	QCBORDecode_EnterMap(&context, NULL);
	RETURN_ON_DECODE_ERROR(&context);

		/* Skip the content for now. */

	QCBORDecode_ExitMap(&context);
	RETURN_ON_DECODE_ERROR(&context);

	/* Payload */
	err = get_claim(&context, cose_sign1_wrapper + 1);
	if (err != TOKEN_VERIFICATION_ERR_SUCCESS) {
		return err;
	}

	/* Signature */
	err = get_claim(&context, cose_sign1_wrapper + 2);
	if (err != TOKEN_VERIFICATION_ERR_SUCCESS) {
		return err;
	}

	QCBORDecode_ExitArray(&context);
	RETURN_ON_DECODE_ERROR(&context);

	*token_payload = cose_sign1_wrapper[1].buffer_data;

	return TOKEN_VERIFICATION_ERR_SUCCESS;
}

static int verify_cca_token(struct q_useful_buf_c  token,
			    struct q_useful_buf_c *platform_token,
			    struct q_useful_buf_c *realm_token)
{
	QCBORDecodeContext context;
	QCBORItem item;
	QCBORError err;

	QCBORDecode_Init(&context, token, QCBOR_DECODE_MODE_NORMAL);
	RETURN_ON_DECODE_ERROR(&context);

	/* ================== Check CCA_TOKEN tag =========================== */
	QCBORDecode_PeekNext(&context, &item);
	if (!QCBORDecode_IsTagged(&context, &item, TAG_CCA_TOKEN)) {
		return TOKEN_VERIFICATION_ERR_INVALID_COSE_TAG;
	}

	/* ================== Get the the platform token ==================== */
	QCBORDecode_EnterMap(&context, NULL);
	RETURN_ON_DECODE_ERROR(&context);

	/*
	 * First element is the CCA platfrom token which is a
	 * COSE_Sign1_Tagged object. It has byte stream wrapper.
	 */
	QCBORDecode_GetByteStringInMapN(&context, CCA_PLAT_TOKEN,
					platform_token);
	RETURN_ON_DECODE_ERROR(&context);

	/* ================== Get the the realm token ======================= */
	/*
	 * Second element is the delegated realm token which is a
	 * COSE_Sign1_Tagged object. It has byte stream wrapper.
	 */
	QCBORDecode_GetByteStringInMapN(&context, CCA_REALM_DELEGATED_TOKEN,
					realm_token);
	RETURN_ON_DECODE_ERROR(&context);

	QCBORDecode_ExitMap(&context);
	RETURN_ON_DECODE_ERROR(&context);

	/* Finishing up the decoding of the top-level wrapper */
	err = QCBORDecode_Finish(&context);
	if (err != QCBOR_SUCCESS) {
		printf("QCBOR decode failed with error at %s:%d. err = %d\r\n",
			__FILE__, (int)__LINE__, (int)err);
		return TOKEN_VERIFICATION_ERR_QCBOR(err);
	}

	return TOKEN_VERIFICATION_ERR_SUCCESS;
}

/*
 * This function expect two COSE_Sing1_Tagged object wrapped with a tagged map:
 *
 * cca-token = #6.44234(cca-token-map) ; 44234 = 0xACCA
 *
 * cca-platform-token = COSE_Sign1_Tagged
 * cca-realm-delegated-token = COSE_Sign1_Tagged
 *
 * cca-token-map = {
 *   0 => cca-platform-token
 *   1 => cca-realm-delegated-token
 * }
 *
 * COSE_Sign1_Tagged = #6.18(COSE_Sign1)
 */
int verify_token(const char *token, size_t size,
		 struct attestation_claims *attest_claims)
{
	/* TODO: do signature check */
	/* TODO: Add tag check on tokens */
	struct q_useful_buf_c buf = {token, size};
	int ret;
	struct q_useful_buf_c realm_token;
	struct q_useful_buf_c realm_token_payload;
	struct q_useful_buf_c platform_token;
	struct q_useful_buf_c platform_token_payload;

	ret = init_claims(attest_claims);
	if (ret != TOKEN_VERIFICATION_ERR_SUCCESS) {
		return ret;
	}

	/* Verify top-level token map and extract the two sub-tokens */
	ret = verify_cca_token(buf, &platform_token, &realm_token);
	if (ret != TOKEN_VERIFICATION_ERR_SUCCESS) {
		return ret;
	}

	/* Verify the COSE_Sign1 wrapper of the realm token */
	ret = verify_token_cose_sign1_wrapping(realm_token,
		&realm_token_payload,
		attest_claims->realm_cose_sign1_wrapper);
	if (ret != TOKEN_VERIFICATION_ERR_SUCCESS) {
		return ret;
	}
	/* Verify the payload of the realm token */
	ret = verify_realm_token(realm_token_payload, attest_claims);
	if (ret != TOKEN_VERIFICATION_ERR_SUCCESS) {
		return ret;
	}

	/* Verify the COSE_Sign1 wrapper of the platform token */
	ret = verify_token_cose_sign1_wrapping(platform_token,
		&platform_token_payload,
		attest_claims->plat_cose_sign1_wrapper);
	if (ret != TOKEN_VERIFICATION_ERR_SUCCESS) {
		return ret;
	}
	/* Verify the payload of the platform token */
	ret = verify_platform_token(platform_token_payload, attest_claims);
	if (ret != TOKEN_VERIFICATION_ERR_SUCCESS) {
		return ret;
	}

	return TOKEN_VERIFICATION_ERR_SUCCESS;
}
