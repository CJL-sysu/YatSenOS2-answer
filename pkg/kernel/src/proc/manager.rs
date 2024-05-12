use self::processor::set_pid;
use crate::{memory::{user::{USER_ALLOCATOR, USER_HEAP_SIZE}, PHYSICAL_OFFSET}, proc::context::*};
use super::*;
use crate::memory::{
    self,
    allocator::{ALLOCATOR, HEAP_SIZE},
    get_frame_alloc_for_sure, PAGE_SIZE,
};
use alloc::{boxed::Box, collections::*, format, sync::{Arc, Weak}};
use boot::{AppList, AppListRef};
use spin::{Mutex, RwLock, RwLockWriteGuard};
use x86::current;

pub static PROCESS_MANAGER: spin::Once<ProcessManager> = spin::Once::new();

pub fn init(init: Arc<Process>, app_list:AppListRef) {

    // FIXME: set init process as Running
    init.write().resume();
    // FIXME: set processor's current pid to init's pid
    processor::set_pid(init.pid());
    PROCESS_MANAGER.call_once(|| ProcessManager::new(init, app_list));
}

pub fn get_process_manager() -> &'static ProcessManager {
    PROCESS_MANAGER
        .get()
        .expect("Process Manager has not been initialized")
}

pub struct ProcessManager {
    processes: RwLock<BTreeMap<ProcessId, Arc<Process>>>,
    ready_queue: Mutex<VecDeque<ProcessId>>,
    app_list: boot::AppListRef,
}

impl ProcessManager {
    pub fn app_list(&self) -> AppListRef {
        self.app_list
    }
    pub fn new(init: Arc<Process>, app_list:AppListRef) -> Self {
        let mut processes = BTreeMap::new();
        let ready_queue = VecDeque::new();
        let pid = init.pid();

        trace!("Init {:#?}", init);

        processes.insert(pid, init);
        Self {
            processes: RwLock::new(processes),
            ready_queue: Mutex::new(ready_queue),
            app_list: app_list,
        }
    }
    pub fn get_exit_code(&self, pid: &ProcessId) -> Option<i32> {
        x86_64::instructions::interrupts::without_interrupts(|| { //这里必须使用without_interrupts,防止出现死锁(折磨了CJL一下午)
            match self.get_proc(&pid){
                Some(proc) => {
                    match proc.read().exit_code(){
                        Some(code) => {
                            Some(code as i32)
                        },
                        None => {
                            None
                        }
                    }
                },
                None => None
            }
        })
    }
    #[inline]
    pub fn push_ready(&self, pid: ProcessId) {
        self.ready_queue.lock().push_back(pid);
    }

    #[inline]
    fn add_proc(&self, pid: ProcessId, proc: Arc<Process>) {
        self.processes.write().insert(pid, proc);
    }

    #[inline]
    fn get_proc(&self, pid: &ProcessId) -> Option<Arc<Process>> {
        self.processes.read().get(pid).cloned()
    }

    pub fn current(&self) -> Arc<Process> {
        self.get_proc(&processor::get_pid())
            .expect("No current process")
    }

    pub fn save_current(&self, context: &ProcessContext) {
        // FIXME: update current process's tick count
        let proc = self.current();
        proc.write().tick();
        // FIXME: save current process's context
        proc.write().save(context);
    }

    pub fn switch_next(&self, context: &mut ProcessContext) -> ProcessId {

        // FIXME: fetch the next process from ready queue
        loop{
            if let Some(pid) = self.ready_queue.lock().pop_front() {
                if let Some(proc) = self.get_proc(&pid) {
                    // FIXME: check if the next process is ready,
                    if proc.read().status() == ProgramStatus::Ready {
                        // FIXME: restore next process's context
                        proc.write().restore(context);
                        // FIXME: update processor's current pid
                        set_pid(proc.pid());
                        // FIXME: return next process's pid
                        return proc.pid();
                    }
                }
            }else {
                return get_pid()
            }
        }
    }
    // //from lab4, delete here in lab5
    // pub fn spawn_kernel_thread(
    //     &self,
    //     entry: VirtAddr,
    //     name: String,
    //     proc_data: Option<ProcessData>,
    // ) -> ProcessId {
    //     let kproc = self.get_proc(&KERNEL_PID).unwrap();
    //     let page_table = kproc.read().clone_page_table();
    //     let proc = Process::new(name, Some(Arc::downgrade(&kproc)), page_table, proc_data);
    //     let pid = proc.pid();
    //     // alloc stack for the new process base on pid
    //     let stack_top = proc.alloc_init_stack();
    //     proc.write().pause();
    //     // FIXME: set the stack frame
    //     proc.write().set_stack_frame(entry, stack_top);
    //     // FIXME: add to process map
    //     self.add_proc(pid, proc);
    //     // FIXME: push to ready queue
    //     self.push_ready(pid);
    //     // FIXME: return new process pid
    //     pid
    // }

    pub fn kill_current(&self, ret: isize) {
        self.kill(processor::get_pid(), ret);
    }

