/* SPDX-License-Identifier: GPL-2.0 */
/*
 * Copyright (C) 2022 Arm Limited.
 * All rights reserved.
 */

#ifndef __ATTEST_DEFINES_H__
#define __ATTEST_DEFINES_H__

#ifdef __cplusplus
extern "C" {
#endif

#define TAG_COSE_SIGN1                       (18)
#define TAG_CCA_TOKEN                       (399)

#define CCA_PLAT_TOKEN                    (44234)    /* 0xACCA */
#define CCA_REALM_DELEGATED_TOKEN         (44241)

/* CCA Platform Attestation Token */
#define CCA_PLAT_CHALLENGE                   (10)    /* EAT nonce */
#define CCA_PLAT_INSTANCE_ID                (256)    /* EAT ueid */
#define CCA_PLAT_PROFILE                    (265)    /* EAT profile */
#define CCA_PLAT_SECURITY_LIFECYCLE        (2395)
#define CCA_PLAT_IMPLEMENTATION_ID         (2396)
#define CCA_PLAT_SW_COMPONENTS             (2399)
#define CCA_PLAT_VERIFICATION_SERVICE      (2400)
#define CCA_PLAT_CONFIGURATION             (2401)
#define CCA_PLAT_HASH_ALGO_ID              (2402)

/* CCA Realm Delegated Attestation Token */
#define CCA_REALM_CHALLENGE                  (10)    /* EAT nonce */
#define CCA_REALM_PERSONALIZATION_VALUE   (44235)
#define CCA_REALM_HASH_ALGO_ID            (44236)
#define CCA_REALM_PUB_KEY                 (44237)
#define CCA_REALM_INITIAL_MEASUREMENT     (44238)
#define CCA_REALM_EXTENSIBLE_MEASUREMENTS (44239)
#define CCA_REALM_PUB_KEY_HASH_ALGO_ID    (44240)

/* Software components */
#define CCA_SW_COMP_MEASUREMENT_VALUE         (2)
#define CCA_SW_COMP_VERSION                   (4)
#define CCA_SW_COMP_SIGNER_ID                 (5)
#define CCA_SW_COMP_HASH_ALGORITHM            (6)

#ifdef __cplusplus
}
#endif

#endif /* __ATTEST_DEFINES_H__ */
