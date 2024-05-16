use core::{
    hint::spin_loop,
    sync::atomic::{AtomicBool, Ordering},
};

use crate::*;

pub struct SpinLock {
    bolt: AtomicBool,
}

impl SpinLock {
    pub const fn new() -> Self {
        Self {
            bolt: AtomicBool::new(false),
        }
    }

    pub fn acquire(&self) {
        // FIXME: acquire the lock, spin if the lock is not available
        while self.bolt.compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed).is_err() {
            spin_loop();
        }
    }

    pub fn release(&self) {
        // FIXME: release the lock
        self.bolt.store(false, Ordering::Release);
    }
}

unsafe impl Sync for SpinLock {} // Why? Check reflection question 5

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Semaphore {
    /* FIXME: record the sem key */
    key: u32
}

impl Semaphore {
    pub const fn new(key: u32) -> Self {
        Semaphore { key }
    }

    #[inline(always)]
    pub fn init(&self, value: usize) -> bool {
        sys_new_sem(self.key, value) == 1
    }

    /* FIXME: other functions with syscall... */
    pub fn remove(&self) -> bool{
        sys_del_sem(self.key) == 1
    }

    pub fn signal(&self){
        //self::print!("signal is calledin pkg/lib/src/sync.rs, line 56");
        sys_signal(self.key);
    }

    pub fn wait(&self){
        sys_wait(self.key);
    }
}

unsafe impl Sync for Semaphore {}

#[macro_export]
macro_rules! semaphore_array {
    [$($x:expr),+ $(,)?] => {
        [ $(Semaphore::new($x),)* ]
    }
}
