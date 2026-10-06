//! `bliki` — the posse-bliki static site build. Rust std only, no crates.
//!
//! Behaviour is defined by `spec/` and verified by `tests/`; see
//! docs/adr/0007-rust-build-zero-dependencies.md.

mod date;
mod entities;
mod frontmatter;
mod json;
mod markdown;
mod mdurl;
mod serve;
mod site;
mod text;
mod unicode;
mod util;

use std::path::{Path, PathBuf};
use std::process::ExitCode;

const USAGE: &str = "usage: bliki [--content DIR] [--out DIR] [--serve] [--port N] [--print-config]

Build the posse-bliki site.

options:
  --content DIR    content vault (default: content)
  --out DIR        output directory (default: public)
  --serve          build, serve the output at 127.0.0.1:PORT and rebuild on changes
  --port N         port for --serve (default: 8080)
  --print-config   print the site config as JSON and exit
  -h, --help       show this help and exit";

struct Args {
    content: PathBuf,
    out: PathBuf,
    serve: bool,
    port: u16,
    print_config: bool,
}

fn parse_args() -> Result<Args, String> {
    let mut args = Args {
        content: PathBuf::from("content"),
        out: PathBuf::from("public"),
        serve: false,
        port: 8080,
        print_config: false,
    };
    let mut it = std::env::args().skip(1);
    while let Some(arg) = it.next() {
        let (flag, inline) = match arg.split_once('=') {
            Some((f, v)) if f.starts_with("--") => (f.to_string(), Some(v.to_string())),
            _ => (arg.clone(), None),
        };
        let mut value = |name: &str| -> Result<String, String> {
            match inline.clone().or_else(|| it.next()) {
                Some(v) => Ok(v),
                None => Err(format!("argument {}: expected one argument", name)),
            }
        };
        match flag.as_str() {
            "--content" => args.content = PathBuf::from(value("--content")?),
            "--out" => args.out = PathBuf::from(value("--out")?),
            "--port" => {
                let v = value("--port")?;
                args.port = v.parse().map_err(|_| format!("argument --port: invalid int value: '{}'", v))?;
            }
            "--serve" => args.serve = true,
            "--print-config" => args.print_config = true,
            "-h" | "--help" => {
                println!("{}", USAGE);
                std::process::exit(0);
            }
            other => return Err(format!("unrecognized arguments: {}", other)),
        }
    }
    Ok(args)
}

fn count_html(dir: &Path) -> usize {
    let mut n = 0;
    if let Ok(rd) = std::fs::read_dir(dir) {
        for entry in rd.flatten() {
            let p = entry.path();
            if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                n += count_html(&p);
            } else if p.extension().map_or(false, |e| e == "html") {
                n += 1;
            }
        }
    }
    n
}

fn main() -> ExitCode {
    let args = match parse_args() {
        Ok(a) => a,
        Err(msg) => {
            eprintln!("{}\nbliki: error: {}", USAGE.lines().next().unwrap_or(""), msg);
            return ExitCode::from(2);
        }
    };
    let cfg = site::default_config();
    if args.print_config {
        println!("{}", cfg.to_json());
        return ExitCode::SUCCESS;
    }
    if let Err(msg) = site::build_site(&args.content, &args.out, &cfg) {
        eprintln!("build error: {}", msg);
        return ExitCode::from(1);
    }
    println!("built {} pages \u{2192} {}", count_html(&args.out), args.out.display());
    if args.serve {
        serve::serve(&args.out, args.port, &args.content, &cfg);
    }
    ExitCode::SUCCESS
}
