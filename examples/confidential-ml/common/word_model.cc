#include "word_model.h"

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

int WordPredictionModel::init(unsigned char *input_model, int size) {
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

int WordPredictionModel::finalize() {
  if (model_arr) {
    delete model_arr;
  }
  return 0;
}

int WordPredictionModel::save(char *ckpt_path) {
  auto save_runner = interpreter->GetSignatureRunner("save");

  if (save_runner->AllocateTensors() != kTfLiteOk) {
    printf("*** Failed to allocate tensors! ***\n");
    return -1;
  }
  auto path = save_runner->input_tensor("checkpoint_path");
  fill_tensor_str(path, ckpt_path);

  if (save_runner->Invoke() != kTfLiteOk) {
    printf("*** Failed to invoke tflite! ***\n");
    return -1;
  }
  return 0;
}

int WordPredictionModel::restore(char *ckpt_path) {
  auto restore_runner = interpreter->GetSignatureRunner("restore");

  if (restore_runner->AllocateTensors() != kTfLiteOk) {
    printf("*** Failed to allocate tensors! ***\n");
    return -1;
  }
  auto path = restore_runner->input_tensor("checkpoint_path");
  fill_tensor_str(path, ckpt_path);

  if (restore_runner->Invoke() != kTfLiteOk) {
    printf("*** Failed to invoke tflite! ***\n");
    return -1;
  }
  return 0;
}

int WordPredictionModel::infer(char *input_word, char *ckpt_path, unsigned char *out_prediction) {
  if (!model_initialized) {
    printf("model not initialized\n");
    return -1;
  }

  // restore first if ckpt exists
  if (access(ckpt_path, F_OK) != -1) {
    restore(ckpt_path);
  }

  // do inference
  // prepare data
  vector<float> input_x[PREDICTIONS];
  vectorize_x(input_word, input_x);

  // signature runner
  auto infer_runner = interpreter->GetSignatureRunner("infer");
  if (infer_runner->AllocateTensors() != kTfLiteOk) {
    printf("*** Failed to allocate tensors! ***\n");
    return -1;
  }

  // run inference
  auto input = infer_runner->input_tensor("x");
  char out_str[DICT_SIZE+1] = {0,};
  for (int i=0; i<PREDICTIONS; i++) {
    printf("tensor bytes: %d, vector len: %d\n", input->bytes, input_x[i].size());
    fill_tensor(input, input_x[i]);

    if (infer_runner->Invoke() != kTfLiteOk) {
      printf("*** Failed to invoke tflite! ***\n");
      return -1;
    }

    // output
    auto output = infer_runner->output_tensor("output");
    TfLiteIntArray* dims = output->dims;
    int64_t* out = output->data.i64;

    if (dims->data[0] != 1) { // output size
      printf("output size wrong: %d\n", dims->data[0]);
      return -1;
    }

    // build output
    int d = (int)out[0];
    out_str[i+STEP_SIZE] = (char)('a' + d);
  }

  for (int i=0; i<STEP_SIZE; i++) {
    out_str[i] = input_word[i];
  }

  // print output
  printf("prediction: %s\n", out_str);
  memcpy(out_prediction, out_str, DICT_SIZE);
  return 0;
}

bool WordPredictionModel::is_initialized() {
  return model_initialized;
}

unsigned char* WordPredictionModel::get() {
  return model_arr;
}

int WordPredictionModel::get_size() {
  return model_size;
}

int WordPredictionModel::train(char *input_word, char *ckpt_path) {
  if (!model_initialized) {
    printf("model not initialized\n");
    return -1;
  }

  // restore first if ckpt exists
  if (access(ckpt_path, F_OK) != -1) {
    restore(ckpt_path);
  }

  // prepare data
  vector<float> input_x[PREDICTIONS];
  vector<float> input_y[PREDICTIONS];
  vectorize_x(input_word, input_x);
  vectorize_y(input_word, input_y);

  // signature runner
  auto train_runner = interpreter->GetSignatureRunner("train");
  if (train_runner->AllocateTensors() != kTfLiteOk) {
    printf("*** Failed to allocate tensors! ***\n");
    return -1;
  }

  // run training
  auto tensor_x = train_runner->input_tensor("x");
  auto tensor_y = train_runner->input_tensor("y");

  for (int e=0; e<epochs; e++) {
    float losses[PREDICTIONS] = {0.0f,};

    for (int i=0; i<PREDICTIONS; i++) {
      fill_tensor(tensor_x, input_x[i]);
      fill_tensor(tensor_y, input_y[i]);

      if (train_runner->Invoke() != kTfLiteOk) {
        printf("*** Failed to invoke tflite! ***\n");
        return -1;
      }

      // output
      auto loss = train_runner->output_tensor("loss");
      TfLiteIntArray* dims = loss->dims;
      float* out = loss->data.f;
      losses[i] = out[0];
    }

    if (e % 10 == 0) {
      printf("epoch: %d, loss: %.3f,%.3f\n", e, losses[0], losses[1]);
    }
  }

  // save checkpoint for serving
  save(ckpt_path);
  return 0;
}

int WordPredictionModel::aggregate(char *input_paths[2], char *output_path) {
  if (!model_initialized) {
    printf("model not initialized\n");
    return -1;
  }

  // check if ckpt exists
  for (int i=0; i<2; i++) {
    if (access(input_paths[i], F_OK) == -1) {
      printf("file not exist: %s\n", input_paths[i]);
      return -1;
    }
  }

  // signature runner
  auto aggregate_runner = interpreter->GetSignatureRunner("aggregate");
  if (aggregate_runner->AllocateTensors() != kTfLiteOk) {
    printf("*** Failed to allocate tensors! ***\n");
    return -1;
  }

  auto paths = aggregate_runner->input_tensor("input_paths");
  auto out_path = aggregate_runner->input_tensor("output_path");
  fill_tensor_multiple_str(paths, input_paths, 2);
  fill_tensor_str(out_path, output_path);

  if (aggregate_runner->Invoke() != kTfLiteOk) {
    printf("*** Failed to invoke tflite! ***\n");
    return -1;
  }
  return 0;
}

void WordPredictionModel::fill_tensor(TfLiteTensor* tensor, vector<float>& buffer) {
  memcpy(tensor->data.f, buffer.data(), tensor->bytes);
}

void WordPredictionModel::fill_tensor_str(TfLiteTensor *tensor, char *str) {
  DynamicBuffer buf;
  buf.AddString(str, strlen(str));
  buf.WriteToTensorAsVector(tensor);
}

void WordPredictionModel::fill_tensor_multiple_str(TfLiteTensor *tensor, char **strs, int num_strs) {
  DynamicBuffer buf;
  for (int i=0; i<num_strs; i++)
    buf.AddString(strs[i], strlen(strs[i]));
  buf.WriteToTensorAsVector(tensor);
}

void WordPredictionModel::vectorize_x(char *input_word, vector<float> out[2]) {
  for (int i=0; i<PREDICTIONS; i++) {
    // sequence handling
    for (int j=i; j<i+STEP_SIZE; j++) {
      int val = (int)(input_word[j] - 'a');
      for (int k=0; k<DICT_SIZE; k++) { // one-hot encoded
        if (val == k)
          out[i].push_back(1.0f);
        else
          out[i].push_back(0.0f);
      }
    }
  }
}

void WordPredictionModel::vectorize_y(char *input_word, vector<float> out[2]) {
  for (int i=STEP_SIZE, j=0; i<STEP_SIZE+PREDICTIONS; i++, j++) {
    int val = (int)(input_word[i] - 'a');
    for (int k=0; k<DICT_SIZE; k++) { // one-hot encoded
      if (val == k)
        out[j].push_back(1.0f);
      else
        out[j].push_back(0.0f);
    }
  }
}