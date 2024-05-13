#![no_std]
#![no_main]

use lib::{sync::{Semaphore, SpinLock}, *};

extern crate lib;

const THREAD_COUNT: usize = 8;
static mut COUNTER: isize = 0;
static LOCK: SpinLock = SpinLock::new();
static SEMA: Semaphore = Semaphore::new(0);

fn test_spin(){
    let mut pids = [0u16; THREAD_COUNT];

    for i in 0..THREAD_COUNT {
        let pid = sys_fork();
        if pid == 0 {
            do_counter_inc_spin();
            sys_exit(0);
        } else {
            pids[i] = pid; // only parent knows child's pid
        }
    }

    let cpid = sys_get_pid();
    println!("process #{} holds threads: {:?}", cpid, &pids);
    sys_stat();

    for i in 0..THREAD_COUNT {
        println!("#{} waiting for #{}...", cpid, pids[i]);
        sys_wait_pid(pids[i]);
    }

    println!("COUNTER result: {}", unsafe { COUNTER });
}

fn test_semaphore(){
    let mut pids = [0u16; THREAD_COUNT];
    let ret = SEMA.init(1);
    //print!("ret = {}", ret);
    for i in 0..THREAD_COUNT {
        let pid = sys_fork();
        if pid == 0 {
            do_counter_inc_sema();
            sys_exit(0);
        } else {
            pids[i] = pid; // only parent knows child's pid
        }
    }

    let cpid = sys_get_pid();
    println!("process #{} holds threads: {:?}", cpid, &pids);
    sys_stat();

    for i in 0..THREAD_COUNT {
        println!("#{} waiting for #{}...", cpid, pids[i]);
        sys_wait_pid(pids[i]);
    }

    println!("COUNTER result: {}", unsafe { COUNTER });
}

fn main() -> isize {
    let pid = sys_fork();

    if pid == 0 {
        print!("\x1b[32m test semaphore begin now\n\x1b[0m");
        test_semaphore();
        print!("\x1b[32m test semaphore end\n\x1b[0m");
    } else {
        sys_wait_pid(pid);
        print!("\x1b[32m test spin begin now\n\x1b[0m");
        unsafe{
            COUNTER = 0;
        }
        test_spin();
        print!("\x1b[32m test spin end\n\x1b[0m");
    }
    0
}

fn do_counter_inc_spin() {
    for _ in 0..100 {
        // FIXME: protect the critical section
        LOCK.acquire();
        inc_counter();
        LOCK.release();
    }
}
fn do_counter_inc_sema() {
    for _ in 0..100 {
        // FIXME: protect the critical section
        SEMA.wait();
        //self::print!("after wait");
        inc_counter();
        //self::print!("before signal");
        SEMA.signal();
    }
}

/// Increment the counter
///
/// this function simulate a critical section by delay
/// DO NOT MODIFY THIS FUNCTION
fn inc_counter() {
    unsafe {
        delay();
        let mut val = COUNTER;
        delay();
        val += 1;
        delay();
        COUNTER = val;
    }
}

#[inline(never)]
#[no_mangle]
fn delay() {
    for _ in 0..0x100 {
        core::hint::spin_loop();
    }
}

entry!(main);
