#
# Copyright (C) 2011-2016 Intel Corporation. All rights reserved.
#
# Redistribution and use in source and binary forms, with or without
# modification, are permitted provided that the following conditions
# are met:
#
#   * Redistributions of source code must retain the above copyright
#     notice, this list of conditions and the following disclaimer.
#   * Redistributions in binary form must reproduce the above copyright
#     notice, this list of conditions and the following disclaimer in
#     the documentation and/or other materials provided with the
#     distribution.
#   * Neither the name of Intel Corporation nor the names of its
#     contributors may be used to endorse or promote products derived
#     from this software without specific prior written permission.
#
# THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS
# "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT
# LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR
# A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT
# OWNER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
# SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT
# LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE,
# DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY
# THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
# (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
# OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
#
#

#
# This file was adapted for the IDASH 2017 competition
# by T.Tortech & S.Carpov (CEA, LIST)
#

######## SGX SDK Settings ########

SGX_SDK ?= /opt/intel/sgxsdk
SGX_MODE ?= HW
SGX_ARCH ?= x64

ifeq ($(shell getconf LONG_BIT), 32)
	SGX_ARCH := x86
else ifeq ($(findstring -m32, $(CXXFLAGS)), -m32)
	SGX_ARCH := x86
endif

ifeq ($(SGX_ARCH), x86)
	SGX_COMMON_CFLAGS := -m32
	SGX_LIBRARY_PATH := $(SGX_SDK)/lib
	SGX_ENCLAVE_SIGNER := $(SGX_SDK)/bin/x86/sgx_sign
	SGX_EDGER8R := $(SGX_SDK)/bin/x86/sgx_edger8r
else
	SGX_COMMON_CFLAGS := -m64
	SGX_LIBRARY_PATH := $(SGX_SDK)/lib64
	SGX_ENCLAVE_SIGNER := $(SGX_SDK)/bin/x64/sgx_sign
	SGX_EDGER8R := $(SGX_SDK)/bin/x64/sgx_edger8r
endif

ifeq ($(SGX_DEBUG), 1)
ifeq ($(SGX_PRERELEASE), 1)
$(error Cannot set SGX_DEBUG and SGX_PRERELEASE at the same time!!)
endif
endif


ifeq ($(SGX_DEBUG), 1)
	SGX_COMMON_CFLAGS += -O0 -g
else
	SGX_COMMON_CFLAGS += -O2
endif

######## CUSTOM Settings ########

CUSTOM_BIN_PATH := ./bin

######## EDL Settings ########

Enclave_EDL_Files := src/enclave/Enclave_t.c src/enclave/Enclave_t.h src/app/Enclave_u.c src/app/Enclave_u.h

######## APP Settings ########

ifneq ($(SGX_MODE), HW)
	Urts_Library_Name := sgx_urts_sim
else
	Urts_Library_Name := sgx_urts
endif

ifneq ($(SGX_MODE), HW)
	App_Link_Flags += -lsgx_uae_service_sim
else
	App_Link_Flags += -lsgx_uae_service
endif

# App_Link_Flags := $(SGX_COMMON_CFLAGS) -L$(SGX_LIBRARY_PATH) -l$(Urts_Library_Name) -lpthread
# App_C_Objects := $(App_C_Files:.c=.o)

App_Name := bin/app
Ce_Name := bin/ce

######## Enclave Settings ########

ifneq ($(SGX_MODE), HW)
	Trts_Library_Name := sgx_trts_sim
	Service_Library_Name := sgx_tservice_sim
else
	Trts_Library_Name := sgx_trts
	Service_Library_Name := sgx_tservice
endif
Crypto_Library_Name := sgx_tcrypto
KeyExchange_Library_Name := sgx_tkey_exchange

#RustEnclave_Include_Paths := -I$(SGX_SDK)/include -I$(SGX_SDK)/include/tlibc -I$(SGX_SDK)/include/stlport -I$(SGX_SDK)/include/epid -I ./enclave -I./include

# -lcompiler-rt-patch
# RustEnclave_Compile_Flags := $(SGX_COMMON_CFLAGS) -nostdinc -fvisibility=hidden -fpie -fstack-protector $(RustEnclave_Include_Paths)

RustEnclave_Link_Libs := -Lsrc/enclave/target/release/ -lenclave
RustEnclave_Link_Flags := $(SGX_COMMON_CFLAGS) -Wl,--no-undefined -nostdlib -nodefaultlibs -nostartfiles -L$(SGX_LIBRARY_PATH) \
	-Wl,--whole-archive -l$(Trts_Library_Name) -Wl,--no-whole-archive \
	-Wl,--start-group -lsgx_tstdc -lsgx_tstdcxx -l$(Crypto_Library_Name) -l$(KeyExchange_Library_Name) -l$(Service_Library_Name) $(RustEnclave_Link_Libs) -Wl,--end-group \
	-Wl,-Bstatic -Wl,-Bsymbolic \
	-Wl,-pie,-eenclave_entry -Wl,--export-dynamic  \
	-Wl,--defsym,__ImageBase=0 \
	-Wl,--gc-sections \
	-Wl,--version-script=src/Enclave.lds


RustEnclave_Name := bin/enclave.so
Signed_RustEnclave_Name := bin/enclave.signed.so

.PHONY: all
all: $(Enclave_EDL_Files) $(Signed_RustEnclave_Name) $(App_Name) $(Ce_Name)

######## EDL Objects ########

$(Enclave_EDL_Files): $(SGX_EDGER8R) src/Enclave.edl
	$(SGX_EDGER8R) --trusted src/Enclave.edl --search-path $(SGX_SDK)/include --trusted-dir src/enclave
	$(SGX_EDGER8R) --untrusted src/Enclave.edl --search-path $(SGX_SDK)/include --untrusted-dir src/app
	@echo "GEN  =>  $(Enclave_EDL_Files)"

######## App ########

.phony: app
app: $(Enclave_EDL_Files)
	cd src/app && cargo build --release

$(App_Name): app
	cp src/app/target/release/app $@

######## Enclave Objects ########

# cp ../compiler-rt/libcompiler-rt-patch.a ./lib
#	$(CXX) enclave/Enclave_t.o -o $@ $(RustEnclave_Link_Flags)
#	cp ./enclave/target/release/libenclave.so ./lib/libenclave.so

$(RustEnclave_Name): enclave
	@mkdir -p bin
	$(CXX) -o $@ $(RustEnclave_Link_Flags)
	@echo "LINK =>  $@"

$(Signed_RustEnclave_Name): $(RustEnclave_Name)
	$(SGX_ENCLAVE_SIGNER) sign -key src/Enclave_private.pem -enclave $(RustEnclave_Name) -out $@ -config src/Enclave.config.xml
	@echo "SIGN =>  $@"

.PHONY: enclave
enclave: $(Enclave_EDL_Files)
	cd src/enclave && cargo build --release

#.PHONY: compiler-rt
#compiler-rt:
#	$(MAKE) -C ../compiler-rt/ 2> /dev/null

.PHONY: clean_all clean
clean_all: clean
	rm -f $(Enclave_EDL_Files)
	cd src/app && cargo clean
	cd src/enclave && cargo clean
	cd src/ce && cargo clean
	rm -rf bin/*

clean:
	rm -f $(App_Name) $(RustEnclave_Name) $(Signed_RustEnclave_Name)

######## Compression and encryption application ########

.phony: ce
ce:
	cd src/ce && cargo build --release

$(Ce_Name): ce
	cp src/ce/target/release/ce $@
