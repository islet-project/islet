/* SPDX-License-Identifier: GPL-2.0 */
/*
 * Copyright (C) 2022 Arm Limited.
 * All rights reserved.
 */

#include <stdio.h>
#include <inttypes.h>
#include "attest_defines.h"
#include "token_dumper.h"

#define COLUMN_WIDTH "20"

void print_raw_token(const char *token, size_t size)
{
	unsigned i;
	char byte;

	printf("\r\nCopy paste token to www.cbor.me\r\n");
	for (i = 0; i < size; ++i) {
		byte = token[i];
		if (byte == 0)
			printf("0x%#02x ", byte);
		else
			printf("0x%02x ", byte);
		if (((i + 1) % 8) == 0)
			printf("\r\n");
	}
	printf("\r\n");
}

static void print_indent(int indent_level)
{
	int i;

	for (i = 0; i < indent_level; ++i) {
		printf("  ");
	}
}

static void print_byte_string(const char *name, int index,
			      struct q_useful_buf_c buf)
{
	unsigned i;

	printf("%-"COLUMN_WIDTH"s (#%d) = [", name, index);
	for (i = 0; i < buf.len; ++i) {
		printf("%02x", ((uint8_t *)buf.ptr)[i]);
	}
	printf("]\r\n");
}

static void print_text(const char *name, int index, struct q_useful_buf_c buf)
{
	unsigned i;

	printf("%-"COLUMN_WIDTH"s (#%d) = \"", name, index);
	for (i = 0; i < buf.len; ++i) {
		printf("%c", ((uint8_t *)buf.ptr)[i]);
	}
	printf("\"\r\n");
}

static void print_claim(const struct claim_t *claim, int indent_level)
{
	print_indent(indent_level);
	if (claim->present) {
		switch (claim->type) {
		case CLAIM_INT64:
			printf("%-"COLUMN_WIDTH"s (#%" PRId64 ") = %" PRId64
				"\r\n", claim->title,
			claim->key, claim->int_data);
			break;
		case CLAIM_BOOL:
			printf("%-"COLUMN_WIDTH"s (#%" PRId64 ") = %s\r\n",
			claim->title, claim->key,
			claim->bool_data?"true":"false");
			break;
		case CLAIM_BSTR:
			print_byte_string(claim->title, claim->key,
				claim->buffer_data);
			break;
		case CLAIM_TEXT:
			print_text(claim->title, claim->key,
				claim->buffer_data);
			break;
		default:
			printf("* Internal error at  %s:%d.\r\n", __FILE__,
				(int)__LINE__);
			break;
		}
	} else {
		printf("* Missing%s claim with key: %" PRId64 " (%s)\r\n",
			claim->mandatory?" mandatory":"",
			claim->key, claim->title);
	}
}

static void print_cose_sign1_wrapper(const char *token_type,
				     const struct claim_t *cose_sign1_wrapper)
{
	printf("\r\n== %s Token cose header:\r\n", token_type);
	print_claim(cose_sign1_wrapper + 0, 0);
	/* Don't print wrapped token bytestring */
	print_claim(cose_sign1_wrapper + 2, 0);
	printf("== End of %s Token cose header\r\n\r\n", token_type);
}

void print_token(const struct attestation_claims *claims)
{
	int i;

	print_cose_sign1_wrapper("Realm", claims->realm_cose_sign1_wrapper);

	printf("\r\n== Realm Token:\r\n");
	/* print the claims except the last one. That is printed in detail
	 * below.
	 */
	for (i = 0; i < CLAIM_COUNT_REALM_TOKEN; ++i) {
		const struct claim_t *claim = claims->realm_token_claims + i;

		print_claim(claim, 0);
	}

	printf("%-"COLUMN_WIDTH"s (#%d)\r\n", "Realm measurements",
		CCA_REALM_EXTENSIBLE_MEASUREMENTS);
	for (i = 0; i < CLAIM_COUNT_REALM_EXTENSIBLE_MEASUREMENTS; ++i) {
		const struct claim_t *claim = claims->realm_measurement_claims + i;

		print_claim(claim, 1);
	}
	printf("== End of Realm Token.\r\n");

	print_cose_sign1_wrapper("Platform", claims->plat_cose_sign1_wrapper);

	printf("\r\n== Platform Token:\r\n");
	for (i = 0; i < CLAIM_COUNT_PLATFORM_TOKEN; ++i) {
		const struct claim_t *claim = claims->plat_token_claims + i;

		print_claim(claim, 0);
	}
	printf("== End of Platform Token\r\n\r\n");

	printf("\r\n== Platform Token SW components:\r\n");

	for (i = 0; i < MAX_SW_COMPONENT_COUNT; ++i) {
		const struct sw_component_t *component =
			claims->sw_component_claims + i;

		if (component->present) {
			printf("  SW component #%d:\r\n", i);
			for (int j = 0; j < CLAIM_COUNT_SW_COMPONENT; ++j) {
				print_claim(component->claims + j, 2);
			}
		}
	}
	printf("== End of Platform Token SW components\r\n\r\n");
}
