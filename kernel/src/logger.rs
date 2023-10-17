use crate::{
    terminal::{Terminal, TERMINAL},
    uninterruptible_mutex::UninterruptibleMutex,
};
use core::fmt::Write;

pub(crate) struct Logger<'a> {
    terminal: &'a UninterruptibleMutex<Terminal<'a>>,
}

impl<'a> Logger<'a> {
    pub const fn new(terminal: &'a UninterruptibleMutex<Terminal<'a>>) -> Logger<'_> {
        Self { terminal }
    }
}

pub(crate) static LOGGER: Logger = Logger::new(&TERMINAL);

pub(crate) fn init() {
    log::set_logger(&LOGGER).unwrap();
    log::set_max_level(log::LevelFilter::Trace);
    log::info!("Logger initialized");
}

impl log::Log for Logger<'_> {
    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        let mut terminal = self.terminal.lock();
        writeln!(terminal, "{:5}: {}", record.level(), record.args()).unwrap();
    }

    fn flush(&self) {
        self.terminal.lock().flush();
    }
}
