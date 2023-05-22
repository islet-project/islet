#include <linux/ioctl.h>
#include <linux/types.h>


#ifndef RSI_ABI_VERSION_GET_MAJOR
#define RSI_ABI_VERSION_GET_MAJOR(_version) ((_version) >> 16)
#endif
#ifndef RSI_ABI_VERSION_GET_MINOR
#define RSI_ABI_VERSION_GET_MINOR(_version) ((_version) & 0xFFFF)
#endif

#define MAX_MEASUR_LEN  0x40
#define CHALLENGE_LEN   0x40
#define MAX_TOKEN_LEN   0x1000

struct rsi_measurement
{
	uint32_t index;
	uint32_t data_len;
	uint8_t data[MAX_MEASUR_LEN];
};

struct rsi_attestation
{
	uint8_t challenge[CHALLENGE_LEN];
	uint32_t token_len;
	uint8_t token[MAX_TOKEN_LEN];
};

#define RSIIO_ABI_VERSION          _IOR('x', 190, uint32_t /*version*/)
#define RSIIO_MEASUREMENT_READ     _IOWR('x', 192, struct rsi_measurement)
#define RSIIO_MEASUREMENT_EXTEND   _IOW('x', 193, struct rsi_measurement)
#define RSIIO_ATTESTATION_TOKEN    _IOWR('x', 194, struct rsi_attestation)

/*
 * Those are pages that have to be defined in the kernel itself.
 * They are used as output pages for RSI calls.
 * Needs small patch to the kernel.
 *
 * This will not be required when the module is builtin in the kernel.
 */
extern struct realm_config __attribute((aligned(MAX_TOKEN_LEN))) config;
extern char __attribute__((aligned(MAX_TOKEN_LEN))) rsi_page_buf[MAX_TOKEN_LEN];
