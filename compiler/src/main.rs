//! llrc: the Llrst bootstrap compiler.
#![allow(dead_code)]
mod ast;
mod borrow;
mod check;
mod codegen;
mod diag;
mod driver;
mod inl;
mod link;
mod mir;
mod parse;
mod resolve;
mod sexpr;
mod thir;
mod traitres;
mod ty;

use driver::Options;
use std::path::PathBuf;
use std::process::Command;

const USAGE: &str = "usage|llrc build FILE [-o OUT] [--target TRIPLE] [--std DIR] [--test] [--emit-obj] [--checked] [--deny CODE]
usage|llrc run FILE [--std DIR] [--deny CODE] [-- ARGS...]
usage|llrc check FILE [--std DIR] [--deny CODE]
usage|llrc emit-mir FILE [--std DIR] [--deny CODE]
usage|llrc version";

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut opt = Options {
        input: PathBuf::new(),
        output: None,
        target: None,
        std_dir: None,
        test: false,
        emit_obj: false,
        emit_mir: false,
        check_only: false,
        checked: false,
        deny: Vec::new(),
    };
    let mut run_args: Vec<String> = Vec::new();
    let mut cmd = String::new();
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-o" => {
                i += 1;
                opt.output = args.get(i).map(PathBuf::from);
            }
            "--target" => {
                i += 1;
                opt.target = args.get(i).cloned();
            }
            "--std" => {
                i += 1;
                opt.std_dir = args.get(i).map(PathBuf::from);
            }
            "--test" => opt.test = true,
            "--emit-obj" => opt.emit_obj = true,
            "--checked" => opt.checked = true,
            "--deny" => {
                i += 1;
                if let Some(code) = args.get(i) {
                    opt.deny.push(code.clone());
                }
            }
            "--" => {
                run_args = args[i + 1..].to_vec();
                break;
            }
            a if a.starts_with('-') => {
                eprintln!("E|<cli>|0|0|0|cli.unknown_flag|flag={}", a);
                std::process::exit(1);
            }
            a => {
                if cmd.is_empty() {
                    cmd = a.to_string();
                } else if opt.input.as_os_str().is_empty() {
                    opt.input = PathBuf::from(a);
                } else {
                    run_args.push(a.to_string());
                }
            }
        }
        i += 1;
    }
    match cmd.as_str() {
        "version" => {
            println!("llrc|0.1.0");
            return;
        }
        "build" | "run" | "check" | "emit-mir" => {}
        _ => {
            eprintln!("{}", USAGE);
            std::process::exit(1);
        }
    }
    if opt.input.as_os_str().is_empty() {
        eprintln!("{}", USAGE);
        std::process::exit(1);
    }
    opt.check_only = cmd == "check" || cmd == "emit-mir";
    opt.emit_mir = cmd == "emit-mir";
    if cmd == "run" {
        let tmp = std::env::temp_dir().join(format!("llr_run_{}", std::process::id()));
        opt.output = Some(tmp.clone());
        let code = driver::run(opt);
        if code != 0 {
            std::process::exit(code);
        }
        let status = Command::new(&tmp).args(&run_args).status();
        let _ = std::fs::remove_file(&tmp);
        match status {
            Ok(s) => std::process::exit(s.code().unwrap_or(101)),
            Err(e) => {
                eprintln!("E|<run>|0|0|0|run.failed|got={}", e);
                std::process::exit(1);
            }
        }
    }
    std::process::exit(driver::run(opt));
}
