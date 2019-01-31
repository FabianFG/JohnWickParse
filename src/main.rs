extern crate byteorder;
extern crate hex;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
#[macro_use]
extern crate erased_serde;

use std::path::Path;
use std::fs;
use std::io::Write;
use std::env;

mod rijndael;
mod assets;
mod archives;

#[derive(Debug)]
struct CommandError {
    message: &'static str,
}

impl std::error::Error for CommandError {

}

impl std::fmt::Display for CommandError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Error: {}", self.message)
    }
}

type CommandResult = Result<(), CommandError>;

fn cerr(message: &'static str) -> CommandResult {
    Err(CommandError {
        message
    })
}

fn serialize(params: &[String]) -> CommandResult {
    let path = match params.get(0) {
        Some(data) => data,
        None => return cerr("No path specified"),
    };

    let package = assets::Package::new(path);
    let serial_package = serde_json::to_string(&package).unwrap();
    let mut file = fs::File::create(path.to_owned() + ".json").unwrap();
    file.write_all(serial_package.as_bytes()).unwrap();

    Ok(())
}

fn filelist(params: &[String]) -> CommandResult {
    let path = match params.get(0) {
        Some(data) => data,
        None => return cerr("No path specified"),
    };
    let key = match std::fs::read_to_string("key.txt") {
        Ok(data) => data,
        Err(_) => return cerr("Could not read key"),
    };

    let archive = archives::PakExtractor::new(path, &key);
    let entries = archive.get_entries();
    let file_list = entries.into_iter().map(|v| v.get_filename()).fold(String::new(), |acc, v| acc + v + "\n");
    let mut file = fs::File::create(path.to_owned() + ".txt").unwrap();
    file.write_all(file_list.as_bytes()).unwrap();

    Ok(())
}

fn extract(params: &[String]) -> CommandResult {
    let path = match params.get(0) {
        Some(data) => data,
        None => return cerr("No path specified"),
    };
    let key = match std::fs::read_to_string("key.txt") {
        Ok(data) => data,
        Err(_) => return cerr("Could not read key"),
    };
    let pattern = match params.get(1) {
        Some(data) => data,
        None => return cerr("No pattern specified"),
    };

    let mut archive = archives::PakExtractor::new(path, &key);
    let entries: Vec<archives::FPakEntry> = archive.get_entries().into_iter().filter(|v| v.get_filename().contains(pattern)).cloned().collect();

    for asset in entries {
        let file_contents = archive.get_file(&asset);
        let path = Path::new(asset.get_filename());
        if let Some(basename) = path.parent() {
            fs::create_dir_all(basename).expect("Could not create directory");
        }
        let mut file = fs::File::create(asset.get_filename()).unwrap();
        file.write_all(&file_contents).unwrap();
    }

    Ok(())
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let command = args.get(1);
    let command = match command {
        Some(data) => data,
        None => {
            println!("No command specified");
            return
        }
    };
    let params = &args[2..];

    let err = match (*command).as_ref() {
        "serialize" => serialize(params),
        "filelist" => filelist(params),
        "extract" => extract(params),
        _ => {
            println!("Invalid command");
            Ok(())
        },
    };
    if let Err(error) = err {
        println!("{}", error);
    }
}
