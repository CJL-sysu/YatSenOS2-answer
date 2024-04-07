use self::processor::set_pid;
use crate::proc::context::*;
use super::*;
use crate::memory::{
    self,
    allocator::{ALLOCATOR, HEAP_SIZE},
    get_frame_alloc_for_sure, PAGE_SIZE,
};
use alloc::{collections::*, format, sync::Arc};
use spin::{Mutex, RwLock, RwLockWriteGuard};

pub static PROCESS_MANAGER: spin::Once<ProcessManager> = spin::Once::new();

pub fn init(init: Arc<Process>) {

    // FIXME: set init process as Running
    init.write().resume();
    // FIXME: set processor's current pid to init's pid
    processor::set_pid(init.pid());
    PROCESS_MANAGER.call_once(|| ProcessManager::new(init));
}

pub fn get_process_manager() -> &'static ProcessManager {
    PROCESS_MANAGER
        .get()
        .expect("Process Manager has not been initialized")
}

pub struct ProcessManager {
    processes: RwLock<BTreeMap<ProcessId, Arc<Process>>>,
    ready_queue: Mutex<VecDeque<ProcessId>>,
}

impl ProcessManager {
    pub fn new(init: Arc<Process>) -> Self {
        let mut processes = BTreeMap::new();
        let ready_queue = VecDeque::new();
        let pid = init.pid();

        trace!("Init {:#?}", init);

        processes.insert(pid, init);
        Self {
            processes: RwLock::new(processes),
            ready_queue: Mutex::new(ready_queue),
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

    pub fn spawn_kernel_thread(
        &self,
        entry: VirtAddr,
        name: String,
        proc_data: Option<ProcessData>,
    ) -> ProcessId {
        let kproc = self.get_proc(&KERNEL_PID).unwrap();
        let page_table = kproc.read().clone_page_table();
        let proc = Process::new(name, Some(Arc::downgrade(&kproc)), page_table, proc_data);
        let pid = proc.pid();
        // alloc stack for the new process base on pid
        let stack_top = proc.alloc_init_stack();
        proc.write().pause();
        // FIXME: set the stack frame
        proc.write().set_stack_frame(entry, stack_top);
        // FIXME: add to process map
        self.add_proc(pid, proc);
        // FIXME: push to ready queue
        self.push_ready(pid);
        // FIXME: return new process pid
        pid
    }

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

        output += format!("Queue  : {:?}\n", self.ready_queue.lock()).as_str();

        output += &processor::print_processors();

        print!("{}", output);
    }
}
