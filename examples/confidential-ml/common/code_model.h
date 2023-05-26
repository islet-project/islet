#include "tensorflow/lite/kernels/kernel_util.h"
#include "tensorflow/lite/kernels/register.h"
#include "tensorflow/lite/string_util.h"

#include <vector>

using namespace std;
using namespace tflite;

class CodeModel {
 public:
  CodeModel() {
    model_initialized = false;
    model_size = 0;
    model_arr = NULL;
  };
  ~CodeModel() {};

  int init(unsigned char *input_model, int size);
  int finalize();

  // For ML
  int infer(char *input_str, unsigned char *out_prediction);

  bool is_initialized();
  unsigned char *get();
  int get_size();

 private:
  void ids_from_str(char *input, vector<float>& out);
  void ids_to_str(int id, unsigned char *out_prediction);
  void fill_tensor(TfLiteTensor* tensor, vector<float>& buffer);

  unique_ptr<FlatBufferModel> model;
  unique_ptr<Interpreter> interpreter;

  bool model_initialized;
  int model_size;
  unsigned char *model_arr;

  static const int max_model_size = 1024 * 256;
};
