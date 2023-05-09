cd ..
cargo build
cd -
g++ main.cpp -lislet_sdk -L/data/islet/out/x86_64-unknown-linux-gnu/debug/
LD_LIBRARY_PATH=/data/islet/out/x86_64-unknown-linux-gnu/debug/ ./a.out
