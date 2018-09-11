# configuration of makefile
.SUFFIXES:

# path definitions
BASE=.
SRC=$(BASE)
BUILD=$(BASE)/build
OBJ=$(BUILD)/obj
BIN=$(BUILD)/bin

# include path
INCLUDE+=-lcrypto

# compilation tools
# CC=clang++
CC=g++

# addition of generic compiler configuration
CFLAGS+=-O3
# CFLAGS+=-g
CFLAGS+=-std=c++11
CFLAGS+=-fpermissive

# linker configuration
# LFLAGS+=-fopenmp
LFLAGS+=$(INCLUDE)
LFLAGS+=-O3
# LFLAGS+=-g

# list of source files to compile
SOURCES=$(wildcard $(SRC)/*.cxx)
OBJECTS=$(SOURCES:$(SRC)/%.cxx=$(OBJ)/%.o)
EXECUTABLE=ce

# definition of rules that must be evaluated at each invocation
.PHONY: all clean

# build all object files
all: $(EXECUTABLE)

# clean all productions
clean:
	rm -rf $(BUILD) $(EXECUTABLE)

# compile an object file from a source file
$(OBJ)/%.o: $(SRC)/%.cxx
	@mkdir -p $(dir $@)
	$(CC) $(CFLAGS) -c -o $@ $<

#compile binary
$(EXECUTABLE): $(OBJECTS)
	@mkdir -p $(dir $@)
	$(CC) -o $@ $^ $(LFLAGS)
	


