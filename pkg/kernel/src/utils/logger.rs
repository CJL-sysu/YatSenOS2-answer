use log::{Metadata, Record, Level, LevelFilter};

pub fn init() {
    static LOGGER: Logger = Logger;
    log::set_logger(&LOGGER).unwrap();

    // FIXME: Configure the logger
    //设置最高输出等级为Trace，以便查看所有log的效果
    log::set_max_level(LevelFilter::Debug);

    info!("Logger Initialized.");
}

struct Logger;

impl log::Log for Logger {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        // FIXME: Implement the logger with serial output
        if self.enabled(record.metadata()) {
            // println!("{} - {}", record.level(), record.args());
            match record.level() {
                Level::Error => println!(
                    "\x1b[31;1;4m[X] Error\x1b[0m  - \x1b[31mfrom {}, at line {}, {}\x1b[0m",
                    record.file_static().unwrap(),
                    record.line().unwrap(),
                    record.args(),
                ),
                Level::Warn => println!(
                    "\x1b[33;1;4m[!] Warning\x1b[0m- \x1b[33mfrom {}, at line {}, {}\x1b[0m",
                    record.file_static().unwrap(),
                    record.line().unwrap(),
                    record.args(),
                ),
                Level::Info => println!(
                    "\x1b[34;1;4m[+] Info\x1b[0m   - \x1b[34mfrom {}, at line {}, {}\x1b[0m",
                    record.file_static().unwrap(),
                    record.line().unwrap(),
                    record.args(),
                ),
                Level::Debug => println!(
                    "\x1b[36;1;4m[#] Debug\x1b[0m  - \x1b[36mfrom {}, at line {}, {}\x1b[0m",
                    record.file_static().unwrap(),
                    record.line().unwrap(),
                    record.args(),
                ),
                Level::Trace => println!(
                    "\x1b[32;1;4m[%] Trace\x1b[0m  - \x1b[32mfrom {}, at line {}, {}\x1b[0m",
                    record.file_static().unwrap(),
                    record.line().unwrap(),
                    record.args(),
                ),
            }
        }
    }

    fn flush(&self) {}
}