    pub fn handle_page_fault(&self, addr: VirtAddr, err_code: PageFaultErrorCode) -> bool {
        // FIXME: handle page fault
        if err_code.contains(PageFaultErrorCode::PROTECTION_VIOLATION){
            info!("PROTECTION_VIOLATION caused page fault");
            false
        }else{
            let proc = self.current();
            if proc.read().is_on_max_stack(addr){
                proc.write().inc_stack_space(addr);
                true
            }else{
                info!("the addr {:#?} is not in the stack of current process", addr);
                false
            }
        }
    }
    
    pub fn kill_self(&self, ret: isize) {
        self.kill(processor::get_pid(), ret);
    }

    pub fn kill(&self, pid: ProcessId, ret: isize) {
        let proc = self.get_proc(&pid);

        if proc.is_none() {
            warn!("Process #{} not found.", pid);
            return;
        }

        let proc = proc.unwrap();

        if proc.read().status() == ProgramStatus::Dead {
            warn!("Process #{} is already dead.", pid);
            return;
        }

        trace!("Kill {:#?}", &proc);

        proc.kill(ret);
    }

    pub fn print_process_list(&self) {
        let mut output = String::from("  PID | PPID | Process Name |  Ticks  | Status\n");

        self.processes
            .read()
            .values()
            .filter(|p| p.read().status() != ProgramStatus::Dead)
            .for_each(|p| output += format!("{}\n", p).as_str());

        // TODO: print memory usage of kernel heap
        let heap_used = ALLOCATOR.lock().used();
        let heap_size = HEAP_SIZE;

        let user_heap_used = USER_ALLOCATOR.lock().used();
        let user_heap_size = USER_HEAP_SIZE;

        let alloc = get_frame_alloc_for_sure();
        let frames_used = alloc.frames_used();
        let frames_recycled = alloc.recycled_count();
        let frames_total = alloc.frames_total();

        let (sys_used, sys_used_unit) = crate::humanized_size(heap_used as u64);
        let (sys_size, sys_size_unit) = crate::humanized_size(heap_size as u64);

        output += format!(
            "Kernel : {:>6.*} {} / {:>6.*} {} ({:>5.2}%)\n",
            2,
            sys_used,
            sys_used_unit,
            2,
            sys_size,
            sys_size_unit,
            heap_used as f64 / heap_size as f64 * 100.0
        )
        .as_str();

        let (user_used, user_used_unit) = crate::humanized_size(user_heap_used as u64);
        let (user_size, user_size_unit) = crate::humanized_size(user_heap_size as u64);

        output += format!(
            "User   : {:>6.*} {} / {:>6.*} {} ({:>5.2}%)\n",
            2,
            user_used,
            user_used_unit,
            2,
            user_size,
            user_size_unit,
            user_heap_used as f64 / user_heap_size as f64 * 100.0
        )
        .as_str();

        // put used/total frames in MiB
        let (used_size, used_unit) =
            crate::humanized_size((frames_used - frames_recycled) as u64 * PAGE_SIZE);
        let (tot_size, tot_unit) = crate::humanized_size(frames_total as u64 * PAGE_SIZE);

        output += format!(
            "Memory : {:>6.*} {} / {:>6.*} {} ({:>5.2}%) [{} recycled]\n",
            2,
            used_size,
            used_unit,
            2,
            tot_size,
            tot_unit,
            (frames_used - frames_recycled) as f64 / frames_total as f64 * 100.0,
            frames_recycled
        )
        .as_str();
    
        output += format!("Queue  : {:?}\n", self.ready_queue.lock()).as_str();

        output += &processor::print_processors();

        print!("{}", output);
    }
    pub fn spawn(
        &self,
        elf: &ElfFile,
        name: String,
        parent: Option<Weak<Process>>,
        proc_data: Option<ProcessData>,
    ) -> ProcessId {
        let kproc = self.get_proc(&KERNEL_PID).unwrap();
        let page_table = kproc.read().clone_page_table();
        let page_table_mapper: x86_64::structures::paging::OffsetPageTable<'static> = page_table.mapper();
        let proc = Process::new(name, parent, page_table, proc_data);
        let pid = proc.pid();

        let mut inner = proc.write();
        // FIXME: load elf to process pagetable
        inner.load_elf(elf, page_table_mapper);
        // FIXME: alloc new stack for process
        
        inner.set_stack_frame(VirtAddr::new_truncate(elf.header.pt2.entry_point()), VirtAddr::new_truncate(STACK_INIT_TOP));
        // FIXME: mark process as ready
        inner.pause();
        drop(inner);

        trace!("New {:#?}", &proc);

        // FIXME: something like kernel thread
        self.add_proc(pid, proc);
        self.push_ready(pid);
        pid
    }

    pub fn fork(&self) {
        // FIXME: get current process
        let current = self.current();
        // FIXME: fork to get child
        let child = current.fork();
        // FIXME: add child to process list
        let child_pid = child.pid();
        self.add_proc(child_pid, child);
        self.push_ready(child_pid);
        // FOR DBG: maybe print the process ready queue?
        debug!("Ready queue: {:?}", self.ready_queue.lock());
    }
}
