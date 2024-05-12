#![no_std]
#![no_main]

use log::debug;
use log::info;
use ysos::*;
use ysos_kernel as ysos;
use syscall_def::syscall;
use syscall_def::Syscall;
extern crate alloc;

boot::entry_point!(kernel_main);

pub fn kernel_main(boot_info: &'static boot::BootInfo) -> ! {
    ysos::init(boot_info);
    //syscall!(Syscall::Exit, 0);
    //syscall!(Syscall::Stat);
    wait(spawn_init());
    ysos::shutdown(boot_info);
}

pub fn spawn_init() -> proc::ProcessId {
    proc::spawn("shell").unwrap()
}
