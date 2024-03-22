use alloc::string::String;
/* your input type */
use crossbeam_queue::ArrayQueue;
use pc_keyboard::DecodedKey;
type Key = DecodedKey;

const BUFFER_SIZE: usize = 128;
lazy_static! {
    static ref INPUT_BUF: ArrayQueue<Key> = ArrayQueue::new(BUFFER_SIZE);
}

#[inline]
pub fn push_key(key: Key) {
    if INPUT_BUF.push(key).is_err() {
        warn!("Input buffer is full. Dropping key '{:?}'", key);
    }
}

#[inline]
pub fn try_pop_key() -> Option<Key> {
    INPUT_BUF.pop()
}

pub fn pop_key() -> Key {
    loop {
        if let Some(key) = try_pop_key() {
            return key;
        }
    }
}
pub fn get_line() -> String {
    let mut line = String::with_capacity(BUFFER_SIZE);
    loop{
        let cur_key = pop_key();
        match cur_key {
            Key::Unicode(c) => {
                match c {
                    '\n'|'\r' => {
                        print!("\n");
                        break;
                    },
                    '\x08' | '\x7f' => {
                        if !line.is_empty() {
                            print!("\x08\x20\x08");
                            line.pop();
                        }
                    }
                    c => {
                        print!("{}", c);
                        line.push(c);
                    }
                }
            }
            _ => {}
        }
    }
    line
}