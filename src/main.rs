use std::env;
use std::io::Write;
use std::process::Command;
use std::fs;
use std::os;
use std::collections::HashMap;
use clipboard_win::{formats, set_clipboard};
use regex::Regex;

const CONFFILE: &str = "waypoint.config.txt";

fn serialize(map: &HashMap<String, String>) -> String {
  let mut result: String = "".to_string();
  for (k, v) in map {
    result = result + k + " = " + v + "\n";
  }
  result.pop(); // remove last newline
  return result;
}

fn shortcuts_from_file(f: &str) -> HashMap<String, String>{
  let s = fs::read_to_string(f).expect("read file");
  let v: Vec<&str> = s.split("\n").collect();
  let mut result = HashMap::new();
  for line in v {
    let (s1, s2) = line.split_once(" = ").expect("failed to split");
    result.insert("[".to_string() + s1 + "]", s2.to_string());    
  }
  return result;
}

fn create_shortcuts_file() {
  fs::File::create(CONFFILE).expect("failed to create config file");
}

fn set_shortcut(s1: &str, s2: &str) {
  let mut current = shortcuts_from_file(CONFFILE);
  if current.keys().any(|k| k == &s1) {
    return;
  };
  current.insert("[".to_string() + s1 + "]", s2.to_string());
  create_shortcuts_file();
  let result = serialize(&current);
  let mut file = fs::File::open(CONFFILE).unwrap();
  file.write(result.as_bytes()).unwrap();
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
  shortcuts.insert(
    "[m]".to_string(), 
    "C:\\Users\\TrybSkupienia\\Desktop\\linalgSFML\\PhysicsVector\\x64\\Debug".to_string()
  );
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
        _ => {
          let mut fin = f.to_string();
          for original in re.captures_iter(f) {
            match shortcuts.get(&original[0]) {
              Some(matched) => {
                fin = fin.replace(&original[0], matched);
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
