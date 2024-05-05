use super::*;
use crate::memory::*;
use alloc::sync::{Arc, Weak};
use alloc::vec::Vec;
use spin::*;
use x86_64::structures::paging::mapper::MapToError;
use x86_64::structures::paging::page::PageRange;
use x86_64::structures::paging::*;
use elf::*;
#[derive(Clone)]
pub struct Process {
    pid: ProcessId,
    inner: Arc<RwLock<ProcessInner>>,
}

pub struct ProcessInner {
    name: String,
    parent: Option<Weak<Process>>,
    children: Vec<Arc<Process>>,
    ticks_passed: usize,
    status: ProgramStatus,
    exit_code: Option<isize>,
    context: ProcessContext,
    page_table: Option<PageTableContext>,
    proc_data: Option<ProcessData>,
}

impl Process {
    #[inline]
    pub fn pid(&self) -> ProcessId {
        self.pid
    }

    #[inline]
    pub fn write(&self) -> RwLockWriteGuard<ProcessInner> {
        self.inner.write()
    }

    #[inline]
    pub fn read(&self) -> RwLockReadGuard<ProcessInner> {
        self.inner.read()
    }

    pub fn new(
        name: String,
        parent: Option<Weak<Process>>,
        page_table: PageTableContext,
        proc_data: Option<ProcessData>,
    ) -> Arc<Self> {
        let name = name.to_ascii_lowercase();

        // create context
        let pid = ProcessId::new();

        let inner = ProcessInner {
            name,
            parent,
            status: ProgramStatus::Ready,
            context: ProcessContext::default(),
            ticks_passed: 0,
            exit_code: None,
            children: Vec::new(),
            page_table: Some(page_table),
            proc_data: Some(proc_data.unwrap_or_default()),
        };

        trace!("New process {}#{} created.", &inner.name, pid);

        // create process struct
        Arc::new(Self {
            pid,
            inner: Arc::new(RwLock::new(inner)),
        })
    }

    pub fn kill(&self, ret: isize) {
        let mut inner = self.inner.write();

        debug!(
            "Killing process {}#{} with ret code: {}",
            inner.name(),
            self.pid,
            ret
        );

        inner.kill(ret);
    }
    // // from lab4, delete in lab5
    //
    // pub fn alloc_init_stack(&self) -> VirtAddr {
    //     // FIXME: alloc init stack base on self pid
    //     // let stack_top:u64 = 0x400000000000 - (self.pid().0 as u64 - 1)*0x100000000;
    //     // let stack_bottom:u64 = 0x400000000000 - (self.pid().0 as u64)*0x100000000;
    //     // let stack_size: u64 = 0x100000;
    //     let stack_bottom: u64 = STACK_INIT_BOT - (self.pid().0 as u64 - 1)*0x100000000;
    //     let stack_top: u64 = STACK_INIT_TOP - (self.pid().0 as u64 - 1)*0x100000000;
    //     let max_stack_bottom: u64 = STACK_MAX - (self.pid().0 as u64)*0x100000000;
    //     let max_stack_size: u64 = STACK_MAX_PAGES;
    //     let stack_size: u64 = 1;
    //     let page_table = &mut self.inner.read().page_table.as_ref().unwrap().mapper();
    //     let frame_allocator = &mut *get_frame_alloc_for_sure();
    //     map_range(stack_bottom as u64, stack_size as u64, page_table, frame_allocator).unwrap();
    //     self.inner.write().proc_data.as_mut().unwrap().set_stack(VirtAddr::new(stack_top), stack_size);
    //     self.inner.write().proc_data.as_mut().unwrap().set_max_stack(VirtAddr::new(max_stack_bottom), max_stack_size);
    //     VirtAddr::new(stack_top as u64)
    // }
    // 
}

impl ProcessInner {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn tick(&mut self) {
        self.ticks_passed += 1;
    }

    pub fn status(&self) -> ProgramStatus {
        self.status
    }

    pub fn pause(&mut self) {
        self.status = ProgramStatus::Ready;
    }

    pub fn resume(&mut self) {
        self.status = ProgramStatus::Running;
    }

    pub fn exit_code(&self) -> Option<isize> {
        self.exit_code
    }

    pub fn clone_page_table(&self) -> PageTableContext {
        self.page_table.as_ref().unwrap().clone_l4()
    }

    pub fn is_ready(&self) -> bool {
        self.status == ProgramStatus::Ready
    }

    /// Save the process's context
    /// mark the process as ready
    pub(super) fn save(&mut self, context: &ProcessContext) {
        // FIXME: save the process's context
        if self.status != ProgramStatus::Running && self.status != ProgramStatus::Ready{
            return;
        }
        self.context.save(context);
        self.pause();
    }

