#include <gflags/gflags.h>

#include "support.h"
#include "certifier.h"
#include "simulated_enclave.h"
#include "cc_helpers.h"

#include <sys/socket.h>
#include <arpa/inet.h>
#include <netinet/in.h>
#include <netdb.h>
#include <openssl/ssl.h>
#include <openssl/rsa.h>
#include <openssl/x509.h>
#include <openssl/evp.h>
#include <openssl/rand.h>
#include <openssl/hmac.h>
#include <openssl/err.h>
#include <pthread.h>

// tensorflow-
#include "tensorflow/lite/kernels/kernel_util.h"
#include "tensorflow/lite/kernels/register.h"
#include "tensorflow/lite/string_util.h"
#include <random>
#include <vector>
#include <iostream>
#include <stdlib.h>
#include <stdio.h>

DEFINE_bool(print_all, false,  "verbose");
DEFINE_string(operation, "", "operation");

DEFINE_string(policy_host, "localhost", "address for policy server");
DEFINE_int32(policy_port, 8123, "port for policy server");
DEFINE_string(data_dir, "./data/", "directory for application data");

DEFINE_string(runtime_host, "localhost", "address for runtime");
DEFINE_int32(runtime_model_port, 8124, "port for runtime (used to deliver model)");
DEFINE_int32(runtime_data_port, 8125, "port for runtime (used to deliver data for device1)");
DEFINE_int32(gui_rx_port, 8127, "port for receiving data from gui");
DEFINE_int32(gui_tx_port, 8128, "port for sending data to gui");

DEFINE_string(policy_store_file, "store.bin", "policy store file name");
DEFINE_string(platform_file_name, "platform_file.bin", "platform certificate");
DEFINE_string(platform_attest_endorsement, "platform_attest_endorsement.bin", "platform endorsement of attest key");
DEFINE_string(attest_key_file, "attest_key_file.bin", "attest key");
DEFINE_string(measurement_file, "example_app.measurement", "measurement");

#include "../certifier-data/policy_key.cc"
#include "../common/word_model.h"
#include "../common/util.h"

static cc_trust_data* app_trust_data = nullptr;
static char ckpt_path[512] = {0,}; // "./checkpoint/model.ckpt";
static WordPredictionModel word_model;

using namespace std;

void download_model(secure_authenticated_channel& channel) {
  string model;
  const char *request_str = "download_tflite_model";

  channel.write(strlen(request_str), (unsigned char *)request_str);
  int n = channel.read(&model);
  if (n <= 0) {
    printf("download model error\n");
    return;
  }
  printf("model read done: %d\n", n);
  word_model.init((unsigned char *)model.data(), n);
}

void download_trained_weights(secure_authenticated_channel& channel) {
  string weights;
  int n = channel.read(&weights);
  if (n <= 0) {
    printf("download trained weights error\n");
    return;
  }
  save_as_file(ckpt_path, (unsigned char *)weights.data(), n);
}

void inference(unsigned char *input_word, unsigned char *out_prediction) {
  word_model.infer((char *)input_word, ckpt_path, out_prediction);
}

void training(unsigned char *input_word) {
  word_model.train((char *)input_word, ckpt_path);
}

void update_data(secure_authenticated_channel& channel, unsigned char *word) {
  // upload data
  channel.write(strlen((const char *)word), word);

  // download a new model
  download_trained_weights(channel);
}

void update_model(secure_authenticated_channel& channel) {
  string global_model;
  unsigned char local_model[64 * 1024] = {0,};

  size_t len = read_file(ckpt_path, local_model, sizeof(local_model));
  if (len == 0) {
    printf("read_file error\n");
    return;
  }

  // upload local model
  channel.write(len, local_model);

  // download global model
  int read_len = channel.read(&global_model);
  if (read_len <= 0) {
    printf("download global_model error\n");
    return;
  }

  save_as_file(ckpt_path, (unsigned char *)global_model.data(), read_len);
  word_model.restore(ckpt_path);
}

void run_shell(secure_authenticated_channel& channel, bool is_federated_learning) {
  // make sure we have a proper model
  download_model(channel);

  // main loop
  while (1) {
    unsigned char msg[256] = {0,};
    unsigned char correct_answer[256] = {0,};
    unsigned char out_prediction[256] = {0,};

    printf("\n");
    printf("Type characters: ");
    scanf("%s", msg);
    inference(msg, out_prediction);
    printf("Prediction: %s\n", out_prediction);
    printf("Type correct answer: ");
    scanf("%s", correct_answer);

    if (is_federated_learning) {
      training(correct_answer);
      update_model(channel);
    } else {
      update_data(channel, correct_answer);
    }
  }
}

void run_gui(secure_authenticated_channel& channel, bool is_federated_learning) {
  download_model(channel);

  while(1) {
    unsigned char read_cmd[256] = {0,};
    unsigned char write_cmd[256] = {0,};

    unsigned char input[1024] = {0,};
    unsigned char out_prediction[1024] = {0,};
    unsigned char correct_answer[1024] = {0,};
    FILE *fp;
    bool pipe_error = false;

    // 1. wait for input from GUI (browser) (using netcat)
    printf("wait for input from GUI..\n");
    sprintf((char *)read_cmd, "nc -l -p %d -q 1 < /dev/null", FLAGS_gui_rx_port);

    // 2. read input
    fp = popen((const char *)read_cmd, "r");
    if (fp == NULL) {
      printf("popen fail\n");
      return;
    }
    char *r = fgets((char *)input, sizeof(input), fp);
    pclose(fp);
    if (r == NULL) {
      printf("pipe null\n");
      pipe_error = true;
    }
    printf("read input from GUI: %s\n", input);

    if (pipe_error) {
      sprintf((char *)out_prediction, "something wrong on device side. please retry");
    } else {
      // 3. do inference
      inference(input, out_prediction);
      printf("Prediction: %s\n", out_prediction);

      // 4. read correct_answer
      // As of now, simply assume input is synonymous to the correct answer.
      printf("Correct answer: %s\n", input);
    }

    // 5. send prediction
    sleep(1);
    sprintf((char *)write_cmd, "echo %s | netcat 0.0.0.0 %d", out_prediction, FLAGS_gui_tx_port);
    system((const char *)write_cmd);
    printf("send prediction to GUI done\n");
    if (pipe_error)
      continue;

    // 6. update correct answer
    if (is_federated_learning) {
      training(input);
      update_model(channel);
    } else {
      update_data(channel, input);
    }
  }
}

int main(int an, char** av) {
  gflags::ParseCommandLineFlags(&an, &av, true);
  an = 1;

  if (FLAGS_operation == "") {
    printf("device.exe --print_all=true|false --operation=op --policy_host=policy-host-address --policy_port=policy-host-port\n");
    printf("\t --data_dir=-directory-for-app-data --runtime_host=runtime-host-address --runtime_model_port=runtime-model-port --runtime_data_port=runtime-data-port\n");
    printf("\t --policy_cert_file=self-signed-policy-cert-file-name --policy_store_file=policy-store-file-name\n");
    printf("\t --gui_rx_port=gui-rx-port --gui_tx_port=gui-tx-port\n");
    printf("Operations are: cold-init, get-certifier, run-shell-ml, run-shell-fl, run-gui-ml, run-gui-fl\n");
    return 0;
  }

  SSL_library_init();
  string enclave_type("simulated-enclave");
  string purpose("authentication");

  string store_file(FLAGS_data_dir);
  store_file.append(FLAGS_policy_store_file);
  app_trust_data = new cc_trust_data(enclave_type, purpose, store_file);
  if (app_trust_data == nullptr) {
    printf("couldn't initialize trust object\n");
    return 1;
  }

  // Init policy key info
  if (!app_trust_data->init_policy_key(initialized_cert_size, initialized_cert)) {
    printf("Can't init policy key\n");
    return 1;
  }

  // Init simulated enclave
  string attest_key_file_name(FLAGS_data_dir);
  attest_key_file_name.append(FLAGS_attest_key_file);
  string platform_attest_file_name(FLAGS_data_dir);
  platform_attest_file_name.append(FLAGS_platform_attest_endorsement);
  string measurement_file_name(FLAGS_data_dir);
  measurement_file_name.append(FLAGS_measurement_file);
  string attest_endorsement_file_name(FLAGS_data_dir);
  attest_endorsement_file_name.append(FLAGS_platform_attest_endorsement);

  if (!app_trust_data->initialize_simulated_enclave_data(attest_key_file_name,
      measurement_file_name, attest_endorsement_file_name)) {
    printf("Can't init simulated enclave\n");
    return 1;
  }

  // Standard algorithms for the enclave
  string public_key_alg("rsa-2048");
  string symmetric_key_alg("aes-256");
  string hash_alg("sha-256");
  string hmac_alg("sha-256-hmac");

  // Carry out operation
  int ret = 0;
  if (FLAGS_operation == "cold-init") {
    if (!app_trust_data->cold_init(public_key_alg,
        symmetric_key_alg, hash_alg, hmac_alg)) {
      printf("cold-init failed\n");
      ret = 1;
    }
  } else if (FLAGS_operation == "get-certifier") {
    if (!app_trust_data->certify_me(FLAGS_policy_host, FLAGS_policy_port)) {
      printf("certification failed\n");
      ret = 1;
    }
  } else if ((FLAGS_operation == "run-shell-ml") || (FLAGS_operation == "run-shell-fl") ||
		(FLAGS_operation == "run-gui-ml") || (FLAGS_operation == "run-gui-fl")) {
    string my_role("client");
    secure_authenticated_channel channel(my_role);
    bool is_federated_learning = false;
    bool is_gui = false;

    if (!app_trust_data->warm_restart()) {
      printf("warm-restart failed\n");
      ret = 1;
      goto done;
    }

    printf("running as client\n");
    if (!app_trust_data->cc_auth_key_initialized_ ||
        !app_trust_data->cc_policy_info_initialized_) {
      printf("trust data not initialized\n");
      ret = 1;
      goto done;
    }

    if (!channel.init_client_ssl(FLAGS_runtime_host, FLAGS_runtime_data_port,
          app_trust_data->serialized_policy_cert_,
          app_trust_data->private_auth_key_,
          app_trust_data->private_auth_key_.certificate())) {
      printf("Can't init client app\n");
      ret = 1;
      goto done;
    }

    if (FLAGS_operation == "run-shell-fl" || FLAGS_operation == "run-gui-fl")
      is_federated_learning = true;
    if (FLAGS_operation == "run-gui-ml" || FLAGS_operation == "run-gui-fl")
      is_gui = true;
    
    sprintf(ckpt_path, "./checkpoint/model_%d.ckpt", FLAGS_runtime_data_port);

    if (is_gui)
      run_gui(channel, is_federated_learning);
    else
      run_shell(channel, is_federated_learning);
  } else {
    printf("Unknown operation\n");
  }

done:
  // app_trust_data->print_trust_data();
  app_trust_data->clear_sensitive_data();
  if (app_trust_data != nullptr) {
    delete app_trust_data;
  }
  word_model.finalize();
  return ret;
}
