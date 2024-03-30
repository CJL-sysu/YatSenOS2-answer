#![no_std]
#![no_main]
#![feature(alloc_error_handler)]

#[macro_use]
extern crate log;
extern crate alloc;

use alloc::boxed::Box;
use alloc::vec;
use elf::{load_elf, map_physical_memory, map_range};
use uefi::{allocator::exit_boot_services, prelude::*};
use x86_64::registers::control::*;
use xmas_elf::ElfFile;
use ysos_boot::*;

mod config;

const CONFIG_PATH: &str = "\\EFI\\BOOT\\boot.conf";

#[entry]
fn efi_main(image: uefi::Handle, mut system_table: SystemTable<Boot>) -> Status {
    uefi_services::init(&mut system_table).expect("Failed to initialize utilities");

    //log::set_max_level(log::LevelFilter::Info);
    log::set_max_level(log::LevelFilter::Trace);
    info!("Running UEFI bootloader...");

    let bs = system_table.boot_services();
    // 2.1 加载相关文件
    // 1. Load config
    let config = { 
        /* FIXME: Load config file */ 
        //打开并加载config文件
        let mut file = open_file(bs, CONFIG_PATH);
        let buf = load_file(bs, &mut file);
        //从config文件内容加载Config结构体
        crate::config::Config::parse(buf)
    };

    info!("Config: {:#x?}", config);

    // 2. Load ELF files
    let elf = { 
        /* FIXME: Load kernel elf file */ 
        //内核存储地址可以从config结构体读出
        let KERNEL_PATH = config.kernel_path;
        //读取内核文件
        let mut file = open_file(bs, KERNEL_PATH);
        let buf = load_file(bs, &mut file);
        //新建ElfFile结构体
        ElfFile::new(buf).unwrap()
    };

    unsafe {
        set_entry(elf.header.pt2.entry_point() as usize);
    }

    // 3. Load MemoryMap
    let max_mmap_size = system_table.boot_services().memory_map_size();
    let mmap_storage = Box::leak(
        vec![0; max_mmap_size.map_size + 10 * max_mmap_size.entry_size].into_boxed_slice(),
    );
    let mmap = system_table
        .boot_services()
        .memory_map(mmap_storage)
        .expect("Failed to get memory map");

    let max_phys_addr = mmap
        .entries()
        .map(|m| m.phys_start + m.page_count * 0x1000)
        .max()
        .unwrap()
        .max(0x1_0000_0000); // include IOAPIC MMIO area

    // 4. Map ELF segments, kernel stack and physical memory to virtual memory
    let mut page_table = current_page_table();
    // 2.2 更新控制寄存器
    // FIXME: root page table is readonly, disable write protect (Cr0)
    unsafe {
        //WRITE_PROTECT: When set, it is not possible to write to read-only pages from ring 0.
        //fn remove(&mut self, other: Self)
        //self & !other
        Cr0::update(|f| f.remove(Cr0Flags::WRITE_PROTECT));
    }
    // FIXME: map physical memory to specific virtual address offset
    let mut frame_allocator = UEFIFrameAllocator(bs);
    map_physical_memory(config.physical_memory_offset, max_phys_addr, &mut page_table, &mut frame_allocator);
    // FIXME: load and map the kernel elf file
    load_elf(&elf, config.physical_memory_offset, &mut page_table, &mut frame_allocator).unwrap();
    // FIXME: map kernel stack
    let _page_range = map_range(
        config.kernel_stack_address,
        match config.kernel_stack_auto_grow{
            0 => config.kernel_stack_size,
            _ => config.kernel_stack_auto_grow / 4096,
        },
        &mut page_table, &mut frame_allocator).unwrap();
    debug!(
        "Kernel init stack: [0x{:x?} -> 0x{:x?})",
        config.kernel_stack_address,
        config.kernel_stack_address + match config.kernel_stack_auto_grow{
            0 => config.kernel_stack_size,
            _ => config.kernel_stack_auto_grow / 4096,
        } * 0x1000
    );
    // FIXME: recover write protect (Cr0)
    unsafe {
        //WRITE_PROTECT: When set, it is not possible to write to read-only pages from ring 0.
        //fn insert(&mut self, other: Self)
        //The bitwise or (|) of the bits in two flags values.
        Cr0::update(|f| f.insert(Cr0Flags::WRITE_PROTECT));
    }
    free_elf(bs, elf);

    // 5. Exit boot and jump to ELF entry
    info!("Exiting boot services...");
    //info!("jump to entry");
    let (runtime, mmap) = system_table.exit_boot_services(MemoryType::LOADER_DATA);
    // NOTE: alloc & log are no longer available

    // construct BootInfo
    let bootinfo = BootInfo {
        memory_map: mmap.entries().copied().collect(),
        physical_memory_offset: config.physical_memory_offset,
        system_table: runtime,
    };

    // align stack to 8 bytes
    let stacktop = config.kernel_stack_address + config.kernel_stack_size * 0x1000 - 8;
    // //用于让程序在这里暂停，方便调试
    // use core::arch::asm;
    // for _ in 0..0x10000000 {
    //         unsafe {
    //             asm!("nop");
    //         }
    //     }
    // //
    unsafe {
        jump_to_entry(&bootinfo, stacktop);
    }
}
