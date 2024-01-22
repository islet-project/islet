## Try out confidential word prediction in ML setting

This section explains how to try out confidential word prediction in ML setting.
We prepare [a docker image](https://github.com/islet-project/islet/releases/download/example-confidential-ml-v1.1/cca_ubuntu_release.tar.gz) that contains everything needed to try out this example and it involves 5 different instances-- *certifier-service*, *runtime*, *model-provider*, *device1*, *device2*-- meaning that you need to open 5 terminals for each of them.

[TODO] Note that as of now we do not offer any convenient way to try out this example in your host machine directly instead of the docker image, as this example involves a lot of dependencies. Anyhow, we plan to support building and testing this example on the host PC in the near future.

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
$ <terminal-2: runtime> ./run.sh word 0    # run ML server (1st arg indictates "word model" while 2nd arg indicates "ML")

$ <terminal-3: model-provider> cd /islet/examples/confidential-ml/model-provider
$ <terminal-3: model-provider> ./build.sh   # a one-time need
$ <terminal-3: model-provider> ./init.sh    # asks certifier-service to do attestation and authentication
$ <terminal-3: model-provider> ./run.sh model.tflite    # sends a word prediction model to runtime
   send-model done, size: 69380
   ACK: 69380  # you can see this message if there is no problem in sending a model.

$ <terminal-4: device1> cd /islet/examples/confidential-ml/device
$ <terminal-4: device1> ./build.sh  # a one-time need
$ <terminal-4: device1> ./init.sh 0.0.0.0
$ <terminal-4: device1> ./run.sh 0.0.0.0 8125 word 0
   Type characters: abo         # type in the first three characters of any five letter words
   Prediction: abou{            # this is an initial prediction as a result of on-device inference
   Type correct answer: about   # provide a correct answer for training
   ...
   ...                          # sends "about" to runtime. runtime does training with this data and sends a newly trained model to this device.
   ...
   Type characters: abo         # type in "abo" again and see if it leads to "about" which is a correct word.
   Prediction: about            # shows a correct guess-!

// when you type a correct answer in device1, the following log comes out in runtime,
// currently, it runs 100 epochs that are enough to reach close to the loss of zero.
---- do training.... ----
epoch: 0, loss: 0.041,0.069
epoch: 10, loss: 0.023,0.037
epoch: 20, loss: 0.014,0.021
epoch: 30, loss: 0.009,0.012
epoch: 40, loss: 0.005,0.007
epoch: 50, loss: 0.003,0.004
epoch: 60, loss: 0.001,0.002
epoch: 70, loss: 0.001,0.001
epoch: 80, loss: 0.000,0.001
epoch: 90, loss: 0.000,0.000
---- training done! ----
```

#### How to test with Islet

In this setting, three instances (*certifier-service*, *runtime*, *model-provider*) run on the host PC directly while only one instance (*device1*) runs on ARM FVP on the same host PC.

[TODO] Note that in this setting *device1* runs on ARM FVP but it does not use Islet's attestation APIs as of now. Once Islet's attestation APIs get merged into *certifier framework*, it accordingly gets switched to using Islet attestation APIs instead of simulated enclave.

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
$ <terminal-2: runtime> ./run.sh word 0  # run ML server

$ <terminal-3: model-provider> cd /islet/examples/confidential-ml/model-provider
$ <terminal-3: model-provider> ./build.sh   # a one-time need
$ <terminal-3: model-provider> ./init.sh    # asks certifier-service to do attestation and authentication
$ <terminal-3: model-provider> ./run.sh model.tflite    # sends a word prediction model to runtime
   send-model done, size: 69380
   ACK: 69380  # you can see this message if there is no problem in sending a model.
```

And then, launch ARM FVP with networking enabled and run *device1* on top of that.
```
# [in PC Host] run fvp-cca with a proper network configuration. To get what these arguments mean, see 'NETWORK.md'.
$ ./scripts/fvp-cca --normal-world=linux-net --realm=linux --rmm=tf-rmm --host-ip=<PC Host IP> --ifname=<ethernet card name> --gateway=<gateway address> --fvp-ip=<FVP IP>

# [in FVP Host] once fvp is launched, run a daemon process for packet forwarding.
$ ./rinetd -c rinetd.conf -f &

# [in FVP Host] run a realm with a rootfs that contains prebuilt example binaries.
$ ./launch-realm.sh

# [in Realm] run device1 using a prebuilt binary
$ <terminal-4: device1> cd /shared/examples/confidential-ml/device
$ <terminal-4: device1> ./init.sh 192.168.33.1
$ <terminal-4: device1> ./run.sh 192.168.33.1 8125 word 0
   Type characters: abo         # type in the first three characters of any five letter words
   Prediction: abou{            # this is an initial prediction as a result of on-device inference
   Type correct answer: about   # provide a correct answer for training
   ...
   ...                          # sends "about" to runtime. runtime does training with this data and sends a newly trained model to this device.
   ...
   Type characters: abo         # type in "abo" again and see if it leads to "about" which is a correct word.
   Prediction: about            # shows a correct guess-!
```

## Try out confidential word prediction in FL setting

This section explains how to try out confidential word prediction in FL setting. We make a simple word prediction model that is based on SimpleRNN of TensorFlow.
We prepare [a docker image](https://github.com/islet-project/islet/releases/download/example-confidential-ml-v1.1/cca_ubuntu_release.tar.gz) that contains everything needed to try out this example and it involves 5 different instances-- *certifier-service*, *runtime*, *model-provider*, *device1*, *device2*-- meaning that you need to open 5 terminals for each of them.s

[TODO] Note that as of now we do not offer any convenient way to try out this example in your host machine directly instead of the docker image, as this example involves a lot of dependencies. Anyhow, we plan to support building and testing this example on the host PC in the near future.

#### How to test with simulated enclave (no actual hardware TEE) on x86_64
```
// make sure that you are in a docker image
// In this case, all instances are running on the same host PC.

$ <terminal-1: certifier-service> cd /islet/examples/confidential-ml/certifier-service
$ <terminal-1: certifier-service> ./run.sh x86_64

$ <terminal-2: runtime> cd /islet/examples/confidential-ml/runtime
$ <terminal-2: runtime> ./build.sh  # a one-time need. you can skip it if it's already built.
$ <terminal-2: runtime> ./init.sh   # asks certifier-service to do attestation and authentication
$ <terminal-2: runtime> ./run.sh word 1   # run FL server

$ <terminal-3: model-provider> cd /islet/examples/confidential-ml/model-provider
$ <terminal-3: model-provider> ./build.sh   # a one-time need
$ <terminal-3: model-provider> ./init.sh    # asks certifier-service to do attestation and authentication
$ <terminal-3: model-provider> ./run.sh model.tflite    # sends a word prediction model to runtime
   send-model done, size: 69380
   ACK: 69380  # you can see this message if there is no problem in sending a model.

$ <terminal-4: device1> cd /islet/examples/confidential-ml/device
$ <terminal-4: device1> ./build.sh  # a one-time need
$ <terminal-4: device1> ./init.sh 0.0.0.0
$ <terminal-4: device1> ./run.sh 0.0.0.0 8125 word 1
   Type characters: abo         # type in the first three characters of any five letter words
   Prediction: abou{            # this is an initial prediction as a result of on-device inference
   Type correct answer: about   # provide a correct answer for training
   ...
   epoch: 90, loss: 0.000,0.000 # do on-device training
   ...                          # wait for a global model from runtime
   ...

$ <terminal-5: device2> cd /islet/examples/confidential-ml/device
$ <terminal-5: device2> ./init.sh 0.0.0.0
$ <terminal-5: device2> ./run.sh 0.0.0.0 8126 word 1
   Type characters: whi          # type in the first three characters of any five letter words
   prediction: whihh             # initial prediction
   Type correct answer: white    # provide a correct answer for training
   ...
   epoch: 90, loss: 0.000,0.000  # training
   ...
   ...                           # download a global model from server
   Type characters: abo          # type in characters that device1 put
   Prediction: about             # shows a correct guess, as the global model reflects "about" which device1 typed in

$ <terminal-4: device1>  # once downloading a gloabl model successfully, it gets back a shell prompt.
   Type characters: whi
   prediction: white             # shows a correct answer, as the global model reflects "white" which device2 typed in
```

#### How to test with Islet

In this setting, four instances (*certifier-service*, *runtime*, *model-provider*, *device1*) run on the host PC directly while only one instance (*device2*) runs on ARM FVP on the same host PC.

[TODO] Note that in this setting *device1* runs on ARM FVP but it does not use Islet's attestation APIs as of now. Once Islet's attestation APIs get merged into *certifier framework*, it accordingly gets switched to using Islet attestation APIs instead of simulated enclave.

First of all, be sure to run a docker image with the following options to be able to interact with ARM FVP.
```
# we have to allow port 8123,8124,8125,8126 that are used to communicate with ARM FVP.
$ sudo docker run --net=bridge -it -p 8123:8123 -p 8124:8124 -p 8125:8125 -p 8126:8126
```

And then, run four instances on the host PC directly.
```
// make sure that you are in a docker image

$ <terminal-1: certifier-service> cd /islet/examples/confidential-ml/certifier-service
$ <terminal-1: certifier-service> ./run.sh x86_64

$ <terminal-2: runtime> cd /islet/examples/confidential-ml/runtime
$ <terminal-2: runtime> ./build.sh  # a one-time need. you can skip it if it's already built.
$ <terminal-2: runtime> ./init.sh   # asks certifier-service to do attestation and authentication
$ <terminal-2: runtime> ./run.sh word 1   # run ML server

$ <terminal-3: model-provider> cd /islet/examples/confidential-ml/model-provider
$ <terminal-3: model-provider> ./build.sh   # a one-time need
$ <terminal-3: model-provider> ./init.sh    # asks certifier-service to do attestation and authentication
$ <terminal-3: model-provider> ./run.sh model.tflite     # sends a word prediction model to runtime
   send-model done, size: 69380
   ACK: 69380  # you can see this message if there is no problem in sending a model.

$ <terminal-4: device1> cd /islet/examples/confidential-ml/device
$ <terminal-4: device1> ./build.sh  # a one-time need
$ <terminal-4: device1> ./init.sh 0.0.0.0
$ <terminal-4: device1> ./run.sh 0.0.0.0 8125 word 1
  # test it the same way we did with "How to test with simulated enclave (no actual hardware TEE) on x86_64"
```

And then, launch ARM FVP with networking enabled and run *device2* on top of that.
```
# [in PC Host] run fvp-cca with a proper network configuration. To get what these arguments mean, see 'NETWORK.md'.
$ ./scripts/fvp-cca --normal-world=linux-net --realm=linux --rmm=tf-rmm --host-ip=<PC Host IP> --ifname=<ethernet card name> --gateway=<gateway address> --fvp-ip=<FVP IP>

# [in FVP Host] once fvp is launched, run a daemon process for packet forwarding.
$ ./rinetd -c rinetd.conf -f &

# [in FVP Host] run a realm with a rootfs that contains prebuilt example binaries.
$ ./launch-realm.sh

# [in Realm] run device2 using a prebuilt binary
$ <terminal-5: device2> cd /shared/examples/confidential-ml/device
$ <terminal-5: device2> ./init.sh 192.168.33.1
$ <terminal-5: device2> ./run.sh 192.168.33.1 8126 word 1
  # test it the same way we did with "How to test with simulated enclave (no actual hardware TEE) on x86_64"
``
