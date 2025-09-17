LINUX_CC := gcc
WIN_CC := x86_64-w64-mingw32-gcc

CCFLAGS := -ggdb -O2 -Wall -Wextra -std=gnu11 -Iinclude -MMD -MP
LDFLAGS := -fsanitize=address

WIN_LDFLAGS = -static -lshlwapi

BUILD_DIR := $(CURDIR)/build
SRC_DIR := ./src
OBJ_DIR := ./obj

LINUX_OBJ_DIR := $(OBJ_DIR)/linux
WIN_OBJ_DIR := $(OBJ_DIR)/win

SOURCES := $(shell find $(SRC_DIR) -name "*.c")

LINUX_OBJECTS := $(SOURCES:$(SRC_DIR)/%.c=$(LINUX_OBJ_DIR)/%.o)
WIN_OBJECTS := $(SOURCES:$(SRC_DIR)/%.c=$(WIN_OBJ_DIR)/%.o)

LINUX_DEPS := $(LINUX_OBJECTS:.o=.d)
WIN_DEPS := $(WIN_OBJECTS:.o=.d)

LINUX_TARGET := $(BUILD_DIR)/hyc
WIN_TARGET := $(BUILD_DIR)/hyc.exe

.PHONY: all clean run

all: linux windows

linux: $(LINUX_TARGET)

windows: $(WIN_TARGET)

$(LINUX_OBJ_DIR)/%.o: $(SRC_DIR)/%.c
	@mkdir -p $(dir $@)
	$(LINUX_CC) $(CCFLAGS) -c $< -o $@

$(WIN_OBJ_DIR)/%.o: $(SRC_DIR)/%.c
	@mkdir -p $(dir $@)
	$(WIN_CC) $(CCFLAGS) -c $< -o $@

$(LINUX_TARGET): $(LINUX_OBJECTS)
	@mkdir -p $(BUILD_DIR)
	$(LINUX_CC) $(LINUX_OBJECTS) -o $(LINUX_TARGET)
	# $(LINUX_CC) $(LINUX_OBJECTS) $(LDFLAGS) -o $(LINUX_TARGET)

$(WIN_TARGET): $(WIN_OBJECTS)
	@mkdir -p $(BUILD_DIR)
	$(WIN_CC) $(WIN_OBJECTS) $(WIN_LDFLAGS) -o $(WIN_TARGET)
	# $(WIN_CC) $(WIN_OBJECTS) $(LDFLAGS) $(WIN_LDFLAGS) -o $(WIN_TARGET)

lr: linux
	$(LINUX_TARGET)

wr: windows
	$(WIN_TARGET)

vars:
	@echo "Sources: $(SOURCES)"
	@echo "Linux objects: $(LINUX_OBJECTS)"
	@echo "Windows objects: $(WIN_OBJECTS)"
clean:
	rm -rf $(BUILD_DIR) $(OBJ_DIR) *.exe *.o *.d

-include $(LINUX_DEPS)
-include $(WIN_DEPS)

