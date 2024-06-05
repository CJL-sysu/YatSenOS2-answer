use crate::humanized_size;

use super::ata::*;
use alloc::boxed::Box;
use alloc::format;
use alloc::string::ToString;
use chrono::DateTime;
use storage::fat16::Fat16;
use storage::mbr::*;
use storage::*;
use x86::time;

pub static ROOTFS: spin::Once<Mount> = spin::Once::new();

pub fn get_rootfs() -> &'static Mount {
    ROOTFS.get().unwrap()
}

pub fn init() {
    info!("Opening disk device...");

    let drive = AtaDrive::open(0, 0).expect("Failed to open disk device");

    // only get the first partition
    let part = MbrTable::parse(drive)
        .expect("Failed to parse MBR")
        .partitions()
        .expect("Failed to get partitions")
        .remove(0);

    info!("Mounting filesystem...");

    ROOTFS.call_once(|| Mount::new(Box::new(Fat16::new(part)), "/".into()));

    trace!("Root filesystem: {:#?}", ROOTFS.get().unwrap());

    info!("Initialized Filesystem.");
}

pub fn ls(root_path: &str) {
    let mut path = root_path.to_string();
    if !path.ends_with("/"){
        path = path + "/";
    }
    let root_path = path.as_str();
    let iter = match get_rootfs().read_dir(root_path) {
        Ok(iter) => iter,
        Err(err) => {
            warn!("{:?}", err);
            return;
        }
    };
    
    // FIXME: format and print the file metadata
    //      - use `for meta in iter` to iterate over the entries
    //      - use `crate::humanized_size_short` for file size
    //      - add '/' to the end of directory names
    //      - format the date as you liket
    //      - do not forget to print the table header
    println!("|name            |type     |size      |created                |modified               |accessed               |");
    println!("|----------------+---------+----------+-----------------------+-----------------------+-----------------------|");
    for meta in iter{
        let create_time = match meta.created{
            Some(time) => time.to_string(),
            None => "unknown".to_string(),
        };
        let modified_time = match meta.modified{
            Some(time) => time.to_string(),
            None => "unknown".to_string(),
        };
        let accessed_time = match meta.accessed{
            Some(time) => time.to_string(),
            None => "unknown".to_string(),
        };
        let entry_type = match meta.entry_type{
            FileType::File => "file",
            FileType::Directory => "directory",
        };
        let (size, units) = humanized_size(meta.len as u64);
        let len = format!("{:.2}{:3}",size, units);
        println!("|{:16}|{:9}|{:10}|{:23}|{:23}|{:23}|", meta.name, entry_type, len, create_time, modified_time, accessed_time);
    }
}
