pub mod file_logger;
pub mod null_logger;

pub use self::{file_logger::FileLogger, null_logger::NullLogger};

pub trait Logger {
    fn new(log_dir: &str, log_name: Option<&str>) -> Result<Self, std::io::Error>
    where
        Self: Sized;
    fn print(&mut self, log: String) -> Result<(), std::io::Error>;
}
