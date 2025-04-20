# Building
# 定义目标架构、编译模式、内核ELF文件路径、内核BIN文件路径和反汇编临时文件路径
TARGET := riscv64gc-unknown-none-elf
MODE := release
KERNEL_ELF := target/$(TARGET)/$(MODE)/os1
KERNEL_BIN := $(KERNEL_ELF).bin
DISASM_TMP := target/$(TARGET)/$(MODE)/asm

# 根据编译模式设置编译参数
# 如果编译模式为release，则设置编译参数为--release
ifeq ($(MODE), release)
	MODE_ARG := --release
endif

# 定义板卡和SBI类型
# BOARD指定为qemu，SBI默认为rustsbi
# BOOTLOADER指定为相应的引导程序路径
BOARD := qemu
SBI ?= rustsbi
BOOTLOADER := ../bootloader/$(SBI)-$(BOARD).bin

# 定义内核入口物理地址
# 内核入口物理地址设置为0x80200000
KERNEL_ENTRY_PA := 0x80200000

# 定义Binutils工具
# OBJDUMP和OBJCOPY分别用于反汇编和复制二进制文件
OBJDUMP := rust-objdump --arch-name=riscv64
OBJCOPY := rust-objcopy --binary-architecture=riscv64

# 定义反汇编参数
# DISASM参数默认为-x
DISASM ?= -x

# 定义build目标
# build目标依赖于env和$(KERNEL_BIN)
build: env $(KERNEL_BIN)

# 定义环境变量
# 检查是否已安装目标架构，如果未安装，则添加目标架构
# 安装cargo-binutils工具
# 添加rust-src和llvm-tools-preview组件
env:
	(rustup target list | grep "riscv64gc-unknown-none-elf (installed)") || rustup target add $(TARGET)
	cargo install cargo-binutils
	rustup component add rust-src
	rustup component add llvm-tools-preview

# 定义$(KERNEL_BIN)目标
# 使用OBJCOPY将内核ELF文件转换为BIN文件
# : 表示执行这个命令的先决条件是kernel
$(KERNEL_BIN): kernel
	@$(OBJCOPY) $(KERNEL_ELF) --strip-all -O binary $@

# 定义kernel目标
# 打印板卡信息
# 根据编译模式编译内核
kernel:
	@echo Platform: $(BOARD)
	@cargo build $(MODE_ARG)

# 定义clean目标
# 使用cargo clean清理编译生成的文件
clean:
	@cargo clean

# 定义disasm目标
# 使用OBJDUMP反汇编内核ELF文件并输出到less
disasm: kernel
	@$(OBJDUMP) $(DISASM) $(KERNEL_ELF) | less

# 定义disasm-vim目标
# 使用OBJDUMP反汇编内核ELF文件并输出到临时文件
# 使用vim打开临时文件
# 删除临时文件
disasm-vim: kernel
	@$(OBJDUMP) $(DISASM) $(KERNEL_ELF) > $(DISASM_TMP)
	@vim $(DISASM_TMP)
	@rm $(DISASM_TMP)

# 定义run目标
# run目标依赖于run-inner
run: run-inner

# 定义run-inner目标
# 使用qemu-system-riscv64运行内核
run-inner: build
	@qemu-system-riscv64 \
		-machine virt \
		-nographic \
		-bios $(BOOTLOADER) \
		-device loader,file=$(KERNEL_BIN),addr=$(KERNEL_ENTRY_PA)

# 定义debug目标
# 使用tmux和gdb进行调试
debug: build
	tmux new-session -d \
		"qemu-system-riscv64 -machine virt -nographic -bios $(BOOTLOADER) -device loader,file=$(KERNEL_BIN),addr=$(KERNEL_ENTRY_PA) -s -S" && \
		tmux split-window -h "riscv64-unknown-elf-gdb -ex 'file $(KERNEL_ELF)' -ex 'set arch riscv:rv64' -ex 'target remote localhost:1234'" && \
		tmux -2 attach-session -d

# 定义gdbserver目标
# 使用qemu-system-riscv64作为gdb服务器
gdbserver: build
	@qemu-system-riscv64 -machine virt -nographic -bios $(BOOTLOADER) -device loader,file=$(KERNEL_BIN),addr=$(KERNEL_ENTRY_PA) -s -S

# 定义gdbclient目标
# 使用gdb作为客户端连接到gdb服务器
gdbclient:
	@riscv64-unknown-elf-gdb -ex 'file $(KERNEL_ELF)' -ex 'set arch riscv:rv64' -ex 'target remote localhost:1234'

# 声明所有目标为伪目标
.PHONY: build env kernel clean disasm disasm-vim run-inner gdbserver gdbclient
