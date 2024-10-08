# Verification

We formally verify Islet using [Kani](https://github.com/model-checking/kani/)'s model
checking. Our verification harnesses adopt the same input and output conditions as well
as similar structures used in [TF-RMM](https://www.trustedfirmware.org/projects/tf-rmm/)'s
harnesses which are extracted from Machine Readable Specification. It would help to check
the conformance of the two systems written in different languages.

## Verification dependency
* [patched Kani](https://github.com/zpzigi754/kani/tree/use-aarch64-latest)

## Available RMI targets

```sh
rmi_features
rmi_granule_delegate
rmi_granule_undelegate
rmi_realm_activate
rmi_rec_aux_count
rmi_rec_destroy
rmi_version
```

## Verifying Islet

```sh
(in islet/model-checking)

# Choose one among the list in `Available RMI targets` for the value of `RMI_TARGET`
$ RMI_TARGET=rmi_granule_undelegate make verify
```

For more about Islet's model-checking, please refer to [here](../islet-model-checking.md).
