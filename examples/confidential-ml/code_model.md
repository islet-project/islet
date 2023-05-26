## Try out confidential code generation in ML setting

This section explains how to try out confidential code generation in ML setting. (For this model, FL is not supported)
We prepare [a docker image](https://github.com/Samsung/islet/releases/download/example-confidential-ml-v1.0/cca_ubuntu_release.tar.gz) that contains everything needed to try out this example and it involves 4 different instances-- *certifier-service*, *runtime*, *model-provider*, *device*-- meaning that you need to open 4 terminals for each of them.

In this example, *device* is not involved in ML operations (inference and training), they just pass user-input on to *runtime* and then *runtime* does inference with the code model and give the result(code) back to *device*.
The code model is a pre-trained model and *runtime* will not do training with user-input. This is the way that most state-of-the-art chatbots work these days.

Note that since this model is a simple text classification model, it might not be able to handle arbitrary requests, that is to say, if you ask a new question that this model is not trained with,
the quality of the output might be low. See [this csv file](./model_provider/code_x_data.csv) to know what requests are supported at this moment.

#### How to test with simulated enclave (no actual hardware TEE) on x86_64

```
// make sure that you are in a docker image
// In this case, all instances are running on the same host PC.
// In ML case, only one device is enough to show how it works.

$ <terminal-1: certifier-service> cd /islet/examples/confidential-ml/certifier-service
$ <terminal-1: certifier-service> ./run.sh x86_64

$ <terminal-2: runtime> cd /islet/examples/confidential-ml/runtime
$ <terminal-2: runtime> ./build.sh  # a one-time need. you can skip it if it's already built.
$ <terminal-2: runtime> ./init.sh   # asks certifier-service to do attestation and authentication
$ <terminal-2: runtime> ./run.sh code 0    # run ML server (1st arg indictates "code model" while 2nd arg indicates "ML")

$ <terminal-3: model-provider> cd /islet/examples/confidential-ml/model-providers
$ <terminal-3: model-provider> ./build.sh   # a one-time need
$ <terminal-3: model-provider> ./init.sh    # asks certifier-service to do attestation and authentication
$ <terminal-3: model-provider> ./run.sh model_code.tflite    # sends a word prediction model to runtime
   send-model done, size: 77820
   ACK: 77820  # you can see this message if there is no problem in sending a model.

$ <terminal-4: device> cd /islet/examples/confidential-ml/device
$ <terminal-4: device> ./build.sh  # a one-time need
$ <terminal-4: device> ./init.sh 0.0.0.0
$ <terminal-4: device> ./run.sh 0.0.0.0 8125 code 0
   Type command: write a function to add two numbers  # type in what you want
   prediction: int add2(int a, int b) {  # provide a function you ask
      return a + b;
   }

// when you type a correct answer in device, the following log comes out in runtime,
// which shows "inference" has been done.
---- prediction ----
int add2(int a, int b) {
    return a + b;
}
---- inference done! ----
```

#### How to test with ISLET

In this setting, three instances (*certifier-service*, *runtime*, *model-provider*) run on the host PC directly while only one instance (*device*) runs on ARM FVP on the same host PC.

First of all, be sure to run a docker image with the following options to be able to interact with ARM FVP.
```
# we have to allow port 8123,8124,8125,8126 that are used to communicate with ARM FVP.
$ sudo docker run --net=bridge -it -p 8123:8123 -p 8124:8124 -p 8125:8125 -p 8126:8126
```

And then, run three instances on the host PC directly.
```
// make sure that you are in a docker image

$ <terminal-1: certifier-service> cd /islet/examples/confidential-ml/certifier-service
$ <terminal-1: certifier-service> ./run.sh x86_64

$ <terminal-2: runtime> cd /islet/examples/confidential-ml/runtime
$ <terminal-2: runtime> ./build.sh  # a one-time need. you can skip it if it's already built.
$ <terminal-2: runtime> ./init.sh   # asks certifier-service to do attestation and authentication
$ <terminal-2: runtime> ./run.sh code 0    # run ML server (1st arg indictates "code model" while 2nd arg indicates "ML")

$ <terminal-3: model-provider> cd /islet/examples/confidential-ml/model-providers
$ <terminal-3: model-provider> ./build.sh   # a one-time need
$ <terminal-3: model-provider> ./init.sh    # asks certifier-service to do attestation and authentication
$ <terminal-3: model-provider> ./run.sh model_code.tflite    # sends a word prediction model to runtime
   send-model done, size: 77820
   ACK: 77820  # you can see this message if there is no problem in sending a model.
```

And then, launch ARM FVP with networking enabled and run *device* on top of that.
```
# [in PC Host] run fvp-cca with a proper network configuration. To get what these arguments mean, see 'NETWORK.md'.
$ ./scripts/fvp-cca --normal-world=linux-net --realm=linux --rmm=tf-rmm --host-ip=<PC Host IP> --ifname=<ethernet card name> --gateway=<gateway address> --fvp-ip=<FVP IP>

# [in FVP Host] once fvp is launched, run a daemon process for packet forwarding.
$ ./rinetd -c rinetd.conf -f &

# [in FVP Host] run a realm with a rootfs that contains prebuilt example binaries.
$ ./launch-realm.sh

# [in Realm] run device using a prebuilt binary
$ <terminal-4: device> cd /shared/examples/confidential-ml/device
$ <terminal-4: device> ./init.sh 192.168.33.1
$ <terminal-4: device> ./run.sh 192.168.33.1 8125 code 0
   Type command: write a function to add two numbers  # type in what you want
   prediction: int add2(int a, int b) {  # provide a function you ask
      return a + b;
   }
```
