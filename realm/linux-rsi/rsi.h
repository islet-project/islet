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

struct rsi_cloak
{
	uint64_t id;
	uint64_t result;
	uint8_t challenge[CHALLENGE_LEN];
	uint32_t token_len;
	uint8_t token[MAX_TOKEN_LEN];
};

#define RSIIO_ABI_VERSION          _IOR('x', 190, uint32_t /*version*/)
#define RSIIO_MEASUREMENT_READ     _IOWR('x', 192, struct rsi_measurement)
#define RSIIO_MEASUREMENT_EXTEND   _IOW('x', 193, struct rsi_measurement)
#define RSIIO_ATTESTATION_TOKEN    _IOWR('x', 194, struct rsi_attestation)

// for Cloak
#define RSIIO_CHANNEL_CREATE    _IOWR('x', 195, struct rsi_cloak)
#define RSIIO_CHANNEL_CONNECT    _IOWR('x', 196, struct rsi_cloak)
#define RSIIO_CHANNEL_GEN_REPORT    _IOWR('x', 197, struct rsi_cloak)
#define RSIIO_CHANNEL_RESULT    _IOWR('x', 198, struct rsi_cloak)

/*
 * Those are pages that have to be defined in the kernel itself.
 * They are used as output pages for RSI calls.
 * Needs small patch to the kernel.
 *
 * This will not be required when the module is builtin in the kernel.
 */
extern struct realm_config __attribute((aligned(MAX_TOKEN_LEN))) config;
extern char __attribute__((aligned(MAX_TOKEN_LEN))) rsi_page_buf[MAX_TOKEN_LEN];
extern char __attribute__((aligned(MAX_TOKEN_LEN))) rsi_page_connector[MAX_TOKEN_LEN];
extern char __attribute__((aligned(MAX_TOKEN_LEN))) rsi_page_creator[MAX_TOKEN_LEN];

// for Cloak (test), rsi_page_buf for att_report, rsi_page_buf_* for others
