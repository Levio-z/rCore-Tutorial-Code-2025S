// 导入console_putchar函数用于输出字符
use crate::sbi::console_putchar;
// 导入Write trait用于实现输出
use core::fmt::{self, Write};
// 定义Stdout结构体用于实现Write trait
struct Stdout;
// 实现Stdout的Write trait
impl Write for Stdout {
    // 实现write_str方法，用于将字符串写入到Stdout
    fn write_str(&mut self, s: &str) -> fmt::Result {
        // 遍历字符串中的每个字符，并将其输出
        for c in s.chars() {
            console_putchar(c as usize);
        }
        Ok(())
    }
}

// 定义print函数，用于格式化输出
pub fn print(args: fmt::Arguments) {
    // 使用Stdout的write_fmt方法进行格式化输出
    Stdout.write_fmt(args).unwrap();
}

// 定义print!宏，用于简化print函数的调用
#[macro_export]
macro_rules! print {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        // 使用format_args!宏格式化参数，并调用print函数
        $crate::console::print(format_args!($fmt $(, $($arg)+)?));
    }
}

// 定义println!宏，用于简化print函数的调用，并在结尾添加换行符
#[macro_export]
macro_rules! println {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        // 使用format_args!宏格式化参数，并在结尾添加换行符，然后调用print函数
        $crate::console::print(format_args!(concat!($fmt, "\n") $(, $($arg)+)?));
    }
}