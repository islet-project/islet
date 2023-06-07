## Try out confidential code generation in ML setting

This section explains how to try out confidential code generation in ML setting. (For this model, FL is not supported)
We prepare [a docker image](https://github.com/Samsung/islet/releases/download/example-confidential-ml-v1.1/cca_ubuntu_release.tar.gz) that contains everything needed to try out this example and it involves 4 different instances-- *certifier-service*, *runtime*, *model-provider*, *device*-- meaning that you need to open 4 terminals for each of them.

In this example, *device* is not involved in ML operations (inference and training), they just pass user-input on to *runtime* and then *runtime* does inference with the code model and give the result(code) back to *device*.
The code model is a pre-trained model and *runtime* will not do training with user-input. This is the way that most state-of-the-art chatbots work these days.

Note that since this model is a simple text classification model, it might not be able to handle arbitrary requests, that is to say, if you ask a new question that this model is not trained with,
the quality of the output might be low. See [this csv file](./model_provider/code_x_data.csv) to know what requests are supported at this moment.

#### Import and run a docker image

Before trying this example, please do the following first to import and run a docker image.
(Note that this docker image is based on Ubuntu 22.04)
```
$ wget https://github.com/Samsung/islet/releases/download/example-confidential-ml-v1.1/cca_ubuntu_release.tar.gz
$ gzip -d cca_ubuntu_release.tar.gz
$ cat cca_ubuntu_release.tar | sudo docker import - cca_release:latest
$ sudo docker run --net=host -it -d --name=cca_ubuntu_release cca_release /bin/bash
```

#### Static IP settings when testing on FVP

All components but device run on `193.168.10.15` while device runs on `193.168.20.10`,
unless you run FVP with a non-default network configuration.

#### How to test with simulated enclave (no actual hardware TEE) on x86_64

```
// In this case, all instances are running on the same host PC so all IPs are set to 0.0.0.0.
// If you want to do with a remote configuration, set a proper IP to init.sh/run.sh of each terminal.
//
// In ML case, only one device is enough to show how it works.
// For each terminal, you need to go in the docker image using "docker exec".

$ <terminal-1> sudo docker exec -it cca_ubuntu_release /bin/bash
$ <terminal-1: certifier-service> cd /islet/examples/confidential-ml/certifier-service
$ <terminal-1: certifier-service> ./run.sh x86_64

$ <terminal-2> sudo docker exec -it cca_ubuntu_release /bin/bash
$ <terminal-2: runtime> cd /islet/examples/confidential-ml/runtime
$ <terminal-2: runtime> ./build.sh  # a one-time need. you can skip it if it's already built.
$ <terminal-2: runtime> ./init.sh   # asks certifier-service to do attestation and authentication
$ <terminal-2: runtime> ./run.sh code 0    # run ML server (1st arg indictates "code model" while 2nd arg indicates "ML")

$ <terminal-3> sudo docker exec -it cca_ubuntu_release /bin/bash
$ <terminal-3: model-provider> cd /islet/examples/confidential-ml/model-provider
$ <terminal-3: model-provider> ./build.sh   # a one-time need
$ <terminal-3: model-provider> ./init.sh    # asks certifier-service to do attestation and authentication
$ <terminal-3: model-provider> ./run.sh model_code.tflite    # sends a code model to runtime
   send-model done, size: 77820
   ACK: 77820  # you can see this message if there is no problem in sending a model.

$ <terminal-4> sudo docker exec -it cca_ubuntu_release /bin/bash
$ <terminal-4: device> cd /islet/examples/confidential-ml/device
$ <terminal-4: device> ./build.sh  # a one-time need
$ <terminal-4: device> ./init.sh 0.0.0.0
$ <terminal-4: device> ./run.sh 0.0.0.0 8125 code 0
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

#### How to test with ISLET

In this setting, three instances (*certifier-service*, *runtime*, *model-provider*) run on the host PC directly while only one instance (*device*) runs on ARM FVP on the same host PC.

And then, run three instances on the host PC directly.
```
// For each terminal, you need to go in the docker image using "docker exec".

$ <terminal-1> sudo docker exec -it cca_ubuntu_release /bin/bash
$ <terminal-1: certifier-service> cd /islet/examples/confidential-ml/certifier-service
$ <terminal-1: certifier-service> ./run.sh x86_64 193.168.10.15

$ <terminal-2> sudo docker exec -it cca_ubuntu_release /bin/bash
$ <terminal-2: runtime> cd /islet/examples/confidential-ml/runtime
$ <terminal-2: runtime> ./build.sh  # a one-time need. you can skip it if it's already built.
$ <terminal-2: runtime> ./init.sh   # asks certifier-service to do attestation and authentication
$ <terminal-2: runtime> ./run.sh 193.168.10.15 code 0    # run ML server (1st arg indictates "code model" while 2nd arg indicates "ML")

$ <terminal-3> sudo docker exec -it cca_ubuntu_release /bin/bash
$ <terminal-3: model-provider> cd /islet/examples/confidential-ml/model-providers
$ <terminal-3: model-provider> ./build.sh   # a one-time need
$ <terminal-3: model-provider> ./init.sh    # asks certifier-service to do attestation and authentication
$ <terminal-3: model-provider> ./run.sh 193.168.10.15 model_code.tflite    # sends a word prediction model to runtime
   send-model done, size: 77820
   ACK: 77820  # you can see this message if there is no problem in sending a model.
```

And then, launch ARM FVP with networking enabled and run *device* on top of that.
```
# [in PC Host] run fvp-cca with a proper network configuration. The following command takes default network configruations, which is recommended.
$ ./scripts/fvp-cca --normal-world=linux-net --realm=linux --rmm=tf-rmm

# [in FVP Host] run a realm with a rootfs that contains prebuilt example binaries.
$ ./launch-realm.sh

# [in Realm] run device using a prebuilt binary
$ <terminal-4: device> cd /shared
$ <terminal-4: device> ./set-realm-ip.sh   # set static ip address first
$ <terminal-4: device> cd /shared/examples/confidential-ml/device
$ <terminal-4: device> ./init_aarch64.sh 193.168.10.15  # 193.168.10.15 is the IP address for host
$ <terminal-4: device> ./run_aarch64.sh 193.168.10.15 8125 code 0 -1 -1 193.168.20.10
   Type command: write a function to add two numbers  # type in what you want
   prediction: int add2(int a, int b) {  # provide a function you ask
      return a + b;
   }
```

#### How to test with GUI (on simulated enclave)

```
// make sure that you are in a docker image
// In this case, all instances are running on the same host PC.

$ <terminal-1> sudo docker exec -it cca_ubuntu_release /bin/bash
$ <terminal-1: certifier-service> cd /islet/examples/confidential-ml/certifier-service
$ <terminal-1: certifier-service> ./run.sh x86_64

$ <terminal-2> sudo docker exec -it cca_ubuntu_release /bin/bash
$ <terminal-2: gui-server> cd /islet/examples/confidential-ml/gui-server/device-chatbot
$ <terminal-2: gui-server> ./run.sh 0.0.0.0 0.0.0.0 8127 8128 &  # that server runs at the port of 8000 and is for device
  Chatbot is listening on port 8000!  # you can connect http://localhost:8000 to see what this chatbot look like

$ <terminal-2: gui-server> cd../runtime-log
$ <terminal-2: gui-server> ./run.sh 3000 3001 &  # 3000 for runtime, 3001 for showing the screen of "HACKED"

$ <terminal-3> sudo docker exec -it cca_ubuntu_release /bin/bash
$ <terminal-3: runtime> cd /islet/examples/confidential-ml/runtime
$ <terminal-3: runtime> ./build.sh  # a one-time need. you can skip it if it's already built.
$ <terminal-3: runtime> ./init.sh   # asks certifier-service to do attestation and authentication
$ <terminal-3: runtime> ./run.sh 0.0.0.0 code 0 example_app.measurement 3000 0   # run ML server, 3000 is the port of GUI server
  # NOTE: for security test (to show attack goes successful), do "./run.sh code 0 example_app.measurement 3000 1" (1 for the last argument)
  # NOTE: for security test (to show it prevents attacks), do "./run.sh code 0 malicious_app.measurement 3000 1"

$ <terminal-4> sudo docker exec -it cca_ubuntu_release /bin/bash
$ <terminal-4: model-provider> cd /islet/examples/confidential-ml/model-providers
$ <terminal-4: model-provider> ./build.sh   # a one-time need
$ <terminal-4: model-provider> ./init.sh    # asks certifier-service to do attestation and authentication
$ <terminal-4: model-provider> ./run.sh 0.0.0.0 model_code.tflite    # sends a word prediction model to runtime
   send-model done, size: 77820
   ACK: 77820  # you can see this message if there is no problem in sending a model.

$ <terminal-5> sudo docker exec -it cca_ubuntu_release /bin/bash
$ <terminal-5: device> cd /islet/examples/confidential-ml/device
$ <terminal-5: device> ./build.sh  # a one-time need
$ <terminal-5: device> ./init.sh 0.0.0.0
$ <terminal-5: device> ./run.sh 0.0.0.0 8125 code 0 8127 8128
   wait for input from GUI..

$ <browser> open a browser and go in http://localhost:8000
$ <browser> type a request in the chatbox, such as "write a function to add two numbers",
  and then device(terminal-5) passes the request on to the runtime(terminal-3),
  and eventually, the chatbot in the browser will show the prediction (code) made by runtime.
  
  Here is the code:
  int min(int a, int b) {
      return a > b ? b : a;
  }
  I hope it helps!
```

#### How to test with GUI (on FVP)

Run all components but device on the host.
```
// For each terminal, you need to go in the docker image using "docker exec".

$ <terminal-1> sudo docker exec -it cca_ubuntu_release /bin/bash
$ <terminal-1: certifier-service> cd /islet/examples/confidential-ml/certifier-service
$ <terminal-1: certifier-service> ./run.sh x86_64 193.168.10.15

$ <terminal-2> sudo docker exec -it cca_ubuntu_release /bin/bash
$ <terminal-2: gui-server> cd /islet/examples/confidential-ml/gui-server/device-chatbot
$ <terminal-2: gui-server> ./run.sh 193.168.10.15 193.168.20.10 8127 8128  # that server runs at the port of 8000 and is for device
  Chatbot is listening on port 8000!  # you can connect http://localhost:8000 to see what this chatbot look like

$ <terminal-3> sudo docker exec -it cca_ubuntu_release /bin/bash
$ <terminal-3: runtime> cd /islet/examples/confidential-ml/runtime
$ <terminal-3: runtime> ./build.sh  # a one-time need. you can skip it if it's already built.
$ <terminal-3: runtime> ./init.sh   # asks certifier-service to do attestation and authentication
$ <terminal-3: runtime> ./run.sh 193.168.10.15 code 0    # run ML server (1st arg indictates "code model" while 2nd arg indicates "ML")

$ <terminal-4> sudo docker exec -it cca_ubuntu_release /bin/bash
$ <terminal-4: model-provider> cd /islet/examples/confidential-ml/model-providers
$ <terminal-4: model-provider> ./build.sh   # a one-time need
$ <terminal-4: model-provider> ./init.sh    # asks certifier-service to do attestation and authentication
$ <terminal-4: model-provider> ./run.sh 193.168.10.15 model_code.tflite    # sends a word prediction model to runtime
   send-model done, size: 77820
   ACK: 77820  # you can see this message if there is no problem in sending a model.
```

And then, launch ARM FVP with networking enabled and run *device* on top of that.
```
# [in PC Host] run fvp-cca with a proper network configuration. The following command takes default network configruations, which is recommended.
$ ./scripts/fvp-cca --normal-world=linux-net --realm=linux --rmm=tf-rmm

# [in FVP Host] run a realm with a rootfs that contains prebuilt example binaries.
$ ./launch-realm.sh

# [in Realm] run device using a prebuilt binary
$ <terminal-5: device> cd /shared
$ <terminal-5: device> ./set-realm-ip.sh   # set static ip address first
$ <terminal-5: device> cd /shared/examples/confidential-ml/device
$ <terminal-5: device> ./init_aarch64.sh 193.168.10.15  # 193.168.10.15 is the IP address for host
$ <terminal-5: device> ./run_aarch64.sh 193.168.10.15 8125 code 0 8127 8128 193.168.20.10
   wait for input from GUI..

$ <browser> open a browser and go in http://193.168.10.15:8000
$ <browser> type a request in the chatbox, such as "write a function to add two numbers",
  and then device(terminal-5) passes the request on to the runtime(terminal-3),
  and eventually, the chatbot in the browser will show the prediction (code) made by runtime.
  
  Here is the code:
  int min(int a, int b) {
      return a > b ? b : a;
  }
  I hope it helps!
```
