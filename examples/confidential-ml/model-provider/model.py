import numpy as np
import tensorflow as tf
import sys

#### Custom model for on-device tflite training, considering model serving ######
char_rdic = list('abcdefghijklmnopqrstuvwxyz- ')  # alphabet letters + ' ' (' ' to represent an empty space)
char_dic = {w: i for i, w in enumerate(char_rdic)}
dic_size = len(char_dic)
word_len = 5
step_size = 3
predictions = word_len - step_size

class Model(tf.Module):
    def __init__(self):
        self.model = tf.keras.Sequential([
            tf.keras.layers.SimpleRNN(units=dic_size, input_shape=(step_size, dic_size), return_sequences=False)
        ])
        self.model.compile(
            optimizer="rmsprop",
            loss=tf.keras.losses.MeanSquaredError(),
            metrics=["accuracy"],
        )

    @tf.function(input_signature=[
        tf.TensorSpec([1, step_size, dic_size], tf.float32),
        tf.TensorSpec([1, dic_size], tf.float32),
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
        tf.TensorSpec([1, step_size, dic_size], tf.float32),
    ])
    def infer(self, x):
        result = self.model(x)
        argmax = tf.math.argmax(result, 1)
        return {
            "output": argmax,  # anything needed further? maybe not
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

    @tf.function(input_signature=[
        tf.TensorSpec(shape=[2], dtype=tf.string),
        tf.TensorSpec(shape=[], dtype=tf.string),
    ])
    def aggregate(self, input_paths, output_path):
        # aggregation function for federated learning
        # aggregation algorithm: FedAvg
        # hard-coded ones: it assumes 2 participants at most as of now.
        #
        # 1. it will be given a number of checkpoint_path, each of them represents parameters of a device
        #    the string "NULL" indicates there is no other string following.
        # 2. read restored weights
        # 3. do FedAvg on multiple weights --> [Q] are all operations needed to do this compatible to tf.function?
        # 4. save a global model as a result of FedAvg.
        restored_tensors = {}
        tensor_names = []
        tensors_to_save = []

        for var in self.model.weights:
            restored = tf.raw_ops.Restore(
                file_pattern=input_paths[0], tensor_name=var.name, dt=var.dtype, name='restore')
            restored_tensors[var.name] = restored
        for var in self.model.weights:
            restored = tf.raw_ops.Restore(
                file_pattern=input_paths[1], tensor_name=var.name, dt=var.dtype, name='restore')
            restored_tensors[var.name] = tf.math.add(restored_tensors[var.name], restored)
        for name, value in restored_tensors.items():
            tensor_names.append(name)
            tensors_to_save.append(tf.math.divide(value, 2))
        tf.raw_ops.Save(
            filename=output_path, tensor_names=tensor_names, data=tensors_to_save, name='save')
        return output_path
##################################################################################

#### Training a custom model in python ####
# prepare data
words = []
x_data = []
y_data = []
with open('wordlist_short.txt') as f:
    for line in f:
        words.append(line.strip())
for word in words:  
    for i in range(predictions):
        d = []
        for c in range(step_size):
            idx = char_dic[word[i+c]]
            d.append(np.eye(dic_size)[idx])
            #d.append(idx)
        x_data.append(np.array(d, dtype='float32'))
        y_val = char_dic[word[i+step_size]]
        y_data.append(np.eye(dic_size)[y_val])

x_data = np.array(x_data, dtype='float32')
y_data = np.array(y_data, dtype='float32')
print(x_data.shape)
print(y_data.shape)

# training
epochs = 5
batch_size = 1
losses = [0 for _ in range(epochs)]
m = Model()

train_ds = tf.data.Dataset.from_tensor_slices((x_data, y_data))
train_ds = train_ds.batch(batch_size)

for i in range(epochs):
    for x,y in train_ds:
        result = m.train(x, y)

    losses[i] = result['loss']
    if (i + 1) % 10 == 0:
        print(f"Finished {i+1} epochs")
        print(f"  loss: {losses[i]:.3f}")

# save weights
m.save('./checkpoint/model.ckpt')
###########################################

#### Testing restore/inference in python ####
# prepare data
input_word = "about"
input_x = []
for i in range(predictions):
    x = []
    substr = input_word[i:i+step_size]
    for c in substr:
        idx = char_dic[c]
        x.append(np.eye(dic_size)[idx])
    input_x.append(x)
input_x = np.array(input_x)

# inference test
test_result = []
for i in range(predictions):
    x = np.reshape(input_x[i], (1, step_size, dic_size))
    result = m.infer(x)['output']
    for t in result:
        test_result.append(char_rdic[t])
print(test_result)
#############################################

#### Converting it to tflite ################
# store functions
saved_dir = "./checkpoint"
tf.saved_model.save(
    m,
    saved_dir,
    signatures={
        'train':
            m.train.get_concrete_function(),
        'infer':
            m.infer.get_concrete_function(),
        'save':
            m.save.get_concrete_function(),
        'restore':
            m.restore.get_concrete_function(),
        'aggregate':
            m.aggregate.get_concrete_function(),
    })

# convert
converter = tf.lite.TFLiteConverter.from_saved_model(saved_dir)
converter.target_spec.supported_ops = [
    tf.lite.OpsSet.TFLITE_BUILTINS,  # enable TensorFlow Lite ops.
    tf.lite.OpsSet.SELECT_TF_OPS    # enable TensorFlow ops.
]
converter.experimental_enable_resource_variables = True
tflite_model = converter.convert()
open("model.tflite", "wb").write(tflite_model)
#############################################

#### Inference using tflite signature! ################
interpreter = tf.lite.Interpreter(model_content=tflite_model)
interpreter.allocate_tensors()
infer = interpreter.get_signature_runner("infer")

test_result = []
for i in range(predictions):
    x_data = np.reshape(input_x[i], (1, step_size, dic_size))
    x_data = np.array(x_data, dtype='float32')
    print(x_data.shape)

    result = infer(x=x_data)['output']
    for t in result:
        test_result.append(char_rdic[t])
print(test_result)
#######################################################

##### save and restore using tflite interpreter #####
# train using tflite
train = interpreter.get_signature_runner("train")
epochs = 100
losses = [0 for _ in range(epochs)]

for i in range(epochs):
    for x,y in train_ds:
        result = train(x=x, y=y)
    losses[i] = result['loss']
    if (i + 1) % 10 == 0:
        print(f"  loss: {losses[i]:.3f}")

# infer again
test_result = []
for i in range(predictions):
    x_data = np.reshape(input_x[i], (1, step_size, dic_size))
    x_data = np.array(x_data, dtype='float32')
    result = infer(x=x_data)['output']
    for t in result:
        test_result.append(char_rdic[t])
print(test_result)

# save
save = interpreter.get_signature_runner("save")
save(checkpoint_path=np.array("./checkpoint/model.ckpt", dtype=np.string_))

# restore
interpreter2 = tf.lite.Interpreter(model_content=tflite_model)
interpreter2.allocate_tensors()

infer = interpreter2.get_signature_runner("infer")
test_result = []
for i in range(predictions):
    x_data = np.reshape(input_x[i], (1, step_size, dic_size))
    x_data = np.array(x_data, dtype='float32')
    result = infer(x=x_data)['output']
    for t in result:
        test_result.append(char_rdic[t])
print(test_result)

restore = interpreter2.get_signature_runner("restore")
restore(checkpoint_path=np.array("./checkpoint/model.ckpt", dtype=np.string_))

test_result = []
for i in range(predictions):
    x_data = np.reshape(input_x[i], (1, step_size, dic_size))
    x_data = np.array(x_data, dtype='float32')
    result = infer(x=x_data)['output']
    for t in result:
        test_result.append(char_rdic[t])
print(test_result)
#####################################################

################ FL aggregation test ################
aggregate = interpreter2.get_signature_runner("aggregate")
paths = []
paths.append(np.array("./checkpoint/model.ckpt", dtype=np.string_))
paths.append(np.array("./checkpoint/model.ckpt", dtype=np.string_))

aggr_res = aggregate(input_paths=np.array(paths), output_path=np.array("./checkpoint/aggr_model.ckpt", dtype=np.string_))
print("aggregated model: ", aggr_res)
#####################################################
