use core::alloc::Layout;

use crate::filesystem::read_file;
use crate::interrupt::clock;
use crate::interrupt::new_sem;
use crate::interrupt::remove_sem;
use crate::interrupt::sem_signal;
use crate::interrupt::sem_wait;
use crate::proc;
use crate::proc::*;
use crate::utils::*;

use super::SyscallArgs;

pub fn spawn_process(args: &SyscallArgs) -> usize {
    // FIXME: get app name by args
    //       - core::str::from_utf8_unchecked
    //       - core::slice::from_raw_parts
    // FIXME: spawn the process by name
    // FIXME: handle spawn error, return 0 if failed
    // FIXME: return pid as usize
    let buf: &[u8];
    unsafe{
        buf = core::slice::from_raw_parts(args.arg0 as *const u8, args.arg1 as usize);
    }
    let name;
    unsafe {
        name = core::str::from_utf8_unchecked(buf);
    };
    //debug!("name is {}",name);
    let res = proc::spawn(name);
    match res {
        Some(pid) => pid.0 as usize,
        None => 0,
    }
}

pub fn sys_write(args: &SyscallArgs) -> usize {//arg0区分标准输出和错误输出，arg1为指针，arg2为字符串长度
    // FIXME: get buffer and fd by args
    //       - core::slice::from_raw_parts
    //info!("at sys_write");
    let buf: &[u8];
    unsafe{
        buf = core::slice::from_raw_parts(args.arg1 as *const u8, args.arg2 as usize);
    }
    // FIXME: call proc::write -> isize
    //let res = proc::write(args.syscall.clone() as u8, buf);
    let res = proc::write(args.arg0 as u8, buf);
    // FIXME: return the result as usize
    res as usize
}

pub fn sys_read(args: &SyscallArgs) -> usize {
    // FIXME: just like sys_write
    let buf;
    unsafe{
        buf = core::slice::from_raw_parts_mut(args.arg1 as *mut u8, args.arg2 as usize);
    };
    //let res = proc::read(args.syscall.clone() as u8, buf);
    let res = proc::read(args.arg0 as u8, buf);
    res as usize
}

pub fn exit_process(args: &SyscallArgs, context: &mut ProcessContext) {
    // FIXME: exit process with retcode
    proc::exit(args.arg0 as isize, context)
}

pub fn list_process() {
    // FIXME: list all processes
    proc::print_process_list();
}

pub fn sys_allocate(args: &SyscallArgs) -> usize {
    let layout = unsafe { (args.arg0 as *const Layout).as_ref().unwrap() };

    if layout.size() == 0 {
        return 0;
    }

    let ret = crate::memory::user::USER_ALLOCATOR
        .lock()
        .allocate_first_fit(*layout);

    match ret {
        Ok(ptr) => ptr.as_ptr() as usize,
        Err(_) => 0,
    }
}

pub fn sys_deallocate(args: &SyscallArgs) {
    let layout = unsafe { (args.arg1 as *const Layout).as_ref().unwrap() };

    if args.arg0 == 0 || layout.size() == 0 {
        return;
    }

    let ptr = args.arg0 as *mut u8;

    unsafe {
        crate::memory::user::USER_ALLOCATOR
            .lock()
            .deallocate(core::ptr::NonNull::new_unchecked(ptr), *layout);
    }
}

pub fn get_pid() -> u16{
    get_process_manager().current().pid().0
}
pub fn service_list_app(){
    proc::list_app();
}
pub fn wait_service_pid(args: &SyscallArgs, context: &mut ProcessContext){
    proc::wait_pid(ProcessId(args.arg0.try_into().unwrap()), context)
}
pub fn sys_clock() -> i64{
    clock::now().timestamp_nanos_opt().unwrap_or_default()
}
pub fn sys_fork(context: &mut ProcessContext){
    proc::fork(context);
}
pub fn sys_sem(args: &SyscallArgs, context: &mut ProcessContext){
    match args.arg0 {
        0 => context.set_rax(new_sem(args.arg1 as u32, args.arg2)),
        1 => context.set_rax(remove_sem(args.arg1 as u32)),
        2 => sem_signal(args.arg1 as u32, context),
        3 => sem_wait(args.arg1 as u32, context),
        _ => context.set_rax(usize::MAX),
    }
}
pub fn sys_list_dir(args: &SyscallArgs){
    let root = unsafe {
        core::str::from_utf8_unchecked(core::slice::from_raw_parts(
            args.arg0 as *const u8,
            args.arg1,
        ))
    };
    crate::drivers::filesystem::ls(root);
}
pub fn sys_cat(args: &SyscallArgs) -> usize{
    let root = unsafe {
        core::str::from_utf8_unchecked(core::slice::from_raw_parts(
            args.arg0 as *const u8,
            args.arg1,
        ))
    };
    let mut buf;
    unsafe{
        buf = core::slice::from_raw_parts_mut(args.arg2 as *mut u8, 4096);
    };
    let ret = read_file(root, &mut buf);
    match ret{
        Ok(len) => len,
        Err(e) => {
            warn!("{:#?}",e);
            0
        }
    }
}
pub fn sys_spawn_file(args: &SyscallArgs) -> usize{
    let buf: &[u8];
    unsafe{
        buf = core::slice::from_raw_parts(args.arg0 as *const u8, args.arg1 as usize);
    }
    let path;
    unsafe {
        path = core::str::from_utf8_unchecked(buf);
    };
    //debug!("name is {}",name);
    let res = proc::spawn_from_file(path);
    match res {
        Some(pid) => pid.0 as usize,
        None => 0,
    }
}