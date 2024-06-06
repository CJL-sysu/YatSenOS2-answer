#![no_std]
#![no_main]

use lib::{vec::Vec, *};
use string::String;
mod mycd;
mod myls;
mod mycat;
mod myrun;
extern crate lib;
static mut CUR_DIR:String = String::new();
fn main() -> isize {
    // println!("To test the abilily to fork, YSOS will run app `fork` first");
    // sys_wait_pid(sys_spawn("fork"));
    // println!("Successfully exited app `fork`, YSOS will run shell next");
    unsafe{
        CUR_DIR = String::from("/");
    }
    loop {
        print!("\x1b[32m\x1b[1mCJL@YSOS\x1b[0m:\x1b[34m\x1b[1m{}\x1b[0m# ", unsafe {
            CUR_DIR.as_str()
        });
        let line = stdin().read_line();
        match line.trim() {
            "exit" => break,
            "app" => sys_list_app(),
            "ps" => sys_stat(),
            // "hello" => {sys_wait_pid(sys_spawn("hello"));},
            // "fac" => {sys_wait_pid(sys_spawn("fac"));},
            "clear" => {print!("\x1b[2J\x1b[1;1H");},
            // "fork" => {sys_wait_pid(sys_spawn("fork"));},
            // "counter" => {sys_wait_pid(sys_spawn("counter"));},
            // "mq" => {sys_wait_pid(sys_spawn("mq"));},
            "help" => {print_help();},
            _ =>{
                if !run(line.trim()){
                    myrun::myrun(line.trim());
                }
            },
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
    - run: 运行用户程序
    - clear: 清屏
    - help: 打印帮助信息
    - cd: 切换文件夹
    - cat: 查看文件内容
    - ls: 列出文件夹下文件\x1b[0m");
}

fn run(s: &str) -> bool{
    let v:Vec<&str> = s.split(" ").collect();
    if v.len() == 0{
        return false;
    }else if v[0] == "run"{
        let mut i = 1;
        while i < v.len(){
            if v[i].len() != 0{
                let pid = sys_spawn(v[i]);
                if pid == 0{
                    println!("Run: unknown process {}", v[i]);
                }else{
                    sys_wait_pid(pid);
                }
                return true;
            }
            i += 1;
        }
        println!("Help: run `the name of app`");
        println!("For example, `run hello` will start an app named `hello`");
        println!("You can view the app list with `app`");
        true
    }else if v[0] == "ls"{
        let dist;
        if v.len() == 1{
            unsafe{
                dist = CUR_DIR.as_str();
            }
        }else{
            dist = v[1];
        }
        myls::myls(dist);
        true
    }else if v[0] == "cat"{
        if v.len() == 1{
            println!("Help: cat `the name of file`");
            false
        }else{
            mycat::mycat(v[1]);
            true
        }
    }else if v[0] == "cd"{
        let target = if v.len() == 1{
            "/"
        }else{
            v[1]
        };
        mycd::mycd(target);
        true
    }else{
        false
    }
}

