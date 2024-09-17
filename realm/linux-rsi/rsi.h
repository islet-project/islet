#include <linux/ioctl.h>
#include <linux/types.h>


#ifndef RSI_ABI_VERSION_GET_MAJOR
#define RSI_ABI_VERSION_GET_MAJOR(_version) (((_version) & 0x7FFF0000) >> 16)
#endif
#ifndef RSI_ABI_VERSION_GET_MINOR
#define RSI_ABI_VERSION_GET_MINOR(_version) ((_version) & 0xFFFF)
#endif
#define RSI_ABI_VERSION_SET(_major, _minor) (((_major) << 16) | (_minor))


#define MAX_MEASUR_LEN  0x40
#define CHALLENGE_LEN   0x40
#define GRANULE_LEN     0x1000

/*
 * Below is the definition of an ioctl API.
 *
 * ABI_VERSION: one output parameter: the lower version.
 *
 * MEASUREMENT_READ: index is an input parameter (0: RIM, 1-4: REMS), data_len
 * and data are output parameters where the measurement is returned. Values
 * smaller than 64 bytes (e.g. SHA256 -> 32 bytes) are padded with zeroes.
 *
 * MEASUREMENT_EXTEND: all paramaters are input, index (1-4: REMS), data and
 * data_len are data to extend with (1-64 bytes).
 *
 * ATTESTATION_TOKEN: challange is an input parameter (exactly 64 bytes). Token
 * is an allocated memory of token_len where the token will be returned. If the
 * size is too small the kernel will return ERANGE and put the required amount
 * in token_len. Calling it with token_len == 0 and token == NULL to get the
 * required amount is allowed. Token is written in parts. If some error occurs
 * during the call the content of token is undefined.
 */

struct rsi_measurement
{
	uint32_t index;
	uint32_t data_len;
	uint8_t data[MAX_MEASUR_LEN];
};

struct rsi_attestation
{
	uint8_t challenge[CHALLENGE_LEN];
	uint64_t token_len;
	uint8_t *token;
};

#define SHA256_HKDF_OUTPUT_SIZE 32
struct rsi_sealing_key
{
	uint64_t flags;
	uint64_t svn;
	uint8_t realm_sealing_key[SHA256_HKDF_OUTPUT_SIZE];
};

struct rsi_realm_metadata
{
	uint8_t metadata[GRANULE_LEN];
};

// TODO: These should be hex
#define RSIIO_ABI_VERSION          _IOR('x', 190, uint64_t /*version*/)
#define RSIIO_MEASUREMENT_READ     _IOWR('x', 192, struct rsi_measurement)
#define RSIIO_MEASUREMENT_EXTEND   _IOW('x', 193, struct rsi_measurement)
#define RSIIO_ATTESTATION_TOKEN    _IOWR('x', 194, struct rsi_attestation)
#define RSIIO_SEALING_KEY          _IOWR('x', 200, struct rsi_sealing_key)
#define RSIIO_REALM_METADATA       _IOR('x', 201, struct rsi_realm_metadata)
