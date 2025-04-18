#![no_std] // 指定不使用标准库
#![no_main] // 指定不使用标准main函数
#![feature(panic_info_message)]//启用这个特性后，panic 处理程序可以通过访问更多的上下文信息
mod lang_items; // 导入语言项模块
mod sbi;// 引入sbi模块
mod console;// 引入console模块
use core::arch::global_asm; // 导入全局汇编宏
global_asm!(include_str!("entry.asm")); // 包含汇编文件
#[no_mangle]
pub fn rust_main() -> ! {
    clear_bss();
    println!("Hello world!");
    // panic!("Shutdown machine!");
    loop {
        
    }
}

fn clear_bss() {
    // 使用外部C函数来获取BSS段的开始和结束地址
    extern "C" {
        // sbss是BSS段的开始地址
        fn sbss();
        // ebss是BSS段的结束地址
        fn ebss();
    }
    // 遍历BSS段的所有地址，并将其初始化为0
    (sbss as usize..ebss as usize).for_each(|a| {
        // 使用volatile写入来确保编译器不会优化掉这次写入
        unsafe { (a as *mut u8).write_volatile(0) }
    });
}