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
#include <string.h>
#include <errno.h>

// operations are: cold-init, get-certifier, send-model
DEFINE_bool(print_all, false,  "verbose");
DEFINE_string(operation, "", "operation");

DEFINE_string(policy_host, "localhost", "address for policy server");
DEFINE_int32(policy_port, 8123, "port for policy server");
DEFINE_string(data_dir, "./data/", "directory for application data");

DEFINE_string(runtime_host, "localhost", "address for runtime");
DEFINE_int32(runtime_model_port, 8124, "port for runtime (used to deliver model)");
DEFINE_int32(runtime_data_port, 8125, "port for runtime (used to deliver data)");

DEFINE_string(policy_store_file, "store.bin", "policy store file name");
DEFINE_string(platform_file_name, "platform_file.bin", "platform certificate");
DEFINE_string(platform_attest_endorsement, "platform_attest_endorsement.bin", "platform endorsement of attest key");
DEFINE_string(attest_key_file, "attest_key_file.bin", "attest key");
DEFINE_string(measurement_file, "model_provider.measurement", "measurement");
DEFINE_string(model_file, "model.tflite", "model file");

// model_provider performs three possible roles
//    cold-init: This creates application keys and initializes the policy store.
//    get-certifier: This obtains the app admission cert naming the public app key from the service.
//    send-model: This sends ML model to Runtime which is capable of doing ML operations and passing the model to devices.

#include "../certifier-data/policy_key.cc"
cc_trust_data* app_trust_data = nullptr;
const int max_model_size = 1024 * 256;  // 256k

// -----------------------------------------------------------------------------------------

void send_model(secure_authenticated_channel& channel, string &model_file_name) {
  printf("Client peer id is %s\n", channel.peer_id_.c_str());
  if (channel.peer_cert_ != nullptr) {
    printf("Client peer cert is:\n");
#ifdef DEBUG
    X509_print_fp(stdout, channel.peer_cert_);
#endif
  }

  unsigned char model[max_model_size] = {0,};
  FILE *ptr;

  ptr = fopen(model_file_name.c_str(), "rb");
  if (ptr == NULL) {
    printf("file open error: %s, %s\n", model_file_name.c_str(), strerror(errno));
    return;
  }

  size_t len = fread(model, 1, sizeof(model), ptr);
  printf("model read done, size: %d\n", len);
  if (len == 0) {
    printf("model read fail, %s\n", model_file_name.c_str());
    return;
  }

  int n = channel.write(len, (byte*)model);
  printf("send-model done, size: %d\n", n);

  string received;
  n = channel.read(&received);
  if (n == sizeof(n)) {
    printf("ACK: %d\n", *((int *)received.data()));
  }
}

int main(int an, char** av) {
  gflags::ParseCommandLineFlags(&an, &av, true);
  an = 1;

  if (FLAGS_operation == "") {
    printf("model_provider.exe --print_all=true|false --operation=op --policy_host=policy-host-address --policy_port=policy-host-port\n");
    printf("\t --data_dir=-directory-for-app-data --runtime_host=runtime-host-address --runtime_model_port=runtime-model-port --runtime_data_port=runtime-data-port\n");
    printf("\t --policy_cert_file=self-signed-policy-cert-file-name --policy_store_file=policy-store-file-name --model_file=model-file-name\n");
    printf("Operations are: cold-init, get-certifier, send-model\n");
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
  } else if (FLAGS_operation == "send-model") {
    string model_file_name(FLAGS_model_file);
    string my_role("client");
    secure_authenticated_channel channel(my_role);

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

    if (!channel.init_client_ssl(FLAGS_runtime_host, FLAGS_runtime_model_port,
          app_trust_data->serialized_policy_cert_,
          app_trust_data->private_auth_key_,
          app_trust_data->private_auth_key_.certificate())) {
      printf("Can't init client app\n");
      ret = 1;
      goto done;
    }

    send_model(channel, model_file_name);
  } else {
    printf("Unknown operation\n");
  }

done:
  // app_trust_data->print_trust_data();
  app_trust_data->clear_sensitive_data();
  if (app_trust_data != nullptr) {
    delete app_trust_data;
  }
  return ret;
}
