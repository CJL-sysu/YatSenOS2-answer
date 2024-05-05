use core::alloc::Layout;

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

    0
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
pub fn wait_pid(args: &SyscallArgs){
    get_process_manager().get_exit_code(&ProcessId(args.arg0 as u16));
}