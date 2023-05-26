#include "tensorflow/lite/kernels/kernel_util.h"
#include "tensorflow/lite/kernels/register.h"
#include "tensorflow/lite/string_util.h"

#include <vector>

using namespace std;
using namespace tflite;

class WordPredictionModel {
 public:
  WordPredictionModel() {
    model_initialized = false;
    model_size = 0;
    model_arr = NULL;
  };
  ~WordPredictionModel() {};

  int init(unsigned char *input_model, int size);
  int finalize();

  // For ML
  int train(char *input_word, char *ckpt_path);
  int infer(char *input_word, char *ckpt_path, unsigned char *out_prediction);
  int save(char *ckpt_path);
  int restore(char *ckpt_path);

  // For FL
  int aggregate(char *input_paths[2], char *output_path);

  bool is_initialized();
  unsigned char *get();
  int get_size();

 private:
  void vectorize_x(char *input_word, vector<float> out[2]);
  void vectorize_y(char *input_word, vector<float> out[2]);
  void fill_tensor(TfLiteTensor* tensor, vector<float>& buffer);
  void fill_tensor_str(TfLiteTensor *tensor, char *str);
  void fill_tensor_multiple_str(TfLiteTensor *tensor, char **strs, int num_strs);

  unique_ptr<FlatBufferModel> model;
  unique_ptr<Interpreter> interpreter;

  bool model_initialized;
  int model_size;
  unsigned char *model_arr;

  // static variables related to the model
  static const int DICT_SIZE = 28;
  static const int LAST_IDX = DICT_SIZE - 1;
  static const int STEP_SIZE = 3;
  static const int WORD_LEN = 5;
  static const int PREDICTIONS = 2;
  static const int max_model_size = 1024 * 256;
  static const int epochs = 100;
};
