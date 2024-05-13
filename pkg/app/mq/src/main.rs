#![no_std]
#![no_main]

use lib::{sync::Semaphore, *};
extern crate lib;
static MUTEX: Semaphore = Semaphore::new(1);
static EMPTY: Semaphore = Semaphore::new(2);
static FULL: Semaphore = Semaphore::new(3);
static MAKE_PRODUCER_COMPLETE: Semaphore = Semaphore::new(4);
static COMSUMER_SHOW_COMPLETE: Semaphore = Semaphore::new(5);
const QUEUE_LEN: usize = 5;
static mut QUEUE: [u8; QUEUE_LEN] = [0; QUEUE_LEN];
static mut QUEUE_HEAD: usize = 0;
const WORKER_NUM: usize = 8;

fn main() -> isize {
    MUTEX.init(1);
    EMPTY.init(QUEUE_LEN);
    FULL.init(0);
    MAKE_PRODUCER_COMPLETE.init(0);
    COMSUMER_SHOW_COMPLETE.init(0);
    let pid = sys_fork();
    if pid == 0{ // child
        make_producer();
    }else{ // parent
        make_consumer();
        sys_wait_pid(pid);
        unsafe{
            print!("QUEUE_HEAD = {}\n", QUEUE_HEAD);
        }
        COMSUMER_SHOW_COMPLETE.remove();
        MAKE_PRODUCER_COMPLETE.remove();
        FULL.remove();
        EMPTY.remove();
        MUTEX.remove();
    }
    
    0
}
entry!(main);

fn make_producer(){
    let mut pids: [u16; WORKER_NUM] = [0u16; WORKER_NUM];
    for i in 0..WORKER_NUM{
        let pid = sys_fork();
        if pid == 0{
            producer();
            sys_exit(0);
        }
        pids[i] = pid;
    }
    MAKE_PRODUCER_COMPLETE.signal();
    COMSUMER_SHOW_COMPLETE.wait();
    let cpid = sys_get_pid();
    for i in 0..WORKER_NUM {
        println!("#{} waiting for #{}...", cpid, pids[i]);
        sys_wait_pid(pids[i]);
    }
    sys_exit(0);
}

fn producer(){
    for i in 0..10{
        EMPTY.wait();
        MUTEX.wait();
        print!("push {} in queue\n", i);
        unsafe{
            if QUEUE_HEAD >= QUEUE_LEN{
                panic!("queue overflow");
            }
            QUEUE[QUEUE_HEAD] = i;
            QUEUE_HEAD += 1;
        }
        MUTEX.signal();
        FULL.signal();
    }
}

fn make_consumer(){
    let mut pids: [u16; WORKER_NUM] = [0u16; WORKER_NUM];
    for i in 0..WORKER_NUM{
        let pid = sys_fork();
        if pid == 0{
            consumer();
            sys_exit(0);
        }
        pids[i] = pid;
    }
    MAKE_PRODUCER_COMPLETE.wait();
    sys_stat();
    COMSUMER_SHOW_COMPLETE.signal();
    let cpid = sys_get_pid();
    for i in 0..WORKER_NUM {
        println!("#{} waiting for #{}...", cpid, pids[i]);
        sys_wait_pid(pids[i]);
    }
}
fn consumer(){
    for _i in 0..10{
        FULL.wait();
        MUTEX.wait();
        unsafe{
            if QUEUE_HEAD == 0{
                panic!("read when queue empty")
            }
            QUEUE_HEAD -= 1;
            let data = QUEUE[QUEUE_HEAD];
            print!("read {} from queue\n", data);
        }
        MUTEX.signal();
        EMPTY.signal();
    }
}
