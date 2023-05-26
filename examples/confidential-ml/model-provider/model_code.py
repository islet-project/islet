import tensorflow as tf
import numpy as np
import sys

# build a vocab
x_data = list(open("code_x_data.csv", "r").readlines())
x_data = [s.strip() for s in x_data]
x_text = []
for s in x_data:
    x_text.extend(s.split(' '))
x_text = sorted(set(x_text))
print(x_text)

y_data = list(open("code_y_data.csv", "r").readlines())
y_data_str = ""
for idx, yd in enumerate(y_data):
    y_data_str += yd
y_data_split = y_data_str.split("// end")
y_data = [yd.strip() for yd in y_data_split]
ids_from_string = tf.keras.layers.StringLookup(vocabulary=list(x_text), mask_token=None)
string_from_ids = tf.keras.layers.StringLookup(vocabulary=ids_from_string.get_vocabulary(), invert=True, mask_token=None)

vocab_size = len(ids_from_string.get_vocabulary())
num_classes = len(y_data)

vocab_size_str = "int code_vocab_size = " + str(len(x_text) + 1) + ";"
vocab_str = "char code_vocab[" + str(len(x_text) + 1) + "][256] = {\"UNK\","
for x in x_text:
    vocab_str += ("\"" + x + "\",")
vocab_str += "};"

y_data_size_str = "int code_label_size = " + str(len(y_data)) + ";"
y_data_str = "char code_label[" + str(len(y_data)) + "][2048] = {\n"
for yd in y_data:
    yd = yd.replace("\n", "\\\n")
    y_data_str += ("\"" + yd + "\",\n")
y_data_str += "};"

conf_file = open("code_conf.cc", "w")
conf_file.write(vocab_size_str)
conf_file.write('\n\n')
conf_file.write(vocab_str)
conf_file.write('\n\n')
conf_file.write(y_data_size_str)
conf_file.write('\n\n')
conf_file.write(y_data_str)
conf_file.close()

# 0. a simple model for text classification
class TextClassifier(tf.Module):
    def __init__(self):
        self.model = tf.keras.Sequential([
            tf.keras.layers.SimpleRNN(units=vocab_size, input_shape=(vocab_size, vocab_size), return_sequences=False),
            tf.keras.layers.Dense(num_classes, activation='sigmoid')
        ])
        self.model.compile(
            optimizer="rmsprop",
            loss=tf.keras.losses.SparseCategoricalCrossentropy(),
            metrics=["accuracy"],
        )
        self.model.summary()
    
    @tf.function(input_signature=[
        tf.TensorSpec([1, vocab_size, vocab_size], tf.float32),
        tf.TensorSpec([1, 1], tf.float32),
    ])
    def train(self, x, y):
        with tf.GradientTape() as tape:
            prediction = self.model(x)
            loss = self.model.loss(y, prediction)
            argmax = tf.math.argmax(prediction, 1)
        gradients = tape.gradient(loss, self.model.trainable_variables)
        self.model.optimizer.apply_gradients(zip(gradients, self.model.trainable_variables))
        result = {"loss": loss, "output": argmax}
        return result

    @tf.function(input_signature=[
        tf.TensorSpec([1, vocab_size, vocab_size], tf.float32),
    ])
    def infer(self, x):
        result = self.model(x)
        argmax = tf.math.argmax(result, 1)
        return {
            "output": argmax,
        }
    
    @tf.function(input_signature=[tf.TensorSpec(shape=[], dtype=tf.string)])
    def save(self, checkpoint_path):
        tensor_names = [weight.name for weight in self.model.weights]
        tensors_to_save = [weight.read_value() for weight in self.model.weights]
        tf.raw_ops.Save(
            filename=checkpoint_path, tensor_names=tensor_names,
            data=tensors_to_save, name='save')
        return {
            "checkpoint_path": checkpoint_path
        }
    
    @tf.function(input_signature=[tf.TensorSpec(shape=[], dtype=tf.string)])
    def restore(self, checkpoint_path):
        restored_tensors = {}
        for var in self.model.weights:
            restored = tf.raw_ops.Restore(
                file_pattern=checkpoint_path, tensor_name=var.name, dt=var.dtype,
                name='restore')
            var.assign(restored)
            restored_tensors[var.name] = restored
        return restored_tensors

def text_from_ids(ids):
    res = ""
    arr = string_from_ids(ids)
    for idx, s in enumerate(arr):
        res += s
        if idx != len(arr) - 1:
            res += ' '
    return res

# 2. train
epochs = 100
def build_x_data(x):
    ids = ids_from_string(x.split(' '))
    d = []
    for i in ids:
        d.append(np.eye(vocab_size)[i])
    pad_len = vocab_size - len(ids)
    for _ in range(pad_len):
        d.append(np.eye(vocab_size)[0])
    d = np.array(d, dtype='float32')
    d = np.reshape(d, (1, vocab_size, vocab_size))
    return d

def print_code(res):
    print(y_data[int(res)])

model = TextClassifier()
for e in range(epochs):
    for idx, x in enumerate(x_data):
        d = build_x_data(x)
        y = np.array([idx])
        y = np.reshape(y, (1, -1))
        res = model.train(d, y)
    if e % 10 == 0:
        print("epoch", e, ":", res)
model.save('./checkpoint/model_code.ckpt')

# infer
d = build_x_data("write a function to copy string")
res = model.infer(d)
print_code(res['output'])

'''
for idx, x in enumerate(x_data):
    d = build_x_data(x)
    res = model.infer(d)
    # print code
    print(y_data[int(res['output'])])
'''

#### Converting it to tflite ################
# store functions
saved_dir = "./checkpoint"
tf.saved_model.save(
    model,
    saved_dir,
    signatures={
        'train':
            model.train.get_concrete_function(),
        'infer':
            model.infer.get_concrete_function(),
        'save':
            model.save.get_concrete_function(),
        'restore':
            model.restore.get_concrete_function(),
    })

# convert
converter = tf.lite.TFLiteConverter.from_saved_model(saved_dir)
converter.target_spec.supported_ops = [
    tf.lite.OpsSet.TFLITE_BUILTINS,  # enable TensorFlow Lite ops.
    tf.lite.OpsSet.SELECT_TF_OPS    # enable TensorFlow ops.
]
converter.experimental_enable_resource_variables = True
tflite_model = converter.convert()
open("model_code.tflite", "wb").write(tflite_model)
#############################################

#### Inference using tflite signature! ################
interpreter = tf.lite.Interpreter(model_content=tflite_model)
interpreter.allocate_tensors()
infer = interpreter.get_signature_runner("infer")

result = infer(x=d)['output']
print_code(result)
#######################################################