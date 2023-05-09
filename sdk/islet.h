// Copyright (c) 2023 Samsung Electronics Co., Ltd. All Rights Reserved.

#include <cstdarg>
#include <cstdint>
#include <cstdlib>
#include <ostream>
#include <new>

extern "C" {

int islet_attest(const unsigned char *user_data,
                 int user_data_len,
                 unsigned char *report_out,
                 int *report_out_len);

int islet_verify(const unsigned char *report,
                 int report_len,
                 unsigned char *claims_out,
                 int *claims_out_len);

int islet_parse(const char *title,
                const unsigned char *claims,
                int claims_len,
                unsigned char *value_out,
                int *value_out_len);

} // extern "C"
