mod context;
mod data;
mod manager;
mod paging;
mod pid;
mod process;
mod processor;
pub mod sync;

use alloc::sync::Arc;
use alloc::vec::Vec;
pub use manager::*;
use process::*;
use x86_64::structures::paging::PageTable;
use crate::filesystem::read_all_file;
use crate::memory::PAGE_SIZE;
use xmas_elf::ElfFile;
use alloc::string::{String, ToString};
pub use context::ProcessContext;
pub use paging::PageTableContext;
pub use data::ProcessData;
pub use pid::ProcessId;

use x86_64::structures::idt::PageFaultErrorCode;
use x86_64::VirtAddr;

use self::processor::get_pid;

// 0xffff_ff00_0000_0000 is the kernel's address space
pub const STACK_MAX: u64 = 0x0000_4000_0000_0000;

pub const STACK_MAX_PAGES: u64 = 0x100000;
pub const STACK_MAX_SIZE: u64 = STACK_MAX_PAGES * PAGE_SIZE;
pub const STACK_START_MASK: u64 = !(STACK_MAX_SIZE - 1);
// [bot..0x2000_0000_0000..top..0x3fff_ffff_ffff]
// init stack
pub const STACK_DEF_PAGE: u64 = 1;
pub const STACK_DEF_SIZE: u64 = STACK_DEF_PAGE * PAGE_SIZE;
pub const STACK_INIT_BOT: u64 = STACK_MAX - STACK_DEF_SIZE;
pub const STACK_INIT_TOP: u64 = STACK_MAX - 8;
// [bot..0xffffff0100000000..top..0xffffff01ffffffff]
// kernel stack
pub const KSTACK_MAX: u64 = 0xffff_ff02_0000_0000;
pub const KSTACK_DEF_PAGE: u64 = 512;
pub const KSTACK_DEF_SIZE: u64 = KSTACK_DEF_PAGE * PAGE_SIZE;
pub const KSTACK_INIT_BOT: u64 = KSTACK_MAX - KSTACK_DEF_SIZE;
pub const KSTACK_INIT_TOP: u64 = KSTACK_MAX - 8;

pub const KERNEL_PID: ProcessId = ProcessId(1);

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ProgramStatus {
    Running,
    Ready,
    Blocked,
    Dead,
}

/// init process manager
pub fn init(boot_info: &'static boot::BootInfo) {
    let mut kproc_data = ProcessData::new();

    // FIXME: set the kernel stack
    kproc_data.set_stack(VirtAddr::new(KSTACK_INIT_BOT), KSTACK_DEF_PAGE);
    trace!("Init process data: {:#?}", kproc_data);

    // kernel process
    let kproc = Process::new(
        String::from("kernel"),
        None,
        PageTableContext::new(),
        Some(kproc_data),
    );
    let app_list = boot_info.loaded_apps.as_ref();
    manager::init(kproc, app_list);

    info!("Process Manager Initialized.");
}

pub fn switch(context: &mut ProcessContext) {
    x86_64::instructions::interrupts::without_interrupts(|| {
        // FIXME: switch to the next process
        let manager = get_process_manager();
        //      - save current process's context
        manager.save_current(context);
        //      - handle ready queue update
        manager.push_ready(get_pid());
        //      - restore next process's context
        manager.switch_next(context);
    });
}
// // from lab4, delete here in lab5
// pub fn spawn_kernel_thread(entry: fn() -> !, name: String, data: Option<ProcessData>) -> ProcessId {
//     x86_64::instructions::interrupts::without_interrupts(|| {
//         let entry = VirtAddr::new(entry as usize as u64);
//         get_process_manager().spawn_kernel_thread(entry, name, data)
//     })
// }

pub fn print_process_list() {
    x86_64::instructions::interrupts::without_interrupts(|| {
        get_process_manager().print_process_list();
    })
}

pub fn env(key: &str) -> Option<String> {
    x86_64::instructions::interrupts::without_interrupts(|| {
        // FIXME: get current process's environment variable
        get_process_manager().current().read().env(key)
    })
}

