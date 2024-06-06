use lib::{print, println, string::{String, ToString}, sys_cat, sys_list_dir};
use mycd::get_path;

use crate::mycd;

pub fn mycat(dist: &str){
    let goto:String;
    //获取用户输入为dist
    
    //得到前往的目录路径
    goto = get_path(&dist.to_string());
    
    let mut buf = [0u8; 4096];
    let len = sys_cat(&goto, &mut buf);
    for i in 0..len{
        print!("{}", buf[i] as char);
    }
    println!();
}