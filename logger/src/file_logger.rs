use std::{
    fs::{create_dir_all, File},
    io::{BufWriter, Write},
};

use chrono::Utc;

use super::Logger;

pub struct FileLogger {
    buf_writer: BufWriter<File>,
}

impl Logger for FileLogger {
    fn new(log_dir: &str, log_name: Option<&str>) -> Result<Self, std::io::Error> {
        let time_text = Utc::now().format("%Y%m%d_%H%M%S_%f").to_string();
        let log_name = log_name.unwrap_or(time_text.as_str());

        create_dir_all(log_dir)?;
        let file_path = format!("{}/{}.log", log_dir, log_name);
        let buf_writer = BufWriter::new(File::create(&file_path)?);

        Ok(FileLogger { buf_writer })
    }

    fn print(&mut self, log: String) -> Result<(), std::io::Error> {
        print!("{}", log);

        write!(self.buf_writer, "{}", log)?;
        self.buf_writer.flush()?;

        Ok(())
    }
}
