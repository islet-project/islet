/* SPDX-License-Identifier: GPL-2.0 */
/*
 * Copyright (C) 2022 Arm Limited.
 * All rights reserved.
 */

#ifndef __TOKEN_DUMPER_H__
#define __TOKEN_DUMPER_H__

#include "token_verifier.h"

void print_raw_token(const char *token, size_t size);
void print_token(const struct attestation_claims *claims);

#endif /* __TOKEN_DUMPER_H__ */
