#include <linux/module.h>    // included for all kernel modules
#include <linux/kernel.h>    // included for KERN_INFO
#include <linux/init.h>      // included for __init and __exit macros
#include <linux/mutex.h>
#include <linux/slab.h>

#include <linux/fs.h>
#include <linux/cdev.h>
#include <linux/device.h>

#include <asm/rsi.h>
#include <asm/uaccess.h>
#include <linux/cc_platform.h>

#include "rsi.h"


MODULE_LICENSE("GPL");
MODULE_AUTHOR("Havner");
MODULE_DESCRIPTION("Linux RSI playground");


#define RSI_TAG   "rsi: "
#define RSI_INFO  KERN_INFO  RSI_TAG
#define RSI_ALERT KERN_ALERT RSI_TAG

#define DEVICE_NAME       "rsi"       /* Name of device in /proc/devices */

static int device_major;              /* Major number assigned to our device driver */
static int device_open_count = 0;     /* Used to prevent multiple open */
static struct class *cls;
#define LINUX_RSI_VERSION \
	RSI_ABI_VERSION_SET(1, 0)         /* Version implemented by this driver */


/* RSI attestation call consists of several arm_smc calls,
 * don't let several users interrupt eachother.
 */
static DEFINE_MUTEX(attestation_call);


static int rsi_ret_to_errno(unsigned long rsi_ret)
{
	switch (rsi_ret) {
	case RSI_SUCCESS:
		return 0;
	case RSI_ERROR_INPUT:
		return EFAULT;
	case RSI_ERROR_STATE:
		return EBADF;
	case RSI_INCOMPLETE:
		return 0;
	default:
		printk(RSI_ALERT "UNKNOWN RSI return code: %lu\n", rsi_ret);
		return ENXIO;
	}
}

static char *rsi_ret_to_str(unsigned long rsi_ret)
{
	switch (rsi_ret) {
	case RSI_SUCCESS:
		return "RSI_SUCCESS";
	case RSI_ERROR_INPUT:
		return "RSI_ERROR_INPUT";
	case RSI_ERROR_STATE:
		return "RSI_ERROR_STATE";
	case RSI_INCOMPLETE:
		return "RSI_INCOMPLETE";
	default:
		return "UNKNOWN RSI return code";
	};
}

static void rsi_playground(void)
{
	unsigned long ret = 0;
	bool realm = false;

	// creative use of an RSI API, rsi_present is static, this is a workaround
	realm = cc_platform_has(CC_ATTR_MEM_ENCRYPT);
	printk(RSI_INFO "Is realm: %s\n", realm ? "true" : "false");

	// version, TODO: loading the driver should probably fail if it's unsupported
	unsigned long lower, higher;
	ret = rsi_get_version(LINUX_RSI_VERSION, &lower, &higher);
	printk(RSI_INFO "RSI version, ret: %s, lower: %lu.%lu, higher: %lu.%lu\n",
	       rsi_ret_to_str(ret),
	       RSI_ABI_VERSION_GET_MAJOR(lower), RSI_ABI_VERSION_GET_MINOR(lower),
	       RSI_ABI_VERSION_GET_MAJOR(higher), RSI_ABI_VERSION_GET_MINOR(higher));

	// get config, just for info/test
	ret = rsi_get_realm_config(&config);
	printk(RSI_INFO "RSI config, ret: %s, ipa_width_in_bits: %lu\n",
	       rsi_ret_to_str(ret), config.ipa_bits);
}

#if 0
#define BYTE_STRING_LEN 4
static void print_data(uint8_t *data, size_t len)
{
	size_t i;
	char ch[BYTE_STRING_LEN], line[32] = {0};

	for (i = 0; i < len; ++i) {
		if (i > 0 && i % 8 == 0) {
			printk(RSI_INFO "%s\n", line);
			line[0] = '\0';
		}
		snprintf(ch, BYTE_STRING_LEN, "%.2X ", data[i]);
		strncat(line, ch, BYTE_STRING_LEN);
	}

	if (line[0] != '\0')
		printk(RSI_INFO "%s\n", line);
}
#endif

/*
 * Chardev
 */

static int device_open(struct inode *i, struct file *f)
{
	printk(RSI_INFO "device %s open\n", DEVICE_NAME);

	if (device_open_count > 0)
		return -EBUSY;

	++device_open_count;
	if (!try_module_get(THIS_MODULE))
		return -ENOENT;

	return 0;
}

static int device_release(struct inode *i, struct file *f)
{
	printk(RSI_INFO "device %s released\n", DEVICE_NAME);

	module_put(THIS_MODULE);
	--device_open_count;

	return 0;
}

static int do_version(uint32_t *version)
{
	struct arm_smccc_1_2_regs input = {0}, output = {0};

	input.a0 = SMC_RSI_ABI_VERSION;
	input.a1 = RSI_ABI_VERSION;
	arm_smccc_1_2_smc(&input, &output);

	printk(RSI_INFO "RSI version, ret: %s\n",
	       rsi_ret_to_str(output.a0));

	if (output.a0 != RSI_SUCCESS)
		return -rsi_ret_to_errno(output.a0);

	*version = output.a1;

	return 0;
}