    /// Restore the process's context
    /// mark the process as running
    pub(super) fn restore(&mut self, context: &mut ProcessContext) {
        
        // FIXME: restore the process's context
        self.context.restore(context);
        // FIXME: restore the process's page table
        self.page_table.as_ref().unwrap().load();
        self.resume();
    }

    pub fn parent(&self) -> Option<Arc<Process>> {
        self.parent.as_ref().and_then(|p| p.upgrade())
    }

    pub fn kill(&mut self, ret: isize) {
        // FIXME: set exit code
        self.exit_code = Some(ret);
        // FIXME: set status to dead
        self.status = ProgramStatus::Dead;
        // info!("{:?}",self.status);
        // FIXME: take and drop unused resources
        self.proc_data.take();
        self.page_table.take();
    }
    pub fn set_stack_frame(&mut self, entry: VirtAddr, stack_top: VirtAddr){
        self.context.init_stack_frame(entry, stack_top);
    }

    pub fn inc_stack_space(&mut self, addr: VirtAddr){
        // 分配新的页面
        let old_page_range = self.proc_data.as_ref()
            .expect("Failed to get proc_data")
            .stack_segment
            .expect("Failed to get page range");
        let old_start_page = old_page_range.start;
        let old_end_page = old_page_range.end;

        let cur_start_page = Page::<Size4KiB>::containing_address(addr);
        let stack_size = old_start_page - cur_start_page;
        let page_table = &mut self.page_table.as_ref().unwrap().mapper();
        let frame_allocator = &mut *get_frame_alloc_for_sure();
        map_range(addr.as_u64() as u64, stack_size as u64, page_table, frame_allocator, false, false).unwrap();
        // 更新页表
        self.page_table.as_ref().unwrap().load();
        // 更新进程数据中的栈信息
        self.proc_data.as_mut().unwrap().set_stack(addr, old_end_page - cur_start_page);
        trace!("increase stack space successfully");
    }

    pub fn load_elf(&mut self, elf:&ElfFile, mut mapper: x86_64::structures::paging::OffsetPageTable<'static>){
        let alloc = &mut *get_frame_alloc_for_sure();
        elf::load_elf(
            elf, 
            *PHYSICAL_OFFSET.get().unwrap(), 
            &mut mapper, 
            alloc, 
            true
        ).unwrap();
        let stack_segment = elf::map_range(STACK_INIT_BOT, STACK_DEF_PAGE, &mut mapper, alloc, true, false).unwrap();
        //info!("stack segment: {:?}", stack_segment);
        self.proc_data.as_mut().unwrap().set_stack(VirtAddr::new(STACK_INIT_TOP), STACK_DEF_PAGE);
        self.proc_data.as_mut().unwrap().set_max_stack(VirtAddr::new(STACK_MAX - STACK_MAX_SIZE), STACK_MAX_PAGES);
    }
}

impl core::ops::Deref for Process {
    type Target = Arc<RwLock<ProcessInner>>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl core::ops::Deref for ProcessInner {
    type Target = ProcessData;

    fn deref(&self) -> &Self::Target {
        self.proc_data
            .as_ref()
            .expect("Process data empty. The process may be killed.")
    }
}

impl core::ops::DerefMut for ProcessInner {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.proc_data
            .as_mut()
            .expect("Process data empty. The process may be killed.")
    }
}

impl core::fmt::Debug for Process {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        let mut f = f.debug_struct("Process");
        f.field("pid", &self.pid);

        let inner = self.inner.read();
        f.field("name", &inner.name);
        f.field("parent", &inner.parent().map(|p| p.pid));
        f.field("status", &inner.status);
        f.field("ticks_passed", &inner.ticks_passed);
        f.field(
            "children",
            &inner.children.iter().map(|c| c.pid.0).collect::<Vec<u16>>(),
        );
        f.field("page_table", &inner.page_table);
        f.field("status", &inner.status);
        f.field("context", &inner.context);
        f.field("stack", &inner.proc_data.as_ref().map(|d| d.stack_segment));
        f.finish()
    }
}

impl core::fmt::Display for Process {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        let inner = self.inner.read();
        write!(
            f,
            " #{:-3} | #{:-3} | {:12} | {:7} | {:?}",
            self.pid.0,
            inner.parent().map(|p| p.pid.0).unwrap_or(0),
            inner.name,
            inner.ticks_passed,
            inner.status
        )?;
        Ok(())
    }
}
