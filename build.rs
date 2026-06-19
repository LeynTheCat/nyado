use std::fs;
use std::path::Path;
use std::env;

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("builtin_langs.rs");
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let config_dir = Path::new(&manifest_dir).join("config");

    //println!("cargo:warning=config_dir = {}", config_dir.display());
    //println!("cargo:warning=exists = {}", config_dir.exists());

    let mut entries = Vec::new();
    if config_dir.exists() && config_dir.is_dir() {
        for entry in fs::read_dir(&config_dir).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.is_file() {
                let name = path.file_name().unwrap().to_str().unwrap();
                // println!("cargo:warning=found file: {}", name);
                if name.starts_with("lang_") && name.ends_with(".toml") {
                    let code = &name[5..name.len()-5];
                    //println!("cargo:warning=  -> lang code: {}", code);
                    let content = fs::read_to_string(&path).unwrap();
                    let escaped = content
                        .replace('\\', "\\\\")
                        .replace('"', "\\\"")
                        .replace('\r', "\\r")
                        .replace('\n', "\\n");
                    entries.push(format!("(\"{}\", \"{}\")", code, escaped));
                } else {
                    println!("cargo:warning=  -> ignored (not lang_*.toml)");
                }
            }
        }
    } else {
        println!("cargo:warning=config_dir does not exist or is not a directory");
    }

    let builtin = format!("pub const BUILTIN_LANGS: &[(&str, &str)] = &[{}];", entries.join(", "));
    fs::write(&dest_path, builtin).unwrap();
    println!("cargo:rerun-if-changed=config");
}