pub fn process_exit(ret: isize) -> ! {
    x86_64::instructions::interrupts::without_interrupts(|| {
        get_process_manager().kill_current(ret);
    });

    loop {
        x86_64::instructions::hlt();
    }
}

pub fn handle_page_fault(addr: VirtAddr, err_code: PageFaultErrorCode) -> bool {
    x86_64::instructions::interrupts::without_interrupts(|| {
        get_process_manager().handle_page_fault(addr, err_code)
    })
}
pub fn list_app() {
    x86_64::instructions::interrupts::without_interrupts(|| {
        let app_list = get_process_manager().app_list();
        if app_list.is_none() {
            println!("[!] No app found in list!");
            return;
        }

        let apps = app_list
            .unwrap()
            .iter()
            .map(|app| app.name.as_str())
            .collect::<Vec<&str>>()
            .join(", ");

        // TODO: print more information like size, entry point, etc.

        println!("[+] App list: {}", apps);
    });
}
pub fn spawn(name: &str) -> Option<ProcessId> {
    let app = x86_64::instructions::interrupts::without_interrupts(|| {
        let app_list = get_process_manager().app_list()?;
        app_list.iter().find(|&app| app.name.eq(name))
    })?;

    elf_spawn(name.to_string(), &app.elf)
}

pub fn spawn_from_file(path: &str) -> Option<ProcessId> {
    let buf = match read_all_file(path){
        Ok(buf) => buf,
        Err(_) => return None
    };
    let elf = match xmas_elf::ElfFile::new(&buf){
        Ok(elf) => elf,
        Err(_) => return None
    };
    let name = match path.rfind("/"){
        Some(pos) => &path[pos+1..],
        None => path
    };
    elf_spawn(name.to_string(), &elf)
}

pub fn elf_spawn(name: String, elf: &ElfFile) -> Option<ProcessId> {
    let pid = x86_64::instructions::interrupts::without_interrupts(|| {
        let manager = get_process_manager();
        let process_name = name.to_lowercase();
        let parent = Arc::downgrade(&manager.current());
        let pid = manager.spawn(elf, name, Some(parent), None);

        debug!("Spawned process: {}#{}", process_name, pid);
        pid
    });

    Some(pid)
}

pub fn read(fd: u8, buf: &mut [u8]) -> isize {
    x86_64::instructions::interrupts::without_interrupts(|| get_process_manager().current().read().read(fd, buf))
}

pub fn write(fd: u8, buf: &[u8]) -> isize {
    x86_64::instructions::interrupts::without_interrupts(|| get_process_manager().current().read().write(fd, buf))
}

pub fn exit(ret: isize, context: &mut ProcessContext) {
    x86_64::instructions::interrupts::without_interrupts(|| {
        let manager = get_process_manager();
        // FIXME: implement this for ProcessManager
        manager.kill_self(ret);
        manager.switch_next(context);
    })
}
#[inline]
pub fn still_alive(pid: ProcessId) -> bool {
    x86_64::instructions::interrupts::without_interrupts(|| {
        // check if the process is still alive
        let pid = get_process_manager().get_exit_code(&pid);
        if let None = pid {
            true
        }else {
            false
        }
    })
}

pub fn fork(context: &mut ProcessContext) {
    x86_64::instructions::interrupts::without_interrupts(|| {
        let manager = get_process_manager();
        // FIXME: save_current as parent
        let parent_pid = manager.current().pid();
        manager.save_current(context);
        // FIXME: fork to get child
        manager.fork();
        // FIXME: push to child & parent to ready queue
        manager.push_ready(parent_pid);
        // FIXME: switch to next process
        manager.switch_next(context);
    })
}
pub fn wait_pid(pid: ProcessId, context: &mut ProcessContext) {
    x86_64::instructions::interrupts::without_interrupts(|| {
        let manager = get_process_manager();
        if let Some(ret) = manager.get_exit_code(&pid) {
            context.set_rax(ret as usize);
        } else {
            manager.wait_pid(pid);
            manager.save_current(context);
            manager.current().write().block();
            manager.switch_next(context);
        }
    })
}