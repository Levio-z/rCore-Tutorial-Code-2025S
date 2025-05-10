// 引入外部 crate
use clap::{App, Arg}; // 用于解析命令行参数
use easy_fs::{BlockDevice, EasyFileSystem}; // 引入 EasyFileSystem 的定义
use std::fs::{read_dir, File, OpenOptions}; // 文件读写操作
use std::io::{Read, Seek, SeekFrom, Write}; // IO trait
use std::sync::Arc; // 原子引用计数，用于共享资源
use std::sync::Mutex; // 互斥锁保护共享资源

const BLOCK_SZ: usize = 512; // 每个块的大小固定为 512 字节

/// 封装一个带互斥锁的 File 结构体，用于模拟块设备
struct BlockFile(Mutex<File>);

/// 实现 BlockDevice trait，提供块级读写能力
impl BlockDevice for BlockFile {
    fn read_block(&self, block_id: usize, buf: &mut [u8]) {
        let mut file = self.0.lock().unwrap();
        file.seek(SeekFrom::Start((block_id * BLOCK_SZ) as u64))
            .expect("Error when seeking!");
        assert_eq!(file.read(buf).unwrap(), BLOCK_SZ, "Not a complete block!");
    }

    fn write_block(&self, block_id: usize, buf: &[u8]) {
        let mut file = self.0.lock().unwrap();
        file.seek(SeekFrom::Start((block_id * BLOCK_SZ) as u64))
            .expect("Error when seeking!");
        assert_eq!(file.write(buf).unwrap(), BLOCK_SZ, "Not a complete block!");
    }
}

/// 主程序入口
fn main() {
    easy_fs_pack().expect("Error when packing easy-fs!");
}

/// 用于将一批用户程序打包进 EasyFileSystem 中
fn easy_fs_pack() -> std::io::Result<()> {
    // 解析命令行参数 -s 表示源目录，-t 表示目标目录
    let matches = App::new("EasyFileSystem packer")
        .arg(
            Arg::with_name("source")
                .short("s")
                .long("source")
                .takes_value(true)
                .help("Executable source dir(with backslash)"),
        )
        .arg(
            Arg::with_name("target")
                .short("t")
                .long("target")
                .takes_value(true)
                .help("Executable target dir(with backslash)"),
        )
        .get_matches();
    
    let src_path = matches.value_of("source").unwrap();
    let target_path = matches.value_of("target").unwrap();
    println!("src_path = {}\ntarget_path = {}", src_path, target_path);

    // 创建一个 block_file 作为文件系统的底层设备
    let block_file = Arc::new(BlockFile(Mutex::new({
        let f = OpenOptions::new()
            .read(true)// 打开后读取
            .write(true)//打开后允许写入
            .create(true)//如果不存在则创建
            .open(format!("{}{}", target_path, "fs.img"))?; // 创建磁盘镜像
        f.set_len(16 * 2048 * 512).unwrap(); // 设置镜像文件大小为 16 MiB
        f
    })));

    // 初始化 EasyFileSystem，容量为 16*2048 个 block
    let efs = EasyFileSystem::create(block_file, 16 * 2048, 1);
    let root_inode = Arc::new(EasyFileSystem::root_inode(&efs));

    // 从源目录中读取所有文件，取其文件名去掉扩展名作为目标文件名
    let apps: Vec<_> = read_dir(src_path)
        .unwrap()
        .into_iter()
        .map(|dir_entry| {
            let mut name_with_ext = dir_entry.unwrap().file_name().into_string().unwrap();
            name_with_ext.drain(name_with_ext.find('.').unwrap()..name_with_ext.len());
            name_with_ext
        })
        .collect();

    for app in apps {
        // 读取源文件内容
        // 打开一个可执行文件，将它的内容全部读入内存中的一个 Vec<u8> 缓冲区中。
        let mut host_file: File = File::open(format!("{}{}", target_path, app)).unwrap();
        let mut all_data: Vec<u8> = Vec::new();
        host_file.read_to_end(&mut all_data).unwrap();

        // 在 easy-fs 中创建新文件并写入数据
        let inode = root_inode.create(app.as_str()).unwrap();
        // 写入u8切片
        inode.write_at(0, all_data.as_slice());
    }

    Ok(())
}

/// 单元测试：测试 EasyFileSystem 的基本读写能力
#[test]
fn efs_test() -> std::io::Result<()> {
    let block_file = Arc::new(BlockFile(Mutex::new({
        let f = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open("target/fs.img")?; // 测试使用的磁盘镜像
        f.set_len(8192 * 512).unwrap(); // 大小为 4MiB
        f
    })));

    EasyFileSystem::create(block_file.clone(), 4096, 1); // 创建文件系统
    let efs = EasyFileSystem::open(block_file.clone()); // 打开文件系统
    let root_inode = EasyFileSystem::root_inode(&efs); // 根目录 inode

    root_inode.create("filea");
    root_inode.create("fileb");

    for name in root_inode.ls() {
        println!("{}", name);
    }

    let filea = root_inode.find("filea").unwrap();
    let greet_str = "Hello, world!";
    filea.write_at(0, greet_str.as_bytes());

    let mut buffer = [0u8; 233];
    let len = filea.read_at(0, &mut buffer);
    assert_eq!(greet_str, core::str::from_utf8(&buffer[..len]).unwrap());

    /// 测试写入和读取随机字符串
    let mut random_str_test = |len: usize| {
        filea.clear(); // 清空文件
        assert_eq!(filea.read_at(0, &mut buffer), 0);
        let mut str = String::new();
        use rand;
        for _ in 0..len {
            str.push(char::from('0' as u8 + rand::random::<u8>() % 10)); // 生成随机数字字符
        }
        filea.write_at(0, str.as_bytes());

        let mut read_buffer = [0u8; 127];
        let mut offset = 0usize;
        let mut read_str = String::new();
        loop {
            let len = filea.read_at(offset, &mut read_buffer);
            if len == 0 {
                break;
            }
            offset += len;
            read_str.push_str(core::str::from_utf8(&read_buffer[..len]).unwrap());
        }
        assert_eq!(str, read_str);
    };

    // 多组长度的随机字符串测试
    random_str_test(4 * BLOCK_SZ);
    random_str_test(8 * BLOCK_SZ + BLOCK_SZ / 2);
    random_str_test(100 * BLOCK_SZ);
    random_str_test(70 * BLOCK_SZ + BLOCK_SZ / 7);
    random_str_test((12 + 128) * BLOCK_SZ);
    random_str_test(400 * BLOCK_SZ);
    random_str_test(1000 * BLOCK_SZ);
    random_str_test(2000 * BLOCK_SZ);

    Ok(())
}
