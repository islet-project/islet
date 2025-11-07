#    
#    File: build.mak
#
#    Usage: make -f build.mak CC=g++ EXE=device INC_PATH=/path/to/include/ INC_PATH2=/include/ LIB_PATH=/path/to/lib/ CERTIFIER=/path/to/certifier TENSOR_FLOW=ON

SRC_DIR=$(CERTIFIER)
OBJ_DIR=.
EXE_DIR=.
LOCAL_LIB=$(LIB_PATH)

S= $(SRC_DIR)/src
O= $(OBJ_DIR)
US=.
I= $(SRC_DIR)/include
INCLUDE=-I$(INC_PATH) -I$(I) -I$(INC_PATH2)

CFLAGS=$(INCLUDE) -O3 -g -Wall -Wno-unused-variable -D X64 -Wno-deprecated-declarations
CFLAGS_ERROR=$(CFLAGS) -Werror
CFLAGS1=$(INCLUDE) -O1 -g -Wall -Wno-unused-variable -D X64 -Wno-deprecated-declarations
CC=$(CC)
LINK=$(CC)
PROTO=protoc
AR=ar
CP=$(CERTIFIER)/certifier_service/certprotos

LDFLAGS= -L$(LOCAL_LIB) -lprotobuf -lpthread -lgflags -lcrypto -lssl
ifeq ($(TENSOR_FLOW),ON)
	LDFLAGS= -Wl,--no-as-needed -L$(LOCAL_LIB) -lprotobuf -lpthread -lgflags -lcrypto -lssl -ltensorflowlite -ltensorflowlite_flex
endif

dobj=	$(O)/$(EXE).o $(O)/certifier.pb.o $(O)/certifier.o $(O)/certifier_proofs.o \
$(O)/support.o $(O)/simulated_enclave.o $(O)/application_enclave.o $(O)/cc_helpers.o $(O)/cc_useful.o

ifeq ($(TENSOR_FLOW),ON)
	dobj=	$(O)/$(EXE).o $(O)/certifier.pb.o $(O)/certifier.o $(O)/certifier_proofs.o \
	$(O)/support.o $(O)/simulated_enclave.o $(O)/application_enclave.o $(O)/cc_helpers.o $(O)/cc_useful.o $(O)/word_model.o $(O)/code_model.o
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

$(I)/certifier.pb.h: $(US)/certifier.pb.cc
	@echo "compiling certifier.pb.h"

$(US)/certifier.pb.cc: $(CP)/certifier.proto
	@echo "compiling certifer.proto"
	$(PROTO) --proto_path=$(CP) --cpp_out=$(US) $<
	cp -f $(US)/certifier.pb.h $(I)

$(O)/certifier.pb.o: $(US)/certifier.pb.cc $(I)/certifier.pb.h
	@echo "compiling certifier.pb.cc"
	$(CC) $(CFLAGS_ERROR) -c -o $(O)/certifier.pb.o $(US)/certifier.pb.cc

$(O)/$(EXE).o: $(US)/$(EXE).cc $(I)/certifier.h $(I)/certifier_framework.h $(US)/certifier.pb.cc
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

$(O)/cc_useful.o: $(S)/cc_useful.cc $(I)/certifier.h $(US)/certifier.pb.cc
	@echo "compiling cc_useful.cc"
	$(CC) $(CFLAGS) -c -o $(O)/cc_useful.o $(S)/cc_useful.cc

$(O)/word_model.o: ../common/word_model.cc ../common/word_model.h
	@echo "compiling word_model.cc"
	$(CC) $(CFLAGS) -c -o $(O)/word_model.o ../common/word_model.cc

$(O)/code_model.o: ../common/code_model.cc ../common/code_model.h
	@echo "compiling code_model.cc"
	$(CC) $(CFLAGS) -c -o $(O)/code_model.o ../common/code_model.cc
