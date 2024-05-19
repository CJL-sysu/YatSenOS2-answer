#![no_std]
#![no_main]
use rand::prelude::*;
use rand_chacha::ChaCha20Rng;
use lib::{sync::Semaphore, *};

extern crate lib;
const WORKER_NUM: usize = 3;
static S: [Semaphore; 8] = semaphore_array![0, 1, 2, 3, 4, 5, 6, 7];
static mut FIRST: u8 =0;
fn main() -> isize {
    for i in 0..8{
        if i != 1{
            S[i].init(0);
        }else{
            S[i].init(1);
        }
    }
    let mut pids: [u16; WORKER_NUM] = [0u16; WORKER_NUM];
    for i in 0..WORKER_NUM{
        let pid = sys_fork();
        if pid == 0{
            match i {
                0 => f1(),
                1 => f2(),
                2 => f3(),
                _ => panic!("error"),
            };
            sys_exit(0);
        }
        pids[i] = pid;
    }
    for i in 0..WORKER_NUM {
        //println!("#{} waiting for #{}...", cpid, pids[i]);
        sys_wait_pid(pids[i]);
    }
    for i in 0..8{
        S[i].remove();
    }
    0
}
entry!(main);

const TIMES: u8 = 10;

fn f1(){//>
    let s = ">";
    let time = lib::sys_time();
    let mut rng = ChaCha20Rng::seed_from_u64(time.timestamp() as u64);
    for _ in 0..TIMES{
        
        sleep((rng.gen::<u64>() % 10).try_into().unwrap());
        S[1].wait();
        unsafe{
            if FIRST == 0{
                FIRST = 1;
            }
        }
        print!("{}",s);
        S[1].signal();
        S[3].signal();
        S[2].wait();
        unsafe{
            if FIRST == 2{
                S[4].wait();
                S[7].signal();
            }else{
                print!("{}",s);
                S[4].signal();
                S[7].wait();
                S[5].signal();
            }
        }
        S[6].wait();
    }
}
fn f2(){//<
    let s = "<";
    let time = lib::sys_time();
    let mut rng = ChaCha20Rng::seed_from_u64(time.timestamp() as u64);
    for _ in 0..TIMES{
        sleep((rng.gen::<u64>() % 10).try_into().unwrap());
        S[1].wait();
        unsafe{
            if FIRST == 0{
                FIRST = 2;
            }
        }
        print!("{}",s);
        S[1].signal();
        S[2].signal();
        S[3].wait();
        unsafe{
            if FIRST == 2{
                print!("{}",s);
                S[4].signal();
                S[7].wait();
                S[5].signal();
            }else{
                S[4].wait();
                S[7].signal();
            }
        }
        S[0].wait();
    }
}
fn f3(){//_
    let s = "_";
    for _ in 0..TIMES{
        S[5].wait();
        print!("{}", s);
        unsafe{
            FIRST = 0;
        }
        S[6].signal();
        S[0].signal();
    }
}