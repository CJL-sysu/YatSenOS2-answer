use lib::{string::{String, ToString}, vec::Vec};

use crate::CUR_DIR;


pub fn mycd(dist: &str){
    let goto:String;
    //获取用户输入为dist
    
    //得到前往的目录路径
    goto = get_path(&dist.to_string());
    
    unsafe{
        CUR_DIR = goto.clone();
    }
}
pub fn get_path(dist: &String) -> String {
    // 根据输入的dist路径，返回一个字符串，表示前往的目录
    // dist可以是相对路径，也可以是绝对路径
    // 仅支持linux路径
    let tar = split_path(dist);
    //考虑第一个字符是不是`/`
    if dist.chars().next().unwrap() != '/' {
        //cur当前的目录
        //let cur = String::from(env::current_dir().unwrap().to_str().unwrap());
        let mut vcur = split_path(unsafe { &CUR_DIR });
        vcur.extend(tar);
        vcur = formatting_path(&vcur);
        join_path(&vcur)
    } else {
        let tar = formatting_path(&tar);
        join_path(&tar)
    }
}
fn split_path(path: &String) -> Vec<String> {
    //将路径字符串分解为数组
    let mut tmps: String = String::from("");
    let mut ret: Vec<String> = Vec::new();
    for ch in path.chars() {
        if ch == '/' {
            if tmps.len() != 0 {
                let add = tmps.clone();
                ret.push(add);
            }
            tmps.clear();
        } else {
            tmps.push(ch);
        }
    }
    if tmps.len() != 0 {
        let add = tmps.clone();
        ret.push(add);
    }
    ret
}
fn formatting_path(path: &Vec<String>) -> Vec<String> {
    //处理路径中的`.`和`..`
    let mut ret: Vec<String> = Vec::new();
    for p in path {
        if p == ".." {
            if ret.len() != 0 {
                ret.pop();
            }
        } else if p != "." {
            let add = p.clone();
            ret.push(add);
        }
    }
    ret
}
fn join_path(path: &Vec<String>) -> String {
    //将表示路径的字符串数组重新组合为字符串路径
    let mut ret = String::from("");
    for s in path {
        ret += "/";
        ret += s;
    }
    match ret.len() {
        0 => String::from("/"),
        _ => ret,
    }
}
