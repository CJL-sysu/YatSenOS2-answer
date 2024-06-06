use chrono::NaiveDateTime;
use syscall_def::Syscall;

#[inline(always)]
pub fn sys_write(fd: u8, buf: &[u8]) -> Option<usize> {
    let ret = syscall!(
        Syscall::Write,
        fd as u64,
        buf.as_ptr() as u64,
        buf.len() as u64
    ) as isize;
    if ret.is_negative() {
        None
    } else {
        Some(ret as usize)
    }
}

#[inline(always)]
pub fn sys_read(fd: u8, buf: &mut [u8]) -> Option<usize> {
    let ret = syscall!(
        Syscall::Read,
        fd as u64,
        buf.as_ptr() as u64,
        buf.len() as u64
    ) as isize;
    if ret.is_negative() {
        None
    } else {
        Some(ret as usize)
    }
}

#[inline(always)]
pub fn sys_wait_pid(pid: u16) -> isize {
    // FIXME: try to get the return value for process
    //        loop until the process is finished
    let status: isize = syscall!(Syscall::WaitPid, pid as u64) as isize;
    // loop {
    //     status = syscall!(Syscall::WaitPid, pid as u64) as isize;
    //     if status != 114514 {
    //         // Why? Check reflection question 5
    //         //x86_64::instructions::hlt();
    //         break;
    //     }
    // }
    status
}

#[inline(always)]
pub fn sys_list_app() {
    syscall!(Syscall::ListApp);
}

#[inline(always)]
pub fn sys_stat() {
    syscall!(Syscall::Stat);
}

#[inline(always)]
pub fn sys_allocate(layout: &core::alloc::Layout) -> *mut u8 {
    syscall!(Syscall::Allocate, layout as *const _) as *mut u8
}

#[inline(always)]
pub fn sys_deallocate(ptr: *mut u8, layout: &core::alloc::Layout) -> usize {
    syscall!(Syscall::Deallocate, ptr, layout as *const _)
}

#[inline(always)]
pub fn sys_spawn(path: &str) -> u16 {
    syscall!(Syscall::Spawn, path.as_ptr() as u64, path.len() as u64) as u16
}

#[inline(always)]
pub fn sys_get_pid() -> u16 {
    syscall!(Syscall::GetPid) as u16
}

#[inline(always)]
pub fn sys_exit(code: isize) -> ! {
    syscall!(Syscall::Exit, code as u64);
    unreachable!("This process should be terminated by now.")
}
#[inline(always)]
pub fn sys_time() -> NaiveDateTime {
    let time = syscall!(Syscall::Time) as i64;
    const BILLION: i64 = 1_000_000_000;
    NaiveDateTime::from_timestamp_opt(time / BILLION, (time % BILLION) as u32).unwrap_or_default()
}
#[inline(always)]
pub fn sys_fork() -> u16 {
    syscall!(Syscall::Fork) as u16
}
#[inline(always)]
pub fn sys_new_sem(key: u32, value: usize) -> usize {
    syscall!(Syscall::Sem, 0, key, value)
}
#[inline(always)]
pub fn sys_del_sem(key: u32) -> usize {
    syscall!(Syscall::Sem, 1, key)
}
#[inline(always)]
pub fn sys_signal(key: u32) -> usize{
    syscall!(Syscall::Sem, 2, key)
}
#[inline(always)]
pub fn sys_wait(key: u32) -> usize{
    syscall!(Syscall::Sem, 3, key)
}
#[inline(always)]
pub fn sys_list_dir(root: &str) {
    syscall!(Syscall::ListDir, root.as_ptr() as u64, root.len() as u64);
}
#[inline(always)]
pub fn sys_cat(root: &str, buf: &mut [u8])-> usize {
    syscall!(Syscall::Cat, root.as_ptr() as u64, root.len() as u64, buf.as_ptr() as u64)
}