//! `--serve`: a tiny static file server on 127.0.0.1 plus mtime-polling rebuilds.

use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Component, Path, PathBuf};
use std::thread;
use std::time::{Duration, SystemTime};

use crate::site::{self, Config};

fn content_type(p: &Path) -> &'static str {
    match p.extension().and_then(|e| e.to_str()).unwrap_or("") {
        "html" => "text/html; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "xml" => "application/xml; charset=utf-8",
        "json" => "application/json; charset=utf-8",
        "txt" => "text/plain; charset=utf-8",
        "svg" => "image/svg+xml",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "ico" => "image/x-icon",
        _ => "application/octet-stream",
    }
}

fn percent_decode(s: &str) -> String {
    let b = s.as_bytes();
    let mut out = Vec::with_capacity(b.len());
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'%' && i + 2 < b.len() {
            let hex = |c: u8| (c as char).to_digit(16);
            if let (Some(h), Some(l)) = (hex(b[i + 1]), hex(b[i + 2])) {
                out.push((h * 16 + l) as u8);
                i += 3;
                continue;
            }
        }
        out.push(b[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

fn respond(stream: &mut TcpStream, status: &str, ctype: &str, extra: &str, body: &[u8], head_only: bool) {
    let header = format!(
        "HTTP/1.1 {}\r\nContent-Type: {}\r\nContent-Length: {}\r\nCache-Control: no-cache\r\nConnection: close\r\n{}\r\n",
        status,
        ctype,
        body.len(),
        extra
    );
    let _ = stream.write_all(header.as_bytes());
    if !head_only {
        let _ = stream.write_all(body);
    }
}

fn handle(mut stream: TcpStream, root: &Path) {
    let mut reader = match stream.try_clone() {
        Ok(s) => BufReader::new(s),
        Err(_) => return,
    };
    let mut request_line = String::new();
    if reader.read_line(&mut request_line).is_err() {
        return;
    }
    // drain headers
    loop {
        let mut line = String::new();
        match reader.read_line(&mut line) {
            Ok(0) => break,
            Ok(_) if line == "\r\n" || line == "\n" => break,
            Ok(_) => continue,
            Err(_) => return,
        }
    }
    let mut parts = request_line.split_whitespace();
    let method = parts.next().unwrap_or("");
    let target = parts.next().unwrap_or("/");
    let head_only = method == "HEAD";
    if method != "GET" && !head_only {
        respond(&mut stream, "405 Method Not Allowed", "text/plain; charset=utf-8", "", b"method not allowed\n", false);
        return;
    }
    let path_part = target.split(['?', '#']).next().unwrap_or("/");
    let decoded = percent_decode(path_part);
    let mut fs_path = root.to_path_buf();
    for comp in Path::new(decoded.trim_start_matches('/')).components() {
        match comp {
            Component::Normal(c) => fs_path.push(c),
            Component::CurDir => {}
            _ => {
                respond(&mut stream, "400 Bad Request", "text/plain; charset=utf-8", "", b"bad request\n", head_only);
                return;
            }
        }
    }
    if fs_path.is_dir() {
        if !decoded.ends_with('/') {
            let loc = format!("Location: {}/\r\n", path_part);
            respond(&mut stream, "301 Moved Permanently", "text/plain; charset=utf-8", &loc, b"", head_only);
            return;
        }
        fs_path.push("index.html");
    }
    match fs::read(&fs_path) {
        Ok(body) => respond(&mut stream, "200 OK", content_type(&fs_path), "", &body, head_only),
        Err(_) => {
            let body = fs::read(root.join("404.html")).unwrap_or_else(|_| b"not found\n".to_vec());
            respond(&mut stream, "404 Not Found", "text/html; charset=utf-8", "", &body, head_only);
        }
    }
}

/// Signature of the content tree: every file's path, size and mtime.
fn snapshot(dir: &Path, out: &mut Vec<(PathBuf, u64, Option<SystemTime>)>) {
    if let Ok(rd) = fs::read_dir(dir) {
        let mut entries: Vec<_> = rd.flatten().collect();
        entries.sort_by_key(|e| e.path());
        for entry in entries {
            let p = entry.path();
            match entry.metadata() {
                Ok(m) if m.is_dir() => {
                    out.push((p.clone(), 0, m.modified().ok()));
                    snapshot(&p, out);
                }
                Ok(m) => out.push((p, m.len(), m.modified().ok())),
                Err(_) => {}
            }
        }
    }
}

pub fn serve(out: &Path, port: u16, content: &Path, cfg: &Config) {
    let listener = match TcpListener::bind(("127.0.0.1", port)) {
        Ok(l) => l,
        Err(err) => {
            eprintln!("cannot listen on 127.0.0.1:{}: {}", port, err);
            std::process::exit(1);
        }
    };
    let root = out.to_path_buf();
    thread::spawn(move || {
        for stream in listener.incoming().flatten() {
            let root = root.clone();
            thread::spawn(move || handle(stream, &root));
        }
    });
    println!("serving at http://127.0.0.1:{}/", port);
    let mut last = Vec::new();
    snapshot(content, &mut last);
    loop {
        thread::sleep(Duration::from_millis(500));
        let mut now = Vec::new();
        snapshot(content, &mut now);
        if now != last {
            last = now;
            match site::build_site(content, out, cfg) {
                Ok(()) => println!("rebuilt"),
                Err(msg) => eprintln!("build error: {}", msg),
            }
        }
    }
}
