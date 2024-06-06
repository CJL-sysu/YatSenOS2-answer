use lib::{print, println, string::{String, ToString}, sys_cat, sys_list_dir, sys_spawn_file, sys_wait_pid};
use mycd::get_path;

use crate::mycd;

pub fn myrun(dist: &str){
    if dist.len() == 0{
        return;
    }
    let path:String = get_path(&dist.to_string());
    let pid = sys_spawn_file(path.as_str());
    //println!("{}",pid);
    if pid == 0{
        println!("Error: failed to run {}", dist);
    }else{
        sys_wait_pid(pid);
    }
}