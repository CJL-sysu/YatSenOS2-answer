use lib::{string::{String, ToString}, sys_list_dir};
use mycd::get_path;

use crate::mycd;

pub fn myls(dist: &str){
    let goto:String;
    //获取用户输入为dist
    
    //得到前往的目录路径
    goto = get_path(&dist.to_string());
    
    sys_list_dir(&goto);
}