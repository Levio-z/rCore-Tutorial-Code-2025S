		.section .text.entry        # 代码段，入口函数放这里
		.globl _start              # 声明 _start 为全局符号
_start:                   # 引导入口点
    la sp, boot_stack_top # 初始化栈指针
    call rust_main        # 调用 Rust 主函数

		.section .bss.stack        # 定义一段未初始化的栈空间
		.globl boot_stack_lower_bound
boot_stack_lower_bound:
    .space 4096 * 16       # 分配 16 页（64KiB）栈空间
		.globl boot_stack_top
boot_stack_top: