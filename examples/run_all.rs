use std::env::{args, current_dir, set_current_dir};
use std::fs::read_dir;
use std::process::Command;

const DIR: &str = "examples";

fn main() {
    let release = args().skip(1).count() > 0;

    for entry in read_dir(DIR).expect("not found exampls") {
        let entry = entry.unwrap();
        let path = entry.path();

        let str = entry.file_name();
        let str = str.to_str().expect("OsString.to_str() failed");
        println!("{}/{}: {}", DIR, path.display(), str);

        if path.is_dir() {
            run_project(str, "", release)
        } else if str.ends_with(".rs") && str != "run_all.rs" {
            run_project("", &str[..str.len() - 3], release)
        }
    }
}

fn run_project(dir: &str, example: &str, release: bool) {
    let origin_path = current_dir().unwrap();
    println!(
        "curren_dir0: {:?}, dir: {}, example: {}, release: {}",
        origin_path, dir, example, release
    );

    let mut path = origin_path.clone();
    path.push(DIR);
    path.push(dir);

    println!("curren_path: {:?}", path.display());

    if example.is_empty() {
        set_current_dir(path.as_path()).unwrap();
    }

    println!("curren_dir1: {:?}", current_dir());

    let mut args = vec!["run"];
    if release {
        args.push("--release")
    }

    if dir.is_empty() {
        args.push("--example");
        args.push(example);
    }

    let exit = Command::new("cargo")
        .args(&args[..])
        .spawn()
        .and_then(|mut p| p.wait())
        .map_err(|e| eprintln!("spawn cargo child process failed: {:?}", e))
        .unwrap();

    assert!(exit.success());

    if example.is_empty() {
        set_current_dir(origin_path.as_path()).unwrap();
    }
}
