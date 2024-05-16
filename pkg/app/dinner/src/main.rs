#![no_std]
#![no_main]


use lib::{sync::Semaphore, *};

extern crate lib;
const PHI_NUM: usize = 5;
static CHOPSTICK: [Semaphore; 5] = semaphore_array![0, 1, 2, 3, 4];
static S1: Semaphore = Semaphore::new(5);
static S2: Semaphore = Semaphore::new(6);
static mut PHI_COUNT: [i32; PHI_NUM] = [0; PHI_NUM];
fn main() -> isize {
    let help = "help: \n函数1：常规解法，会造成死锁\n函数2：要求奇数号哲学家先拿左边的筷子，然后再拿右边的筷子,而偶数号哲学家刚好相反。不存在死锁和饥饿\n函数3，要求哲学家必须按照筷子编号从小到大拿筷子,会造成不公平\n函数4：使用服务生协调，不存在死锁和饥饿";
    let stdin1 = stdin();
    println!("请选择使用的函数");
    println!("{}",help);
    let s = stdin1.read_line();
    let s = s.trim();
    match s{
        "1" => {
            println!("函数1：常规解法，会造成死锁");
        }
        "2" => {
            println!("函数2：要求奇数号哲学家先拿左边的筷子，然后再拿右边的筷子,而偶数号哲学家刚好相反。不存在死锁和饥饿");
        }
        "3" => {
            println!("函数3，要求哲学家必须按照筷子编号从小到大拿筷子,会造成不公平");
        },
        "4" => {
            println!("函数4：使用服务生协调，不存在死锁和饥饿");
        }
        _ => {
            println!("invalid input");
            println!("{}",help);
            return 0;
        }
    };
    //初始化信号量
    for i in 0..PHI_NUM{
        CHOPSTICK[i].init(1);
    }
    S1.init(1);
    S2.init(1);
    let mut pids: [u16; PHI_NUM] = [0u16; PHI_NUM];
    for i in 0..PHI_NUM{
        let pid = sys_fork();
        if pid == 0{
            if s == "1"{
                philosopher1(i);
            }else if s == "2"{
                philosopher2(i);
            }else if s == "3"{
                philosopher3(i);
            }else if s == "4"{
                philosopher4(i);
            }else{
                panic!("s should be 1, 2 or 3");
            }
            sys_exit(0);
        }
        pids[i] = pid;
    }
    let cpid = sys_get_pid();
    for i in 0..PHI_NUM {
        //println!("#{} waiting for #{}...", cpid, pids[i]);
        sys_wait_pid(pids[i]);
    }
    //销毁信号量
    for i in 0..PHI_NUM{
        CHOPSTICK[i].remove();
    }
    S1.remove();
    S2.remove();
    0
}
const SLEEP_TIME: i64 = 2;
// 函数1，常规解法，会造成死锁
fn philosopher1(i: usize){
    let c1 = i;
    let c2 = (i + 1) % PHI_NUM;
    for _a in 0..20{
        //thinking
        CHOPSTICK[c1].wait();
        println!("Philosopher {} get chopstick {}", i, c1);
        sleep(SLEEP_TIME);
        CHOPSTICK[c2].wait();
        println!("Philosopher {} get chopstick {}", i, c2);
        sleep(SLEEP_TIME);
        //eating
        println!("\x1b[32mPhilosopher {} is eating\x1b[0m", i);
        CHOPSTICK[c1].signal();
        println!("Philosopher {} release chopstick {}", i, c1);
        sleep(SLEEP_TIME);
        CHOPSTICK[c2].signal();
        println!("Philosopher {} release chopstick {}", i, c2);
    }
}
// 函数2，要求奇数号哲学家先拿左边的筷子，然后再拿右边的筷子,而偶数号哲学家刚好相反。不存在死锁和饥饿
fn philosopher2(i: usize){
    let mut c1 = i;
    let mut c2 = (i + 1) % PHI_NUM;
    for _a in 0..20{
        //thinking
        if i % 2 == 0 {
            c1 = c1 ^ c2;
            c2 = c1 ^ c2;
            c1 = c1 ^ c2;
        }
        CHOPSTICK[c1].wait();
        println!("Philosopher {} get chopstick {}", i, c1);
        sleep(SLEEP_TIME);
        CHOPSTICK[c2].wait();
        println!("Philosopher {} get chopstick {}", i, c2);
        sleep(SLEEP_TIME);
        //eating
        unsafe{
            PHI_COUNT[i] += 1;
            println!("\x1b[32mPhilosopher {} is eating, he has eaten {} times.\x1b[0m", i, PHI_COUNT[i]);
        }
        CHOPSTICK[c1].signal();
        println!("Philosopher {} release chopstick {}", i, c1);
        sleep(SLEEP_TIME);
        CHOPSTICK[c2].signal();
        println!("Philosopher {} release chopstick {}", i, c2);
    }
}

// 函数3，要求哲学家必须按照筷子编号从小到大拿筷子,会造成不公平
fn philosopher3(i: usize){
    let mut c1 = i;
    let mut c2 = (i + 1) % PHI_NUM;
    for _a in 0..100{
        //thinking
        if c1 > c2 {
            c1 = c1 ^ c2;
            c2 = c1 ^ c2;
            c1 = c1 ^ c2;
        }
        CHOPSTICK[c1].wait();
        //println!("Philosopher {} get chopstick {}", i, c1);
        sleep(SLEEP_TIME);
        CHOPSTICK[c2].wait();
        //println!("Philosopher {} get chopstick {}", i, c2);
        sleep(SLEEP_TIME);
        //eating
        unsafe{
            PHI_COUNT[i] += 1;
            println!("\x1b[32mPhilosopher {} is eating, he has eaten {} times.\x1b[0m", i, PHI_COUNT[i]);
        }
        
        CHOPSTICK[c1].signal();
        //println!("Philosopher {} release chopstick {}", i, c1);
        sleep(SLEEP_TIME);
        CHOPSTICK[c2].signal();
        //println!("Philosopher {} release chopstick {}", i, c2);
    }
}

//函数4：使用服务生协调，不存在死锁和饥饿
static mut CHO_USED: [bool; 5] = [false; 5];

fn ask_server_for_cho(i: usize) -> bool{
    let c1 = i;
    let c2 = (i + 1) % PHI_NUM;
    unsafe{
        if !CHO_USED[c1] && !CHO_USED[c2] {
            CHO_USED[c1] = true;
            CHO_USED[c2] = true;
            return true;
        }else{
            return false;
        }
    }
}
fn release_cho(i: usize){
    let c1 = i;
    let c2 = (i + 1) % PHI_NUM;
    unsafe{
        CHO_USED[c1] = false;
        CHO_USED[c2] = false;
    }
}
fn philosopher4(i: usize){
    let mut c1 = i;
    let mut c2 = (i + 1) % PHI_NUM;
    for _a in 0..30{
        sleep(SLEEP_TIME);
        //thinking
        loop{
            S1.wait();
            let ret = ask_server_for_cho(i);
            S1.signal();
            if ret{
                break;
            }
        }
        // eating
        println!("Philosopher {} get chopstick {}", i, c1);
        println!("Philosopher {} get chopstick {}", i, c2);
        unsafe{
            PHI_COUNT[i] += 1;
            println!("\x1b[32mPhilosopher {} is eating, he has eaten {} times.\x1b[0m", i, PHI_COUNT[i]);
        }
        println!("Philosopher {} release chopstick {}", i, c1);
        println!("Philosopher {} release chopstick {}", i, c2);
        S2.wait();
        release_cho(i);
        S2.signal();
        
    }
}

entry!(main);
