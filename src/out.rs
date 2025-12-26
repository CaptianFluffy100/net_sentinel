use colored::*;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

fn get_timestamp() -> String {
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(duration) => {
            let total_secs = duration.as_secs();
            let millis = duration.subsec_millis();
            let hours = (total_secs / 3600) % 24;
            let minutes = (total_secs / 60) % 60;
            let seconds = total_secs % 60;
            format!("{:02}:{:02}:{:02}.{:03}", hours, minutes, seconds, millis)
        }
        Err(_) => "00:00:00.000".to_string(),
    }
}

pub fn ok(script: &str, msg: &str) {
  println!("[{}][{}] {}", get_timestamp(), script.bold().green(), msg.green());
}

pub fn warning(script: &str, msg: &str) {
  println!("[{}][{}] {}", get_timestamp(), script.bold().yellow(), msg.yellow());
}

pub fn error(script: &str, msg: &str) {
  println!("[{}][{}] {}", get_timestamp(), script.bold().red(), msg.red());
}

pub fn debug(script: &str, msg: &str) {
  println!("[{}][{}] {}", get_timestamp(), script.bold(), msg);
}

pub fn info(script: &str, msg: &str) {
  println!("[{}][{}] {}", get_timestamp(), script.bold().blue(), msg.blue());
}

pub fn secret(script: &str, msg: &str) {
  println!("[{}][{}] {}", get_timestamp(), script.bold().purple(), msg.purple());
}