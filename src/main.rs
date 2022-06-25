use std::{env, fs::OpenOptions};
use std::io::Write;
use std::fs;
use std::collections::HashMap;
use clipboard_win::{formats, set_clipboard};
use regex::Regex;

const CONFFILE: &str = "waypoint.config.txt";

fn serialize(map: &HashMap<String, String>) -> String {
  let mut result: String = "".to_string();
  for (k, v) in map {
    result = result + k + " = " + v + "\n";
  }
  return result;
}

fn deserialize(line: &str) -> (String, String) {
  let (s1, s2) = line.split_once(" = ").expect("failed to split");
  return (s1.to_string(), s2.to_string());
}

fn shortcuts_from_file(f: &str) -> HashMap<String, String>{
  let s = fs::read_to_string(f).expect("read file");
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

fn rm_braces(s: &str) -> &str {
  return &s[1..s.len()-1];
}

fn create_shortcuts_file() {
  fs::File::create(CONFFILE).expect("failed to create config file");
}

fn set_shortcut(s1: &str, s2: &str) {
  let mut current = shortcuts_from_file(CONFFILE);
  if current.keys().any(|k| k == &s1) {
    return;
  };
  current.insert(s1.to_string(), s2.to_string());
  //let mut file = fs::File::open(CONFFILE).unwrap(); READONLY
  let mut file = OpenOptions::new().write(true).open(CONFFILE).unwrap();
  create_shortcuts_file();
  let result = serialize(&current);
  file.write(result.as_bytes()).unwrap();
  file.flush().unwrap();
}

fn main() {
  let mut shortcuts = HashMap::new();
  let handle = fs::read_dir(".").unwrap();
  match handle.map(|file| file.unwrap()).find(|file| file.file_name() == CONFFILE) {
    Some(f) => {
      shortcuts = shortcuts_from_file(f.path().to_str().unwrap());
    },
    None => {
      create_shortcuts_file()
    }
  };
  let args: Vec<String> = env::args().collect(); 
  let re = Regex::new(r"\[\w*\]").unwrap();
  match args.get(1) {
    Some(f) => {
      match f.as_str() {
        "--add" => {
          let s1 = args.get(2).expect("not enough args");
          let s2 = args.get(3).expect("not enough args");
          set_shortcut(s1, s2);
        },
        "--list" => {
          let l = shortcuts_from_file(CONFFILE);
          for (k, v) in l {
            println!("{k} -> {v}");
          }
        },
        "--rm" => {
          let l = shortcuts_from_file(CONFFILE);
          let s1 = args.get(2).expect("not enough args");
          //let drained = l.drain_filter(|&k, v| k != braces(s1)); LOL what the fuck rust? It's been  in nightly 2 years...
          create_shortcuts_file();
          for (k, v) in l.into_iter() {
            if k != s1.to_string() {
              set_shortcut(&k, &v);
            }
          }
        },
        _ => {
          let mut fin = f.to_string();
          println!("{f}");
          for original in re.captures_iter(f) {
            match shortcuts.get(rm_braces(&original[0])) {
              Some(matched) => {
                fin = fin.replace(&original[0], matched);
                println!("{}", &fin);
              },
              None => {
                panic!("unrecognized shortcut")
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
  //let path = &args[1];
  //let mut output = Command::new("cmd");
  //output.current_dir("C:/");
  //println!("{}", output.get_current_dir().unwrap().to_str().unwrap());
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