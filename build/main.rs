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
mod webmention;

use std::path::{Path, PathBuf};
use std::process::ExitCode;

const USAGE: &str = "usage: bliki [--content DIR] [--out DIR] [--base-url URL] [--webmentions FILE] [--serve] [--port N] [--print-config] [--pending-webmentions FEED]

Build the posse-bliki site.

options:
  --content DIR    content vault (default: content)
  --out DIR        output directory (default: public)
  --base-url URL   URL the site is served from, overriding the config
                   (a path such as /posse-bliki is prefixed to every link)
  --webmentions FILE
                   show the received webmentions in FILE (webmention.io jf2)
                   on the posts they respond to
  --serve          build, serve the output at 127.0.0.1:PORT and rebuild on changes
  --port N         port for --serve (default: 8080)
  --print-config   print the site config as JSON and exit
  --pending-webmentions FEED
                   don't build; compare OUT/feed.json with the previously
                   deployed FEED and print one `endpoint<TAB>source<TAB>target`
                   line per new post and syndication target, then exit
  -h, --help       show this help and exit";

struct Args {
    content: PathBuf,
    out: PathBuf,
    base_url: Option<String>,
    webmentions: Option<PathBuf>,
    pending: Option<PathBuf>,
    serve: bool,
    port: u16,
    print_config: bool,
}

fn parse_args() -> Result<Args, String> {
    let mut args = Args {
        content: PathBuf::from("content"),
        out: PathBuf::from("public"),
        base_url: None,
        webmentions: None,
        pending: None,
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
            "--base-url" => args.base_url = Some(value("--base-url")?),
            "--webmentions" => args.webmentions = Some(PathBuf::from(value("--webmentions")?)),
            "--pending-webmentions" => args.pending = Some(PathBuf::from(value("--pending-webmentions")?)),
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
    let mut cfg = site::default_config();
    if let Some(url) = args.base_url {
        cfg.base_url = Box::leak(url.into_boxed_str());
    }
    if args.print_config {
        println!("{}", cfg.to_json());
        return ExitCode::SUCCESS;
    }
    let read = |p: &Path| std::fs::read_to_string(p).map_err(|err| format!("cannot read {}: {}", p.display(), err));
    if let Some(previous) = &args.pending {
        let lines = read(previous)
            .and_then(|prev| Ok((prev, read(&args.out.join("feed.json"))?)))
            .and_then(|(prev, cur)| webmention::pending(&prev, &cur, cfg.syndicate_to));
        return match lines {
            Ok(lines) => {
                for l in lines {
                    println!("{}", l);
                }
                ExitCode::SUCCESS
            }
            Err(msg) => {
                eprintln!("bliki: error: {}", msg);
                ExitCode::from(1)
            }
        };
    }
    if let Some(p) = &args.webmentions {
        match read(p).and_then(|t| webmention::load(&t)) {
            Ok(m) => cfg.mentions = m,
            Err(msg) => {
                eprintln!("build error: {}: {}", p.display(), msg);
                return ExitCode::from(1);
            }
        }
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
