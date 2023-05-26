#include "code_model.h"

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "code_conf.cc"

int CodeModel::init(unsigned char *input_model, int size) {
  model_arr = new unsigned char[max_model_size];
  memcpy(model_arr, input_model, size);

  model = tflite::FlatBufferModel::BuildFromBuffer((const char *)model_arr, model_size);
  if (!model) {
    printf("*** Failed to mmap model ***\n");
    return -1;
  }

  tflite::ops::builtin::BuiltinOpResolver resolver;
  tflite::InterpreterBuilder(*model, resolver)(&interpreter);
  if (!interpreter) {
    printf("*** Failed to construct interpreter! ***\n");
    return -1;
  }

  model_initialized = true;
  model_size = size;
  return 0;
}

int CodeModel::finalize() {
  if (model_arr) {
    delete model_arr;
  }
  return 0;
}

void CodeModel::ids_from_str(char *input, vector<float>& out) {
  char *token;
  const char s[2] = " ";
  vector<float> ids;

  token = strtok(input, s);
  while (token != NULL) {
    bool found = false;
    for (int i=0; i<code_vocab_size; i++) {
      if (strcmp(token, code_vocab[i]) == 0) {
        ids.push_back((float)i);
        found = true;
        break;
      }
    }
    if (!found)
      ids.push_back(0.0f);
    token = strtok(NULL, s);
  }
  printf("msg: %s\n", input);
  printf("ids size: %d\n", ids.size());

  // padding
  int padding = code_vocab_size - ids.size();
  for (int i=0; i<padding; i++) {
    ids.push_back(0.0f);
  }

  // one-hot conversion
  for (int i=0; i<ids.size(); i++) {
    for (int j=0; j<code_vocab_size; j++) {
      if (ids[i] == j)
        out.push_back(1.0f);
      else
        out.push_back(0.0f);
    }
  }
  printf("out size: %d\n", out.size());
}

void CodeModel::ids_to_str(int id, unsigned char *out_prediction) {
  if (id >= code_label_size) {
    strcpy((char *)out_prediction, "wrong prediction result");
  } else {
    strcpy((char *)out_prediction, code_label[id]);
  }
}

int CodeModel::infer(char *input_str, unsigned char *out_prediction) {
  if (!model_initialized) {
    printf("model not initialized\n");
    return -1;
  }

  // do inference
  // prepare data
  vector<float> input_x;
  ids_from_str(input_str, input_x);

  // signature runner
  auto infer_runner = interpreter->GetSignatureRunner("infer");
  if (infer_runner->AllocateTensors() != kTfLiteOk) {
    printf("*** Failed to allocate tensors! ***\n");
    return -1;
  }

  // run inference
  auto input = infer_runner->input_tensor("x");
  //char out_str[2048] = {0,};

  fill_tensor(input, input_x);
  if (infer_runner->Invoke() != kTfLiteOk) {
    printf("*** Failed to invoke tflite! ***\n");
    return -1;
  }

  auto output = infer_runner->output_tensor("output");
  int64_t* out = output->data.i64;
  int answer = (int)out[0];
  ids_to_str(answer, out_prediction);
  return 0;
}

bool CodeModel::is_initialized() {
  return model_initialized;
}

unsigned char* CodeModel::get() {
  return model_arr;
}

int CodeModel::get_size() {
  return model_size;
}

void CodeModel::fill_tensor(TfLiteTensor* tensor, vector<float>& buffer) {
  memcpy(tensor->data.f, buffer.data(), tensor->bytes);
}