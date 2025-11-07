#!/bin/bash

set -e

ROOT=$(git rev-parse --show-toplevel)
HERE=$ROOT/examples/confidential-ml

sudo apt-get install -y cmake libxnnpack0 libxnnpack-dev
curl -Lo bazelisk https://github.com/bazelbuild/bazelisk/releases/latest/download/bazelisk-linux-amd64
chmod +x bazelisk
sudo mv bazelisk /usr/local/bin/bazel
bazel version

cd $HERE && git clone https://github.com/tensorflow/tensorflow.git tensorflow_src
cd tensorflow_src
git checkout v2.15.0

# tflite build
bazel build -c opt --define tflite_with_flex=true //tensorflow/lite:libtensorflowlite.so
bazel build -c opt --config=monolithic tensorflow/lite/delegates/flex:tensorflowlite_flex

cd ../
mkdir tflite_libs/
cd tflite_libs/
ln -s $HERE/tensorflow_src/bazel-bin/tensorflow/lite/libtensorflowlite.so libtensorflowlite.so
ln -s $HERE/tensorflow_src/bazel-bin/tensorflow/lite/delegates/flex/libtensorflowlite_flex.so libtensorflowlite_flex.so

