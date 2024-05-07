#![no_std]
#![no_main]

use lib::*;

extern crate lib;

fn main() -> isize {
    loop {
        print!("[>] ");
        let line = stdin().read_line();
        match line.trim() {
            "exit" => break,
            "app" => sys_list_app(),
            "ps" => sys_stat(),
            "hello" => {sys_wait_pid(sys_spawn("hello"));},
            "fac" => {sys_wait_pid(sys_spawn("fac"));},
            "clear" => {print!("\x1b[2J\x1b[1;1H");},
            "help" => {print_help();},
            _ => println!("[=] {}", line),
        }
    }
    0
}

entry!(main);
fn print_help(){
    println!("\x1b[34;1;4m[*] help:\x1b[0m
    \x1b[34mYSOS shell 使用帮助\x1b[0m
    author: CJL-22330004
    \x1b[32m指令:
    - exit: 退出
    - ps: 展示当前所有进程
    - app: 展示所有用户程序
    - hello: 运行用户程序hello
    - fac: 运行用户程序fac, 用于计算阶乘
    - clear: 清屏
    - help: 打印帮助信息\x1b[0m");
}

