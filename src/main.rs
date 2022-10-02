use std::env;
use std::io::{Write, Read};
use std::fs::{self, File};
use std::collections::HashMap;
use std::ops::Deref;
use clipboard_win::{formats, set_clipboard};
use regex::Regex;
use std::path::PathBuf;

const LOGFILE: &str = "waypoints.logs.txt";
const CONFFILE: &str = "waypoint.config.txt";

struct FileManager<'a> {
    pub path: String,
    pub filename: &'a str,
}

impl<'a> FileManager<'a> {
    pub fn new(f_path: &'a PathBuf, filename: &'a str) -> FileManager<'a> {
        let path = f_path.to_string_lossy().to_string();
        if !f_path.is_file() {
            ConfigManager::create_file(&path);
        };
        return FileManager {path, filename}
    }
    pub fn create_file(path: &str) -> fs::File {
        return fs::File::create(path).unwrap();
    }
    pub fn get_path(&self) -> String {
        return format!("{}{}", self.path, self.filename); 
    }
    pub fn read(&self) -> String {
        let mut s = String::from("");
        self.get_file().read_to_string(&mut s).expect("Failed to read");
        return s
    }
    pub fn append(&mut self, newline: String) {
        let content = self.read();
        self.rewrite_with(format!("{}\n\r{}", content, newline));
    }
    pub fn get_file(&self) -> File {
        return fs::OpenOptions::new()
            .write(true)
            .read(true)
            .open(&self.path)
            .expect(format!("failed to read {}", self.path).as_str());
    }
    pub fn rewrite_with(&mut self, data: String) {
        self.get_file().set_len(0).unwrap();
        self.get_file().write(
            data.as_bytes()
        ).expect("error writting to file");
    }
}


struct ConfigManager<'a> {
    manager: FileManager<'a>
}
struct LogManager<'a> {
    manager: FileManager<'a>
}
impl<'a> LogManager<'a> {
    pub fn new(f_path: &PathBuf) -> LogManager {
        return LogManager {
            manager: FileManager::new(f_path, CONFFILE)
        }
    }
    pub fn get_path(&self) -> String {
        return self.manager.get_path()
    }
    pub fn create_file(path: &str) -> fs::File {
        return fs::File::create(path).unwrap();
    }
    pub fn get_file(&self) -> File {
        return self.manager.get_file()
    }
}

impl<'a> ConfigManager<'a> {
    pub fn list(&mut self) {
        let l = self.read();
        for (k, v) in l {
            println!("{k} -> {v}");
        }
    }
    pub fn remove(&mut self, key: &str) { 
        let l = self.read();
        let mut result = HashMap::new();
        for (k, v) in l.into_iter() {
            if k != key.to_string() {
            result.insert(k, v);
            }
        }
        self.manager.rewrite_with(serialize(&result));
    }
    pub fn new(f_path: &PathBuf) -> ConfigManager {
        return ConfigManager {
            manager: FileManager::new(f_path, CONFFILE)
        }
    }
    pub fn create_file(path: &str) -> fs::File {
        return fs::File::create(path).unwrap();
    }
    pub fn get_file(&self) -> File {
        return self.manager.get_file()
    }
    pub fn read(&mut self) -> HashMap<String, String> {
        let s = self.manager.read();
        let v: Vec<&str> = s.split("\n").collect();
        let mut result = HashMap::new();
        let check = Regex::new(r"\w* = .*\n?").unwrap();
        for line in v {
            if check.is_match(line) {
                let (s1, s2) = deserialize(line);
                result.insert(s1, s2);
            }
        }
        return result;
    }
    pub fn set(&mut self, s1: &str, s2: &str) {
        let mut current = self.read();
        if current.keys().any(|k| k == &s1) {
        return;
        };
        current.insert(s1.to_string(), s2.to_string());
        self.manager.rewrite_with(serialize(&current));
    }
}


fn serialize(map: &HashMap<String, String>) -> String {
    let mut result: String = String::from("");
    for (k, v) in map {
        result = format!("{}{} = {}\n", result, k.as_str(), v.as_str())
    }
    return result;
}

fn deserialize(line: &str) -> (String, String) {
    let (s1, s2) = line.split_once(" = ").expect("failed to split");
    return (s1.to_string(), s2.to_string());
}

fn rm_braces(s: &str) -> &str {
    return &s[1..s.len()-1];
}

fn main() {
    let mut path = env::current_exe().unwrap();
    path.pop();
    path.push(CONFFILE);
    let mut config = ConfigManager::new(&path);
    let mut hist = LogManager::new(&path);
    let args: Vec<String> = env::args().collect(); 
    let re = Regex::new(r"\[\w*\]").unwrap();
    let shortcuts = config.read();

    match args.get(1) {
        Some(arg) => {
            match arg.as_str() {
                "--help" => {

                }
                "--hist" => {
                    println!("{}", hist.get_path())
                }
                "--path" => {
                    println!("{:?}", path.as_os_str());
                }
                "--add" => {
                    let s1 = args.get(2).expect("not enough args");
                    let s2 = args.get(3).expect("not enough args");
                    config.set(s1, s2);
                },
                "--list" => {
                    config.list();
                },
                "--rm" => {
                    config.remove(args.get(2).expect("Not enough args!"));
                    config.list();
                },
                _ => {
                    let vw: Vec<String> = env::args().collect();
                    let vw = &vw[1..];
                    let mut s = String::new();
                    for x in vw {
                        s += format!("{x} ").as_str();
                    }
                    s.pop();
                    let mut fin = s;
                    for original in re.captures_iter(fin.clone().as_str()) {
                        match shortcuts.get(rm_braces(&original[0])) {
                        Some(matched) => {
                            fin = fin.replace(&original[0], matched);
                        },
                        None => {
                            if &original[0] == "[rn]" {
                            fin = fin.replace(&original[0], "\r\n");
                            } else {
                            panic!("unrecognized shortcut")
                            }
                        }
                        };
                    }
                    set_clipboard(formats::Unicode, fin).expect("Setting clipboard");
                },
            }
        }
        None => {
            panic!("No argument provided");
        }
    };
}

#[cfg(test)]
mod unit_tests {
  use std::collections::HashMap;
  use crate::*;
  const TESTCONFIG: &str = "waypoints = C:\\Users\\thisUser\\Desktop\\rust\\waypoints\nmain = C:\nd = C:\\Users\\thisUser\\Desktop\nwps = C:\\Users\\thisUser\\Desktop\\rust\\waypoints\\target\\debug";
  #[test]
  fn test_serialization() {
    let mut h = HashMap::new();
    h.insert("main".to_string(), "C:".to_string());
    let result = serialize(&h);
    assert_eq!(result, "main = C:\n", "{}", result);
  }
  #[test]
  fn test_deserialization() {
    let mut iter = TESTCONFIG.split("\n");
    let (k, v) = deserialize(iter.next().unwrap());
    assert_eq!(k, "waypoints");
    assert_eq!(v, "C:\\Users\\thisUser\\Desktop\\rust\\waypoints");
    let (k, v) = deserialize(iter.next().unwrap());
    assert_eq!(k, "main");
    assert_eq!(v, "C:");
  }
}