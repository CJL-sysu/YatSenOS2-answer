use alloc::{collections::BTreeMap, sync::Arc};
use spin::RwLock;
use x86_64::structures::paging::{
    page::{PageRange, PageRangeInclusive},
    Page,
};
use crate::resource::ResourceSet;

use self::sync::{SemaphoreResult, SemaphoreSet};

use super::*;

#[derive(Debug, Clone)]
pub struct ProcessData {
    // shared data
    pub(super) env: Arc<RwLock<BTreeMap<String, String>>>,

    // process specific data
    pub(super) stack_segment: Option<PageRange>,
    pub(super) max_stack_segment: Option<PageRange>,
    pub(super) resources: Arc<RwLock<ResourceSet>>,
    pub(super) semaphores: Arc<RwLock<SemaphoreSet>>
}

impl Default for ProcessData {
    fn default() -> Self {
        Self {
            env: Arc::new(RwLock::new(BTreeMap::new())),
            stack_segment: None,
            max_stack_segment: None,
            resources: Arc::new(RwLock::new(ResourceSet::default())),
            semaphores: Arc::new(RwLock::new(SemaphoreSet::default()))
        }
    }
}

impl ProcessData {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn env(&self, key: &str) -> Option<String> {
        self.env.read().get(key).cloned()
    }

    pub fn set_env(&mut self, key: &str, val: &str) {
        self.env.write().insert(key.into(), val.into());
    }

    pub fn set_stack(&mut self, start: VirtAddr, size: u64) {
        let start = Page::containing_address(start);
        self.stack_segment = Some(Page::range(start, start + size));
        //info!("stack segment: {:?}", self.stack_segment);
        let start_u64 = Page::range(start, start + size).start.start_address().as_u64();
        let end_u64 = Page::range(start, start + size).end.start_address().as_u64();
        trace!("set stack: [{:#x} , {:#x})", start_u64, end_u64);
    }
    pub fn set_max_stack(&mut self, start: VirtAddr, size: u64) {
        let start = Page::containing_address(start);
        self.max_stack_segment = Some(Page::range(start, start + size));
        let start_u64 = Page::range(start, start + size).start.start_address().as_u64();
        let end_u64 = Page::range(start, start + size).end.start_address().as_u64();
        trace!("set max stack: [{:#x} , {:#x})", start_u64, end_u64);
    }
    pub fn is_on_stack(&self, addr: VirtAddr) -> bool {
        // FIXME: check if the address is on the stack
        match self.stack_segment{
            Some(range) => {
                let addr = addr.as_u64();
                let start = range.start.start_address().as_u64();
                let end = range.end.start_address().as_u64();
                trace!("stack range of current process: [{:#x} , {:#x}), addr = {:#x}", start, end, addr);
                //注意左开右闭
                start <= addr && addr < end
            },
            None => false
        }
    }
    pub fn is_on_max_stack(&self, addr: VirtAddr) -> bool {
        // FIXME: check if the address is on the stack
        match self.max_stack_segment{
            Some(range) => {
                let addr = addr.as_u64();
                let start = range.start.start_address().as_u64();
                let end = range.end.start_address().as_u64();
                trace!("stack range of current process: [{:#x} , {:#x}), addr = {:#x}", start, end, addr);
                //注意左开右闭
                start <= addr && addr < end
            },
            None => false
        }
    }
    pub fn read(&self, fd: u8, buf: &mut [u8]) -> isize {
        self.resources.read().read(fd, buf)
    }

    pub fn write(&self, fd: u8, buf: &[u8]) -> isize {
        self.resources.write().write(fd, buf)
    }

    pub fn new_sem(&mut self, key:u32, value:usize) -> bool{
        self.semaphores.write().insert(key, value)
    }
    
    pub fn remove_sem(&mut self, key:u32) -> bool{
        self.semaphores.write().remove(key)
    }

    pub fn sem_signal(&mut self, key:u32) -> SemaphoreResult{
        self.semaphores.write().signal(key)
    }

    pub fn sem_wait(&mut self, key:u32, pid: ProcessId) -> SemaphoreResult{
        self.semaphores.write().wait(key, pid)
    }
}
