#![no_std]
#![no_main]

use ata::AtaDrive;
use filesystem::ls;
use filesystem::read_all_file;
use filesystem::read_file;
use filesystem::try_get_file;
use log::debug;
use log::info;
use storage::mbr::MbrTable;
use storage::PartitionTable;
use ysos::*;
use ysos_kernel as ysos;
use syscall_def::syscall;
use syscall_def::Syscall;
extern crate alloc;

boot::entry_point!(kernel_main);

pub fn kernel_main(boot_info: &'static boot::BootInfo) -> ! {
    ysos::init(boot_info);
    //try_get_file("/hello.txt");
    //println!("{:#?}",read_all_file("/hello.txt"));
    // let buf = read_all_file("/hello.txt").unwrap();
    // for x in &buf{
    //     print!("{},", x);
    // }
    // println!();
    // for x in &buf{
    //     print!("{}", *x as char);
    // }
    // println!();
    wait(spawn_init());
    ysos::shutdown(boot_info);
}

pub fn spawn_init() -> proc::ProcessId {
    proc::spawn("shell").unwrap()
}

pub fn open_drive(){
    let drive = AtaDrive::open(0, 0).unwrap();
    let mbrtab = MbrTable::parse(drive)
        .expect("Failed to parse MBR");
    let parts = mbrtab.partitions().expect("Failed to get partitions");
    let mut i = 0;
    for p in parts{
        let offset = p.offset;
        let size = p.size;
        info!("Found partition#{} at offset {} with size {}", i, offset, size);
        i += 1;
    }
}