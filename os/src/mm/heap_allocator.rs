//! 内核堆内存分配器模块
//! 
//! 该模块实现了操作系统内核的动态内存分配功能：
//! 1. 定义了全局分配器 HEAP_ALLOCATOR
//! 2. 实现了内存分配错误处理
//! 3. 提供了堆内存初始化功能
//! 4. 包含了堆内存分配的测试用例

use crate::config::KERNEL_HEAP_SIZE;
// 使用buddy system（伙伴系统）作为分配算法的实现
use buddy_system_allocator::LockedHeap;

// 声明全局分配器
#[global_allocator]
/// 堆分配器实例，使用互斥锁包装以支持多线程访问
static HEAP_ALLOCATOR: LockedHeap = LockedHeap::empty();

// 实现内存分配错误处理函数
#[alloc_error_handler]
/// 当堆内存分配发生错误时，打印错误信息并触发 panic
pub fn handle_alloc_error(layout: core::alloc::Layout) -> ! {
    panic!("Heap allocation error, layout = {:?}", layout);
}

/// 堆空间，大小由 KERNEL_HEAP_SIZE 常量定义
/// 使用静态可变数组作为堆内存空间
static mut HEAP_SPACE: [u8; KERNEL_HEAP_SIZE] = [0; KERNEL_HEAP_SIZE];

/// 初始化堆分配器
/// 
/// 该函数需要在操作系统启动早期被调用，以便初始化内核堆分配器
/// 使用 unsafe 块是因为需要操作静态可变数组 HEAP_SPACE
pub fn init_heap() {
    unsafe {
        HEAP_ALLOCATOR
            .lock()
            .init(HEAP_SPACE.as_ptr() as usize, KERNEL_HEAP_SIZE);
    }
}

/// 堆内存分配器的测试函数
/// 
/// 测试内容包括：
/// 1. Box 智能指针的分配和释放
/// 2. Vec 动态数组的创建、插入和遍历
/// 3. 确保分配的内存位于 .bss 段内
#[allow(unused)]
pub fn heap_test() {
    use alloc::boxed::Box;
    use alloc::vec::Vec;
    // 声明外部符号，用于获取 .bss 段的起始和结束地址
    extern "C" {
        fn sbss();
        fn ebss();
    }
    // 计算 .bss 段的地址范围
    let bss_range = sbss as usize..ebss as usize;
    
    // 测试 1：Box 分配
    let a = Box::new(5);
    assert_eq!(*a, 5);
    // 验证分配的内存是否在 .bss 段内
    assert!(bss_range.contains(&(a.as_ref() as *const _ as usize)));
    drop(a);
    
    // 测试 2：Vec 动态数组
    let mut v: Vec<usize> = Vec::new();
    // 向 Vec 中插入数据
    for i in 0..500 {
        v.push(i);
    }
    // 验证数据正确性
    for (i, val) in v.iter().take(500).enumerate() {
        assert_eq!(*val, i);
    }
    // 验证 Vec 的内存分配是否在 .bss 段内
    assert!(bss_range.contains(&(v.as_ptr() as usize)));
    drop(v);
    println!("heap_test passed!");
}
