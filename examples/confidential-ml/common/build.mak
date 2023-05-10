#    
#    File: build.mak
#
#    Usage: make -f build.mak CC=g++ EXE=device INC_PATH=/path/to/include/ LIB_PATH=/path/to/lib/ TENSOR_FLOW=ON

SRC_DIR=../../../third-party/certifier
OBJ_DIR=.
EXE_DIR=.
LOCAL_LIB=$(LIB_PATH)

S= $(SRC_DIR)/src
O= $(OBJ_DIR)
US=.
I= $(SRC_DIR)/include
INCLUDE=-I$(INC_PATH) -I$(I)

CFLAGS=$(INCLUDE) -O3 -g -Wall -Wno-unused-variable -D X64 -Wno-deprecated-declarations
CFLAGS1=$(INCLUDE) -O1 -g -Wall -Wno-unused-variable -D X64 -Wno-deprecated-declarations
CC=$(CC)
LINK=$(CC)
PROTO=protoc
AR=ar

LDFLAGS= -L$(LOCAL_LIB) -lpthread -lgflags -lcrypto -lssl -lprotobuf
ifeq ($(TENSOR_FLOW),ON)
	LDFLAGS= -Wl,--no-as-needed -L$(LOCAL_LIB) -lpthread -lgflags -lcrypto -lssl -lprotobuf -ltensorflowlite_flex -ltensorflowlite
endif

dobj=	$(O)/$(EXE).o $(O)/certifier.pb.o $(O)/certifier.o $(O)/certifier_proofs.o \
$(O)/support.o $(O)/simulated_enclave.o $(O)/application_enclave.o $(O)/cc_helpers.o

ifeq ($(TENSOR_FLOW),ON)
	dobj=	$(O)/$(EXE).o $(O)/certifier.pb.o $(O)/certifier.o $(O)/certifier_proofs.o \
	$(O)/support.o $(O)/simulated_enclave.o $(O)/application_enclave.o $(O)/cc_helpers.o $(O)/word_model.o
endif

all:	$(EXE).exe
clean:
	@echo "removing object files"
	rm $(O)/*.o
	@echo "removing executable file"
	rm $(EXE_DIR)/$(EXE).exe

$(EXE).exe: $(dobj) 
	@echo "linking executable files"
	$(LINK) -o $(EXE_DIR)/$(EXE).exe $(dobj) $(LDFLAGS)

$(US)/certifier.pb.cc: $(S)/certifier.proto
	$(PROTO) --proto_path=$(S) --cpp_out=$(US) $(S)/certifier.proto
	mv $(US)/certifier.pb.h $(I)

$(I)/certifier.pb.h: $(US)/certifier.pb.cc

$(O)/certifier.pb.o: $(US)/certifier.pb.cc $(I)/certifier.pb.h
	@echo "compiling certifier.pb.cc"
	$(CC) $(CFLAGS) -c -o $(O)/certifier.pb.o $(US)/certifier.pb.cc

$(O)/$(EXE).o: $(US)/$(EXE).cc $(I)/certifier.h $(US)/certifier.pb.cc
	@echo "compiling $(EXE).cc"
	$(CC) $(CFLAGS) -c -o $(O)/$(EXE).o $(US)/$(EXE).cc

$(O)/certifier.o: $(S)/certifier.cc $(I)/certifier.pb.h $(I)/certifier.h
	@echo "compiling certifier.cc"
	$(CC) $(CFLAGS) -c -o $(O)/certifier.o $(S)/certifier.cc

$(O)/certifier_proofs.o: $(S)/certifier_proofs.cc $(I)/certifier.pb.h $(I)/certifier.h
	@echo "compiling certifier_proofs.cc"
	$(CC) $(CFLAGS) -c -o $(O)/certifier_proofs.o $(S)/certifier_proofs.cc

$(O)/support.o: $(S)/support.cc $(I)/support.h
	@echo "compiling support.cc"
	$(CC) $(CFLAGS) -c -o $(O)/support.o $(S)/support.cc

$(O)/simulated_enclave.o: $(S)/simulated_enclave.cc $(I)/simulated_enclave.h
	@echo "compiling simulated_enclave.cc"
	$(CC) $(CFLAGS) -c -o $(O)/simulated_enclave.o $(S)/simulated_enclave.cc

$(O)/application_enclave.o: $(S)/application_enclave.cc $(I)/application_enclave.h
	@echo "compiling application_enclave.cc"
	$(CC) $(CFLAGS) -c -o $(O)/application_enclave.o $(S)/application_enclave.cc

$(O)/cc_helpers.o: $(S)/cc_helpers.cc $(I)/certifier.h $(US)/certifier.pb.cc
	@echo "compiling cc_helpers.cc"
	$(CC) $(CFLAGS) -c -o $(O)/cc_helpers.o $(S)/cc_helpers.cc

$(O)/word_model.o: ../common/word_model.cc ../common/word_model.h
	@echo "compiling word_model.cc"
	$(CC) $(CFLAGS) -c -o $(O)/word_model.o ../common/word_model.cc
