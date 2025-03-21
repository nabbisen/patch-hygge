use std::ffi::OsString;
use std::io::{self, BufRead, BufReader, Read, Write};
use std::process::Command;
use std::time::UNIX_EPOCH;
use std::{fs::File, path::Path};

use chardetng::EncodingDetector;
use chrono::{Local, TimeZone};
use encoding_rs::{Encoding, UTF_8};

use super::types::{FileAttr, ListDirReponse, ReadContent};

const UTF8_CHARSET: &str = "UTF-8";
const NOT_TEXTFILE_CHARSET: &str = "(bytes array)";

pub fn filepaths_content(old: &str, new: &str) -> Result<Vec<ReadContent>, String> {
    if is_textfile(old) && is_textfile(new) {
        return Ok(vec![textfile_content(old), textfile_content(new)]);
    }

    if old.ends_with(".xlsx") && new.ends_with(".xlsx") {
        // todo
        return Ok(vec![]);
    }

    Err("Neither textfile(s) nor comparable file(s)".to_owned())
}

fn is_textfile(filepath: &str) -> bool {
    let file = File::open(filepath);
    match file {
        Ok(f) => {
            let mut reader = BufReader::new(f);
            let mut buffer = String::new();
            reader.read_line(&mut buffer).is_ok()
        }
        Err(_) => false,
    }
}

fn textfile_content(filepath: &str) -> ReadContent {
    let mut file = File::open(filepath).expect(format!("failed to open {}", filepath).as_str());
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).unwrap();

    let is_binary = buffer.windows(2).any(|window| window[0] == 0x00);
    if is_binary {
        const BYTES_ARRAY_ROW_LENGTH: usize = 16;
        let mut grid = String::new();
        for chunk in buffer.chunks(BYTES_ARRAY_ROW_LENGTH) {
            for byte in chunk {
                grid.push_str(&format!("{:02X} ", byte));
            }
            grid.push_str("\n");
        }
        return ReadContent {
            charset: NOT_TEXTFILE_CHARSET.to_owned(),
            content: grid,
        };
    }

    match std::str::from_utf8(&buffer) {
        Ok(x) => {
            return ReadContent {
                charset: UTF8_CHARSET.to_owned(),
                content: x.to_owned(),
            }
        }
        Err(_) => (),
    }

    let mut detector = EncodingDetector::new();
    detector.feed(&buffer, true);
    let encoding = detector.guess(None, false);
    let (decoded, _, had_errors) = encoding.decode(&buffer);
    if had_errors {
        eprint!("not binary, not utf-8 text and not any other encoded text.")
    }
    ReadContent {
        charset: encoding.name().to_owned(),
        content: decoded.to_string(),
    }
}

pub fn list_dir(current_dir: &str) -> Result<ListDirReponse, String> {
    let target_dir = if current_dir.is_empty() {
        std::env::current_dir().expect("Failed to get current directory")
    } else {
        Path::new(current_dir)
            .canonicalize()
            .expect(format!("Failed to canonicalize path: {}", current_dir).as_str())
    };
    let mut dirs = Vec::<String>::new();
    let mut files = Vec::<FileAttr>::new();

    let read = match std::fs::read_dir(target_dir.as_path()) {
        Ok(x) => x,
        Err(err) => {
            return Err(format!("Invalid path: {} ({})", current_dir, err));
        }
    };
    for x in read {
        match x {
            Ok(dir_entry) => {
                let name = dir_entry.file_name().to_string_lossy().to_string();
                match dir_entry.metadata() {
                    Ok(metadata) => {
                        if metadata.is_dir() {
                            dirs.push(name)
                        } else {
                            let modified = metadata
                                .modified()
                                .unwrap()
                                .duration_since(UNIX_EPOCH)
                                .unwrap();
                            let local_timestamp = Local.timestamp_nanos(modified.as_nanos() as i64);
                            let last_modified =
                                local_timestamp.format("%Y-%m-%d %H:%M:%S").to_string();
                            files.push(FileAttr {
                                name,
                                bytes_size: format!(
                                    "{} bytes",
                                    comma_separated_bytes_size(metadata.len())
                                ),
                                human_readable_size: human_readable_size(metadata.len()),
                                last_modified,
                            })
                        }
                    }
                    _ => {}
                }
            }
            // todo
            Err(err) => println!("Failed to get dir/file info due to {}", err),
        }
    }

    dirs.sort();
    files.sort();

    Ok(ListDirReponse {
        current_dir: target_dir.to_string_lossy().to_string(),
        dirs: dirs,
        files: files,
    })
}

pub fn comma_separated_bytes_size(size: u64) -> String {
    let size_str = size.to_string();

    let mut ret = String::new();
    for (i, c) in size_str.chars().rev().enumerate() {
        if i != 0 && i % 3 == 0 {
            ret.push(',');
        }
        ret.push(c);
    }

    ret.chars().rev().collect()
}

pub fn human_readable_size(size: u64) -> String {
    const UNIT: u64 = 1024;
    const K: u64 = UNIT;
    const M: u64 = UNIT.pow(2);
    const G: u64 = UNIT.pow(3);
    const T: u64 = UNIT.pow(4);

    let (size, unit) = if size >= T {
        (size as f64 / T as f64, "TB")
    } else if size >= G {
        (size as f64 / G as f64, "GB")
    } else if size >= M {
        (size as f64 / M as f64, "MB")
    } else if size >= K {
        (size as f64 / K as f64, "KB")
    } else {
        (size as f64, "bytes")
    };

    let size_str = size.to_string();
    let size_str_parts = size_str.split(".").collect::<Vec<&str>>();
    let int = size_str_parts[0].parse::<u64>().unwrap();
    let comma_separated_int = comma_separated_bytes_size(int);

    let comma_separated_size = if 1 < size_str_parts.len() {
        format!("{}.{:.2}", comma_separated_int, size_str_parts[1])
    } else {
        comma_separated_int
    };
    format!("{} {}", comma_separated_size, unit)
}

pub fn save(filepath: &str, content: &str, charset: &str) -> Result<(), io::Error> {
    let encoding = Encoding::for_label(charset.as_bytes()).unwrap_or(UTF_8);
    let (encoded, _, _) = encoding.encode(content);
    let mut file = File::create(filepath)?;
    file.write_all(&encoded)?;
    Ok(())
}

pub fn file_manager_command() -> &'static str {
    #[cfg(target_os = "windows")]
    {
        "explorer"
    }
    #[cfg(target_os = "macos")]
    {
        "open"
    }
    #[cfg(target_os = "linux")]
    {
        if Command::new("nautilus").arg("--version").output().is_ok() {
            "nautilus"
        } else if Command::new("dolphin").arg("--version").output().is_ok() {
            "dolphin"
        } else if Command::new("nemo").arg("--version").output().is_ok() {
            "nemo"
        } else if Command::new("thunar").arg("--version").output().is_ok() {
            "thunar"
        } else {
            "xdg-open"
        }
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        compile_error!("Unsupported operating system")
    }
}

pub fn arg_to_filepath(arg: &Option<OsString>) -> Option<String> {
    if let Some(s) = arg {
        let s = s.to_string_lossy();
        if Path::new(s.as_ref()).is_file() {
            Some(s.into_owned())
        } else {
            None
        }
    } else {
        None
    }
}
