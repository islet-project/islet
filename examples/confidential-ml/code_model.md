# Try out confidential code generation in ML setting

This section explains how to try out confidential code generation in ML setting. (For this model, FL is not supported)
This example involves 4 different instances-- *certifier-service*, *runtime*, *model-provider*, *device*-- meaning that you need to open 4 terminals for each of them.

In this example, *device* is not involved in ML operations (inference and training), they just pass user-input on to *runtime* and then *runtime* does inference with the code model and give the result(code) back to *device*.
The code model is a pre-trained model and *runtime* will not do training with user-input. This is the way that most state-of-the-art chatbots work these days.

Note that since this model is a simple text classification model, it might not be able to handle arbitrary requests, that is to say, if you ask a new question that this model is not trained with,
the quality of the output might be low. See [this csv file](./model_provider/code_x_data.csv) to know what requests are supported at this moment.

## Prerequisite

### 1. Install dependencies

```
$ cd $(islet)/examples/confidential-ml
$ ./setup.sh
$ ./setup-tf.sh  # it takes a pretty while..
```

### 2. Configure Measurements and Policies

```
$ ./provisioning.sh
```

Currently, unlike cross-platform-e2ee example, this example has not yet taken a real CCA attestation report.
It needs to be updated in a way that takes a real CCA report from Islet HES.

### 3. Run network enabled FVP

Run a network-enabled FVP first
```
$ cd $(islet)
$ ./scripts/fvp-cca --normal-world=linux-net --realm=linux --hes --no-telnet

# if you've successfully built Islet, add "--run-only" when running fvp-cca. It speeds things up a lot.
```

Then, connects to the FVP via telnet
```
$ telnet localhost 5000
```

In the host Linux environment on FVP, launch the realm:
```
$ ./launch-realm.sh net
```

Finally, in the realm Linux environment on FVP, set the realm IP address and load the RSI kernel module:
```
$ cd /shared
$ ./set-realm-ip.sh
$ insmod rsi.ko
```

### 4. Build all programs

```
$ cd /islet/examples/confidential-ml/
$ ./build-service.sh  # build certifier service
$ cd runtime
$ ./build.sh  # build runtime
$ cd ../model-provider
$ ./build.sh  # build model-provider
$ cd ../device
$ ./build.sh  # build device
```

## How to test with simulated enclave (no actual hardware TEE) on x86_64

```
// All terminals are a host terminal, not FVP's.

$ <terminal-1: certifier-service> cd /islet/examples/confidential-ml/
$ <terminal-1: certifier-service> ./run-service.sh

$ <terminal-2: runtime> cd /islet/examples/confidential-ml/runtime
$ <terminal-2: runtime> ./run.sh

$ <terminal-3: model-provider> cd /islet/examples/confidential-ml/model-provider
$ <terminal-3: model-provider> ./run.sh
   send-model done, size: 77820
   ACK: 77820  # you can see this message if there is no problem in sending a model.

$ <terminal-4: device> cd /islet/examples/confidential-ml/device
$ <terminal-4: device> ./run.sh
   Type command: write a function to add two numbers  # type in what you want
   prediction: int add2(int a, int b) {  # provide a function you ask
      return a + b;
   }

// when you make a request, the following log comes out in runtime (terminal-2),
// which shows "inference" has been done.
---- prediction ----
int add2(int a, int b) {
    return a + b;
}
---- inference done! ----
```

## How to test with Islet (WIP)

WIP
