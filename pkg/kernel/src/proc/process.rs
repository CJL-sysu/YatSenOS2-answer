use super::*;
use crate::memory::*;
use alloc::sync::{Arc, Weak};
use alloc::vec::Vec;
use spin::*;
use x86_64::structures::paging::mapper::MapToError;
use x86_64::structures::paging::page::PageRange;
use x86_64::structures::paging::*;
use elf::*;
use core::intrinsics::copy_nonoverlapping;
use core::arch::asm;
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

        inner.kill(self.pid, ret);
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
    pub fn fork(self: &Arc<Self>) -> Arc<Self> {
        // FIXME: lock inner as write
        let mut inner = self.inner.write();
        // FIXME: inner fork with parent weak ref
        let child_inner = inner.fork(Arc::downgrade(self));
        let child_pid = ProcessId::new();
        // FOR DBG: maybe print the child process info
        //          e.g. parent, name, pid, etc.
        debug!("fork: parrent is {}#{}, child is {}#{}.", inner.name, self.pid, child_inner.name, child_pid);
        // FIXME: make the arc of child
        let child = Arc::new(Self {
            pid: child_pid,
            inner: Arc::new(RwLock::new(child_inner)),
        });
        // FIXME: add child to current process's children list
        inner.children.push(child.clone());
        // FIXME: set fork ret value for parent with `context.set_rax`
        inner.context.set_rax(child.pid.0 as usize);
        
        // FIXME: mark the child as ready & return it
        inner.pause(); // !!!!!!!! actually, mark the parent as ready here
        child
    }
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

    pub fn block(&mut self){
        self.status = ProgramStatus::Blocked;
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

    pub fn kill(&mut self, pid: ProcessId, ret: isize) {
        // FIXME: set exit code
        self.exit_code = Some(ret);
        // FIXME: set status to dead
        self.status = ProgramStatus::Dead;
        // info!("{:?}",self.status);
        // FIXME: take and drop unused resources
        self.clean_up_stack(pid);
        self.proc_data.take();
        self.page_table.take();
    }

    fn clean_up_stack(&mut self, pid: ProcessId) {
        let page_table = self.page_table.take().unwrap();
        let mut mapper = page_table.mapper();

        let frame_deallocator = &mut *get_frame_alloc_for_sure();
        let start_count = frame_deallocator.recycled_count();

        let proc_data = self.proc_data.as_mut().unwrap();
        let stack = proc_data.stack_segment.unwrap();

        trace!(
            "Free stack for {}#{}: [{:#x} -> {:#x}) ({} frames)",
            self.name,
            pid,
            stack.start.start_address(),
            stack.end.start_address(),
            stack.count()
        );

        elf::unmap_range(
            stack.start.start_address().as_u64(),
            stack.count() as u64,
            &mut mapper,
            frame_deallocator,
            true,
        )
        .unwrap();

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
        map_range(addr.as_u64() as u64, stack_size as u64, page_table, frame_allocator, true, false).unwrap();
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

    pub fn fork(&mut self, parent: Weak<Process>) -> ProcessInner {
        // FIXME: get current process's stack info
        let stack_info = self.stack_segment.unwrap();
        // FIXME: clone the process data struct
        let child_proc_data = self.proc_data.clone().unwrap();
        // FIXME: clone the page table context (see instructions)
        let child_page_table = self.page_table.as_ref().unwrap().fork();
        // FIXME: alloc & map new stack for child (see instructions)
        let frame_allocator = &mut *get_frame_alloc_for_sure();
        let mapper = &mut self.page_table.as_ref().unwrap().mapper();
        let parent_stack_base = stack_info.start.start_address().as_u64();
        let parent_stack_top = stack_info.end.start_address().as_u64();
        let mut child_stack_base = parent_stack_base - (self.children.len() as u64 + 1)* STACK_MAX_SIZE;
        while elf::map_range(child_stack_base, stack_info.count() as u64, mapper, frame_allocator, true, true).is_err(){
            trace!("Map thread stack to {:#x} failed.", child_stack_base);
            child_stack_base -= STACK_MAX_SIZE;
        };
        trace!("map child stack to {:#x} succeed", child_stack_base);
        // FIXME: copy the *entire stack* from parent to child
        ProcessInner::clone_range(parent_stack_base, child_stack_base, stack_info.count());
        //debug!("finished clone range");
        // FIXME: update child's context with new *stack pointer*
        //          > update child's stack to new base
        let mut child_context = self.context;
        child_context.set_stack_offset(child_stack_base - parent_stack_base);
        //          > keep lower bits of *rsp*, update the higher bits
        // let mut old_rsp:u64 = 0;
        // unsafe{
        //     //寄存器rsp的值赋值给get_rsp
        //     asm!("mov {}, rsp", out(reg) old_rsp);
        // }
        // let mut new_rsp = old_rsp - (parent_stack_base - child_stack_base);
        // unsafe{
        //     asm!("mov rsp, {}", in(reg) new_rsp);
        // }
        // debug!("update rsp from {:#x} to {:#x}",old_rsp, new_rsp);

        //          > also update the stack record in process data
        let mut child_proc_data = self.proc_data.clone().unwrap();
        let child_stack_top = child_stack_base + stack_info.count() as u64 * Size4KiB::SIZE;
        let child_stack = Page::range(
            Page::containing_address(VirtAddr::new_truncate(child_stack_base)),
            Page::containing_address(VirtAddr::new_truncate(child_stack_top))
        );
        child_proc_data.stack_segment = Some(child_stack);
        child_proc_data.set_max_stack(VirtAddr::new(child_stack_top-STACK_MAX_SIZE), STACK_MAX_PAGES);
        // FIXME: set the return value 0 for child with `context.set_rax`
        child_context.set_rax(0);
        // FIXME: construct the child process inner
        Self { 
            name: self.name.clone(),
            parent: Some(parent),
            children: Vec::new(), 
            ticks_passed: 0,
            status: ProgramStatus::Ready,
            exit_code: None,
            context: child_context,
            page_table: Some(child_page_table),
            proc_data: Some(child_proc_data) 
        }
        // NOTE: return inner because there's no pid record in inner
    }
    /// Clone a range of memory
    ///
    /// - `src_addr`: the address of the source memory
    /// - `dest_addr`: the address of the target memory
    /// - `size`: the count of pages to be cloned
    fn clone_range(src_addr: u64, dest_addr: u64, size: usize) {
        trace!("Clone range: {:#x} -> {:#x}", src_addr, dest_addr);
        unsafe {
            copy_nonoverlapping::<u8>(
                src_addr as *mut u8,
                dest_addr as *mut u8,
                size * Size4KiB::SIZE as usize,
            );
        }
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
