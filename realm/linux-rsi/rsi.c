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

/* RSI attestation call consists of several arm_smc calls,
 * don't let several users interrupt eachother.
 */
static DEFINE_MUTEX(attestation_call);


static void rsi_playground(void)
{
	unsigned long ret = 0;
	bool realm = false;
	unsigned long ver = 0;

	// creative use of an API
	realm = cc_platform_has(CC_ATTR_MEM_ENCRYPT);
	printk(RSI_INFO "Is realm: %s\n", realm ? "true" : "false");

	// version
	ver = rsi_get_version();
	printk(RSI_INFO "RSI version: %lu.%lu\n",
	       RSI_ABI_VERSION_GET_MAJOR(ver), RSI_ABI_VERSION_GET_MINOR(ver));

	// get config
	ret = rsi_get_realm_config(&config);
	printk(RSI_INFO "Config ret: %lu, Bits: %lX\n", ret, config.ipa_bits);
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
		printk(RSI_ALERT "unknown ret code returned from RSI: %lu\n", rsi_ret);
		return ENXIO;
	}
}

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

static int do_measurement_read(struct rsi_measurement *measur)
{
	struct arm_smccc_1_2_regs input = {0}, output = {0};

	input.a0 = SMC_RSI_MEASUREMENT_READ;
	input.a1 = measur->index;
	arm_smccc_1_2_smc(&input, &output);

	if (output.a0 != RSI_SUCCESS)
		return -rsi_ret_to_errno(output.a0);

	measur->data_len = sizeof(output.a1) * 8;
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
	memcpy((uint8_t*)&output.a3, measur->data, measur->data_len);

	arm_smccc_1_2_smc(&input, &output);

	if (output.a0 != RSI_SUCCESS)
		return -rsi_ret_to_errno(output.a0);

	return 0;
}

static int do_attestation_init(phys_addr_t page, struct rsi_attestation *attest)
{
	struct arm_smccc_1_2_regs input = {0}, output = {0};

	input.a0 = SMC_RSI_ATTESTATION_TOKEN_INIT;
	input.a1 = page;
	memcpy((uint8_t*)&output.a2, attest->challenge, sizeof(attest->challenge));

	arm_smccc_1_2_smc(&input, &output);

	// TODO: which is correct?
	if (output.a0 == RSI_INCOMPLETE || output.a0 == RSI_SUCCESS)
		return 0;
	else
		return -rsi_ret_to_errno(output.a0);
}

static int do_attestation_continue(phys_addr_t page, struct rsi_attestation *attest)
{
	struct arm_smccc_1_2_regs input = {0}, output = {0};

	input.a0 = SMC_RSI_ATTESTATION_TOKEN_CONTINUE;
	input.a1 = page;

	arm_smccc_1_2_smc(&input, &output);

	if (output.a0 == RSI_SUCCESS) {
		attest->token_len = output.a1;
		return 0;  // we're done
	}

	if (output.a0 == RSI_INCOMPLETE)
		return 1;  // carry on

	return -rsi_ret_to_errno(output.a0);
}

static int do_attestation(struct rsi_attestation *attest)
{
	int ret;
	phys_addr_t page = virt_to_phys(rsi_page_buf);

	mutex_lock(&attestation_call);

	ret = do_attestation_init(page, attest);
	if (ret != 0)
		goto unlock;

	do {
		ret = do_attestation_continue(page, attest);
	} while (ret == 1);

unlock:
	mutex_unlock(&attestation_call);

	if (ret == 0)
		memcpy(attest->token, rsi_page_buf, attest->token_len);

	return ret;
}

static long device_ioctl(struct file *f, unsigned int cmd, unsigned long arg)
{
	int ret = 0;

	uint32_t version = 0;
	struct rsi_measurement *measur = NULL;
	struct rsi_attestation *attest = NULL;

	switch (cmd) {
	case RSIIO_ABI_VERSION:
		printk(RSI_INFO "ioctl: abi_version\n");

		version = (uint32_t)rsi_get_version();
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

	cls = class_create(THIS_MODULE, DEVICE_NAME);
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
