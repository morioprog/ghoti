use super::Logger;

pub struct NullLogger {}

impl Logger for NullLogger {
    fn new(_log_dir: &str, _log_name: Option<&str>) -> Result<Self, std::io::Error> {
        Ok(NullLogger {})
    }
    fn print(&mut self, _log: String) -> Result<(), std::io::Error> {
        Ok(())
    }
}