static int do_measurement_read(struct rsi_measurement *measur)
{
	struct arm_smccc_1_2_regs input = {0}, output = {0};

	input.a0 = SMC_RSI_MEASUREMENT_READ;
	input.a1 = measur->index;
	arm_smccc_1_2_smc(&input, &output);

	printk(RSI_INFO "RSI measurement read, ret: %s\n",
	       rsi_ret_to_str(output.a0));

	if (output.a0 != RSI_SUCCESS)
		return -rsi_ret_to_errno(output.a0);

	measur->data_len = sizeof(output.a1) * 8; // 512 bits always returned, padded with 0
	memcpy(measur->data, (uint8_t*)&output.a1, measur->data_len);

	return 0;
}

static int do_measurement_extend(struct rsi_measurement *measur)
{
	struct arm_smccc_1_2_regs input = {0}, output = {0};

	if (measur->data_len == 0 || measur->data_len > 64) {
		printk(RSI_ALERT "measurement_extend: must be in 1-64 bytes range\n");
		return -EINVAL;
	}

	input.a0 = SMC_RSI_MEASUREMENT_EXTEND;
	input.a1 = measur->index;
	input.a2 = measur->data_len;
	memcpy((uint8_t*)&input.a3, measur->data, measur->data_len);

	arm_smccc_1_2_smc(&input, &output);

	printk(RSI_INFO "RSI measurement extend, ret: %s\n",
	       rsi_ret_to_str(output.a0));

	return -rsi_ret_to_errno(output.a0);
}

static int do_attestation_init(struct rsi_attestation *attest)
{
	struct arm_smccc_1_2_regs input = {0}, output = {0};

	input.a0 = SMC_RSI_ATTESTATION_TOKEN_INIT;
	memcpy((uint8_t*)&input.a1, attest->challenge, sizeof(attest->challenge));

	arm_smccc_1_2_smc(&input, &output);

	printk(RSI_INFO "RSI attestation init, ret: %s, max_token_len: %lu\n",
	       rsi_ret_to_str(output.a0), output.a1);

	// not enough space in the buffer
	if (output.a1 > attest->token_len) {
		printk(RSI_ALERT "More space is needed for the token, got: %llu, need: %lu\n",
		       attest->token_len, output.a1);
		// return how much data is actually needed
		attest->token_len = output.a1;
		return -ERANGE;
	}

	return -rsi_ret_to_errno(output.a0);
}

static int do_attestation_continue(phys_addr_t granule, unsigned long *read)
{
	struct arm_smccc_1_2_regs input = {0}, output = {0};
	unsigned long offset = 0;

	do {
		input.a0 = SMC_RSI_ATTESTATION_TOKEN_CONTINUE;
		input.a1 = granule;
		input.a2 = offset;
		input.a3 = GRANULE_LEN - offset;

		arm_smccc_1_2_smc(&input, &output);

		printk(RSI_INFO "RSI attestation continue, ret: %s, read: %lu\n",
		       rsi_ret_to_str(output.a0), output.a1);

		if (output.a0 != RSI_SUCCESS && output.a0 != RSI_INCOMPLETE) {
			return -rsi_ret_to_errno(output.a0);
		}

		offset += output.a1;
	} while (output.a0 == RSI_INCOMPLETE && offset < GRANULE_LEN);

	// this iteration is done, doesn't mean we're done completely
	*read = offset;

	// we're done, read the buffer
	if (output.a0 == RSI_SUCCESS)
		return 0;

	// run out of buffer, read it and carry on from offset 0
	if (output.a0 == RSI_INCOMPLETE)
		return 1;

	return -rsi_ret_to_errno(output.a0);
}

static int do_attestation(struct rsi_attestation *attest)
{
	int ret, err;
	phys_addr_t granule = virt_to_phys(rsi_page_buf);
	unsigned long total = 0;

	mutex_lock(&attestation_call);

	ret = do_attestation_init(attest);
	if (ret != 0)
		goto unlock;

	if (attest->token == NULL)
		return -EINVAL;

	// fill as much into granule as possible,
	// either till the buffer is full or we have the whole token
	do {
		unsigned long read = 0;
		ret = do_attestation_continue(granule, &read);
		err = copy_to_user(attest->token + total, rsi_page_buf, read);
		if (err != 0) {
			printk(RSI_ALERT "ioctl: copy_to_user failed: %d\n", ret);
			ret = err;
			goto unlock;
		}
		total += read;
	} while (ret == 1);

unlock:
	mutex_unlock(&attestation_call);

	if (ret == 0) {
		attest->token_len = total;
	}

	return ret;
}

