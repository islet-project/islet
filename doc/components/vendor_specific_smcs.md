# Vendor-Specific SMC Allocation for Realm Metadata Mechanism and Sealing Keys Derivation

According to the "SMC Calling Convention" document ([ARM DEN 0028F](https://developer.arm.com/documentation/den0028/e/)), there are two SMC Function Identifier ranges that can be used for vendor-specific purposes (*6.1 Allocation of values*, Table 6-2; note that we're interested only in SMC64 calls).

| SMC Function identifier | Service type |
| --- | --- |
| 0xC6000000-0xC600FFFF | SMC64: Vendor Specific Hypervisor Service Calls |
| 0xC7000000-0xC700FFFF | SMC64: Vendor Specific EL3 Monitor Service Calls |

For our purposes, we split one of these ranges similarly to the "SMC: Standard Service Calls" to implement additional, non-standard RMI, RSI, and RMM EL3 calls.

If we choose "SMC64: Vendor-Specific EL3 Monitor Service Calls", we can split this region into three ranges.

| SMC Function identifier | Reserved for |
| --- | --- |
| 0xC7000150-0xC700018F | Vendor-specific RMI |
| 0xC7000190-0xC70001AF | Vendor-specific RSI |
| 0xC70001B0-0xC70001CF | Vendor-specific RMMD EL3 Service calls |

Thus, for the purposes of the **Realm Metadata** mechanism and **Sealing Keys Derivation**, the following SMCs have been reserved:

| SMC Function identifier | Command name | Description |
| --- | --- | --- |
| 0xC7000150 | RMI_ISLET_REALM_SET_METADATA | Assigns the Realm Metadata to a particular realm (for more details, refer to the [Realm metadata](./realm_metadata.md) document) |
| 0xC7000191 | RSI_ISLET_REALM_SEALING_KEY | Retrieves the Sealing Key (for more details, refer to the [Sealing Key Derivation](./sealing.md) document) |
| 0xC70001B0 | RMM_ISLET_GET_VHUK | Retrieves the VHUK from the RMMD service (for more details, refer to the [Sealing Key Derivation](./sealing.md) document) |

The **_ISLET_** infix has been added to command names to indicate that these SMCs come from a particular project (Islet) and to prevent possible naming conflicts in the future.