static long device_ioctl(struct file *f, unsigned int cmd, unsigned long arg)
{
	int ret = 0, retry = 0;

	uint32_t version = 0;
	struct rsi_measurement *measur = NULL;
	struct rsi_attestation *attest = NULL;

	switch (cmd) {
	case RSIIO_ABI_VERSION:
		printk(RSI_INFO "ioctl: abi_version\n");

		ret = do_version(&version);
		if (ret != 0) {
			printk(RSI_ALERT "ioctl: version failed: %d\n", ret);
		}
		ret = copy_to_user((uint32_t*)arg, &version, sizeof(uint32_t));
		if (ret != 0) {
			printk(RSI_ALERT "ioctl: copy_to_user failed: %d\n", ret);
			return ret;
		}

		break;
	case RSIIO_MEASUREMENT_READ:
		measur = kmalloc(sizeof(struct rsi_measurement), GFP_KERNEL);
		if (measur == NULL) {
			printk("ioctl: failed to allocate");
			return -ENOMEM;
		}

		ret = copy_from_user(measur, (struct rsi_measurement*)arg, sizeof(struct rsi_measurement));
		if (ret != 0) {
			printk(RSI_ALERT "ioctl: copy_from_user failed: %d\n", ret);
			goto end;
		}

		printk(RSI_INFO "ioctl: measurement_read: %u\n", measur->index);

		ret = do_measurement_read(measur);
		if (ret != 0) {
			printk(RSI_ALERT "ioctl: measurement_read failed: %d\n", ret);
			goto end;
		}

		ret = copy_to_user((struct rsi_measurement*)arg, measur, sizeof(struct rsi_measurement));
		if (ret != 0) {
			printk(RSI_ALERT "ioctl: copy_to_user failed: %d\n", ret);
			goto end;
		}

		break;
	case RSIIO_MEASUREMENT_EXTEND:
		measur = kmalloc(sizeof(struct rsi_measurement), GFP_KERNEL);
		if (measur == NULL) {
			printk("ioctl: failed to allocate");
			return -ENOMEM;
		}

		ret = copy_from_user(measur, (struct rsi_measurement*)arg, sizeof(struct rsi_measurement));
		if (ret != 0) {
			printk(RSI_ALERT "ioctl: copy_from_user failed: %d\n", ret);
			goto end;
		}

		printk(RSI_INFO "ioctl: measurement_extend: %u, %u\n", measur->index, measur->data_len);

		ret = do_measurement_extend(measur);
		if (ret != 0) {
			printk(RSI_ALERT "ioctl: measurement_extend failed: %d\n", ret);
			goto end;
		}

		break;
	case RSIIO_ATTESTATION_TOKEN:
		attest = kmalloc(sizeof(struct rsi_attestation), GFP_KERNEL);
		if (attest == NULL) {
			printk("ioctl: failed to allocate");
			return -ENOMEM;
		}

		ret = copy_from_user(attest, (struct rsi_attestation*)arg, sizeof(struct rsi_attestation));
		if (ret != 0) {
			printk(RSI_ALERT "ioctl: copy_from_user failed: %d\n", ret);
			goto end;
		}

		printk(RSI_INFO "ioctl: attestation_token");

		ret = do_attestation(attest);
		if (ret == -ERANGE) {
			retry = 1;
			ret = 0;
		}
		if (ret != 0) {
			printk(RSI_ALERT "ioctl: attestation failed: %d\n", ret);
			goto end;
		}

		ret = copy_to_user((struct rsi_attestation*)arg, attest, sizeof(struct rsi_attestation));
		if (ret != 0) {
			printk(RSI_ALERT "ioctl: copy_to_user failed: %d\n", ret);
			goto end;
		}

		break;
	default:
		printk(RSI_ALERT "ioctl: unknown ioctl cmd\n");
		return -EINVAL;
	}

	ret = 0;

end:
	kfree(attest);
	kfree(measur);

	// token not taken, inform more space is needed
	if (retry)
		return -ERANGE;

	return ret;
}

static struct file_operations chardev_fops = {
	.open = device_open,
	.release = device_release,
	.unlocked_ioctl = device_ioctl,
};

/*
 * Module
 */

static int __init rsi_init(void)
{
	printk(RSI_INFO "Initializing\n");

	device_major = register_chrdev(0, DEVICE_NAME, &chardev_fops);
	if (device_major < 0) {
		printk(RSI_ALERT "register_chrdev failed with %d\n", device_major);
		return device_major;
	}

	printk(RSI_INFO "Chardev registered with major %d\n", device_major);

	cls = class_create(DEVICE_NAME);
	device_create(cls, NULL, MKDEV(device_major, 0), NULL, DEVICE_NAME);

	printk(RSI_INFO "Device created on /dev/%s\n", DEVICE_NAME);

	rsi_playground();

	return 0;
}

static void __exit rsi_cleanup(void)
{
	printk(RSI_INFO "Cleaning up module\n");

	device_destroy(cls, MKDEV(device_major, 0));
	class_destroy(cls);

	unregister_chrdev(device_major, DEVICE_NAME);
}


module_init(rsi_init);
module_exit(rsi_cleanup);
