//! URL normalisation as done by markdown-it-py (`mdurl` parse/format/encode/decode
//! plus punycode for hostnames).

use crate::util::py_strip;

#[derive(Default)]
struct Url {
    protocol: Option<String>,
    slashes: bool,
    auth: Option<String>,
    port: Option<String>,
    hostname: Option<String>,
    hash: Option<String>,
    search: Option<String>,
    pathname: Option<String>,
}

fn hostless(proto: &str) -> bool {
    matches!(proto, "javascript" | "javascript:")
}

fn slashed(proto: &str) -> bool {
    matches!(
        proto,
        "http" | "https" | "ftp" | "gopher" | "file" | "http:" | "https:" | "ftp:" | "gopher:" | "file:"
    )
}

const NON_HOST_CHARS: &[char] = &[
    '%', '/', '?', ';', '#', '\'', '{', '}', '|', '\\', '^', '`', '<', '>', '"', '`', ' ', '\r', '\n', '\t',
];
const HOST_ENDING_CHARS: &[char] = &['/', '?', '#'];

fn is_host_part_char(c: char) -> bool {
    c == '+' || c == '_' || c == '-' || c.is_ascii_alphanumeric()
}

/// `^[+a-z0-9A-Z_-]{0,63}$` (Python `$` also matches before a final newline).
fn host_part_ok(part: &[char]) -> bool {
    let p = if part.last() == Some(&'\n') { &part[..part.len() - 1] } else { part };
    p.len() <= 63 && p.iter().all(|&c| is_host_part_char(c))
}

fn find(hay: &[char], c: char) -> Option<usize> {
    hay.iter().position(|&x| x == c)
}

fn parse(url: &str) -> Url {
    let mut u = Url::default();
    let rest_s = py_strip(url);
    let mut rest: Vec<char> = rest_s.chars().collect();
    let mut lower_proto = String::new();
    let mut proto = String::new();
    // PROTOCOL_PATTERN = ^([a-z0-9.+-]+:) (ignorecase)
    {
        let mut i = 0;
        while i < rest.len() && (rest[i].is_ascii_alphanumeric() || matches!(rest[i], '.' | '+' | '-')) {
            i += 1;
        }
        if i > 0 && i < rest.len() && rest[i] == ':' {
            proto = rest[..=i].iter().collect();
            lower_proto = proto.to_lowercase();
            u.protocol = Some(proto.clone());
            rest.drain(..=i);
        }
    }
    // slashes_denote_host is always true here.
    let slashes = rest.len() >= 2 && rest[0] == '/' && rest[1] == '/';
    if slashes && !(!proto.is_empty() && hostless(&proto)) {
        rest.drain(..2);
        u.slashes = true;
    }
    if !hostless(&proto) && (slashes || (!proto.is_empty() && !slashed(&proto))) {
        let mut host_end: Option<usize> = None;
        for &c in HOST_ENDING_CHARS {
            if let Some(h) = find(&rest, c) {
                if host_end.map_or(true, |e| h < e) {
                    host_end = Some(h);
                }
            }
        }
        let limit = match host_end {
            None => rest.len(),
            Some(e) => (e + 1).min(rest.len()),
        };
        let at_sign = rest[..limit].iter().rposition(|&c| c == '@');
        if let Some(a) = at_sign {
            u.auth = Some(rest[..a].iter().collect());
            rest.drain(..=a);
        }
        let mut host_end: Option<usize> = None;
        for &c in NON_HOST_CHARS {
            if let Some(h) = find(&rest, c) {
                if host_end.map_or(true, |e| h < e) {
                    host_end = Some(h);
                }
            }
        }
        let mut host_end = host_end.unwrap_or(rest.len());
        if host_end > 0 && rest[host_end - 1] == ':' {
            host_end -= 1;
        }
        let host: Vec<char> = rest[..host_end].to_vec();
        rest.drain(..host_end);
        parse_host(&mut u, host);
        let hostname: Vec<char> = u.hostname.take().unwrap_or_default().chars().collect();
        let mut hostname = hostname;
        let ipv6 = hostname.first() == Some(&'[') && hostname.last() == Some(&']');
        if !ipv6 {
            let parts: Vec<Vec<char>> = hostname.split(|&c| c == '.').map(|p| p.to_vec()).collect();
            let l = parts.len();
            let mut i = 0;
            while i < l {
                let part = &parts[i];
                if part.is_empty() {
                    i += 1;
                    continue;
                }
                if !host_part_ok(part) {
                    let newpart: Vec<char> =
                        part.iter().map(|&c| if (c as u32) > 127 { 'x' } else { c }).collect();
                    if !host_part_ok(&newpart) {
                        let mut valid_parts: Vec<Vec<char>> = parts[..i].to_vec();
                        let mut not_host: Vec<Vec<char>> = parts[i + 1..].to_vec();
                        // HOSTNAME_PART_START = ^([+a-z0-9A-Z_-]{0,63})(.*)$ ; `.` excludes \n
                        if !part.contains(&'\n') || part.iter().position(|&c| c == '\n') == Some(part.len() - 1) {
                            let mut k = 0;
                            while k < part.len() && k < 63 && is_host_part_char(part[k]) {
                                k += 1;
                            }
                            let tail_end = if part.last() == Some(&'\n') { part.len() - 1 } else { part.len() };
                            if k <= tail_end {
                                valid_parts.push(part[..k].to_vec());
                                not_host.insert(0, part[k..tail_end].to_vec());
                            }
                        }
                        if !not_host.is_empty() {
                            let mut joined: Vec<char> = Vec::new();
                            for (j, p) in not_host.iter().enumerate() {
                                if j > 0 {
                                    joined.push('.');
                                }
                                joined.extend_from_slice(p);
                            }
                            joined.extend_from_slice(&rest);
                            rest = joined;
                        }
                        let mut hn: Vec<char> = Vec::new();
                        for (j, p) in valid_parts.iter().enumerate() {
                            if j > 0 {
                                hn.push('.');
                            }
                            hn.extend_from_slice(p);
                        }
                        hostname = hn;
                        break;
                    }
                }
                i += 1;
            }
        }
        if hostname.len() > 255 {
            hostname.clear();
        }
        if ipv6 {
            hostname = hostname[1..hostname.len() - 1].to_vec();
        }
        u.hostname = Some(hostname.into_iter().collect());
    }
    if let Some(h) = find(&rest, '#') {
        u.hash = Some(rest[h..].iter().collect());
        rest.truncate(h);
    }
    if let Some(q) = find(&rest, '?') {
        u.search = Some(rest[q..].iter().collect());
        rest.truncate(q);
    }
    if !rest.is_empty() {
        u.pathname = Some(rest.iter().collect());
    }
    if slashed(&lower_proto) && u.hostname.as_deref().map_or(false, |h| !h.is_empty()) && u.pathname.is_none() {
        u.pathname = Some(String::new());
    }
    u
}

fn parse_host(u: &mut Url, mut host: Vec<char>) {
    // PORT_PATTERN = :[0-9]*$  (search: leftmost match)
    let mut found: Option<usize> = None;
    for i in 0..host.len() {
        if host[i] == ':' {
            let tail = &host[i + 1..];
            let tail = if tail.last() == Some(&'\n') { &tail[..tail.len() - 1] } else { tail };
            if tail.iter().all(|c| c.is_ascii_digit()) {
                found = Some(i);
                break;
            }
        }
    }
    if let Some(i) = found {
        // the match itself excludes a trailing newline matched by `$`
        let end = if host.last() == Some(&'\n') && i + 1 < host.len() { host.len() - 1 } else { host.len() };
        let port: String = host[i..end].iter().collect();
        if port != ":" {
            u.port = Some(port[1..].to_string());
        }
        let plen = end - i;
        host.truncate(host.len() - plen);
    }
    if !host.is_empty() {
        u.hostname = Some(host.into_iter().collect());
    }
}

fn format(u: &Url) -> String {
    let mut r = String::new();
    if let Some(p) = &u.protocol {
        r.push_str(p);
    }
    if u.slashes {
        r.push_str("//");
    }
    if let Some(a) = &u.auth {
        if !a.is_empty() {
            r.push_str(a);
            r.push('@');
        }
    }
    if let Some(h) = &u.hostname {
        if h.contains(':') {
            r.push('[');
            r.push_str(h);
            r.push(']');
        } else {
            r.push_str(h);
        }
    }
    if let Some(p) = &u.port {
        if !p.is_empty() {
            r.push(':');
            r.push_str(p);
        }
    }
    for s in [&u.pathname, &u.search, &u.hash].into_iter().flatten() {
        r.push_str(s);
    }
    r
}

fn recode_host(u: &Url) -> bool {
    u.hostname.as_deref().map_or(false, |h| !h.is_empty())
        && match &u.protocol {
            None => true,
            Some(p) => p.is_empty() || matches!(p.as_str(), "http:" | "https:" | "mailto:"),
        }
}

/// markdown-it `normalizeLink`.
pub fn normalize_link(url: &str) -> String {
    let mut u = parse(url);
    if recode_host(&u) {
        if let Some(h) = to_ascii(u.hostname.as_deref().unwrap_or("")) {
            u.hostname = Some(h);
        }
    }
    encode(&format(&u))
}

/// markdown-it `normalizeLinkText`.
pub fn normalize_link_text(url: &str) -> String {
    let mut u = parse(url);
    if recode_host(&u) {
        if let Some(h) = to_unicode(u.hostname.as_deref().unwrap_or("")) {
            u.hostname = Some(h);
        }
    }
    decode(&format(&u), ";/?:@&=+$,#%")
}

/// markdown-it `validateLink`.
pub fn validate_link(url: &str) -> bool {
    let u = py_strip(url).to_lowercase();
    let bad = ["vbscript:", "javascript:", "file:", "data:"].iter().any(|p| u.starts_with(p));
    if !bad {
        return true;
    }
    ["data:image/gif;", "data:image/png;", "data:image/jpeg;", "data:image/webp;"]
        .iter()
        .any(|p| u.starts_with(p))
}

const ENCODE_KEEP: &str = ";/?:@&=+$,-_.!~*'()#";

fn encode(s: &str) -> String {
    let chars: Vec<char> = s.chars().collect();
    let l = chars.len();
    let mut out = String::with_capacity(s.len());
    let mut i = 0;
    while i < l {
        let c = chars[i];
        if c == '%' && i + 2 < l && chars[i + 1].is_ascii_hexdigit() && chars[i + 2].is_ascii_hexdigit() {
            out.push(c);
            out.push(chars[i + 1]);
            out.push(chars[i + 2]);
            i += 3;
            continue;
        }
        if (c as u32) < 128 {
            if c.is_ascii_alphanumeric() || ENCODE_KEEP.contains(c) {
                out.push(c);
            } else {
                out.push_str(&format!("%{:02X}", c as u32));
            }
            i += 1;
            continue;
        }
        let mut buf = [0u8; 4];
        for b in c.encode_utf8(&mut buf).bytes() {
            out.push_str(&format!("%{:02X}", b));
        }
        i += 1;
    }
    out
}

fn hexval(c: char) -> u32 {
    c.to_digit(16).unwrap_or(0)
}

fn decode(s: &str, exclude: &str) -> String {
    let chars: Vec<char> = s.chars().collect();
    let mut out = String::with_capacity(s.len());
    let mut i = 0;
    let is_esc = |j: usize| {
        j + 2 < chars.len() && chars[j] == '%' && chars[j + 1].is_ascii_hexdigit() && chars[j + 2].is_ascii_hexdigit()
    };
    while i < chars.len() {
        if !is_esc(i) {
            out.push(chars[i]);
            i += 1;
            continue;
        }
        // collect the run of %XX
        let mut bytes: Vec<u32> = Vec::new();
        let mut j = i;
        while is_esc(j) {
            bytes.push(hexval(chars[j + 1]) * 16 + hexval(chars[j + 2]));
            j += 3;
        }
        decode_run(&bytes, exclude, &mut out);
        i = j;
    }
    out
}

fn decode_run(b: &[u32], exclude: &str, out: &mut String) {
    let l = b.len();
    let mut i = 0;
    let try_decode = |bs: &[u8], out: &mut String| match std::str::from_utf8(bs) {
        Ok(s) => out.push_str(s),
        Err(_) => {
            for _ in 0..bs.len() {
                out.push('\u{FFFD}');
            }
        }
    };
    while i < l {
        let b1 = b[i];
        if b1 < 0x80 {
            let c = char::from_u32(b1).unwrap_or('\u{FFFD}');
            if exclude.contains(c) {
                out.push_str(&format!("%{:02X}", b1));
            } else {
                out.push(c);
            }
            i += 1;
            continue;
        }
        if (b1 & 0xE0) == 0xC0 && i + 1 < l {
            let b2 = b[i + 1];
            if (b2 & 0xC0) == 0x80 {
                try_decode(&[b1 as u8, b2 as u8], out);
                i += 2;
                continue;
            }
        }
        if (b1 & 0xF0) == 0xE0 && i + 2 < l {
            let (b2, b3) = (b[i + 1], b[i + 2]);
            if (b2 & 0xC0) == 0x80 && (b3 & 0xC0) == 0x80 {
                try_decode(&[b1 as u8, b2 as u8, b3 as u8], out);
                i += 3;
                continue;
            }
        }
        if (b1 & 0xF8) == 0xF0 && i + 3 < l {
            let (b2, b3, b4) = (b[i + 1], b[i + 2], b[i + 3]);
            if (b2 & 0xC0) == 0x80 && (b3 & 0xC0) == 0x80 && (b4 & 0xC0) == 0x80 {
                try_decode(&[b1 as u8, b2 as u8, b3 as u8, b4 as u8], out);
                i += 4;
                continue;
            }
        }
        out.push('\u{FFFD}');
        i += 1;
    }
}

// --- punycode (RFC 3492, as Python's `punycode` codec) ---

fn map_domain(s: &str, f: impl Fn(&str) -> Option<String>) -> Option<String> {
    let parts: Vec<&str> = s.split('@').collect();
    let mut result = String::new();
    let mut s = s;
    if parts.len() > 1 {
        result.push_str(parts[0]);
        result.push('@');
        s = parts[1];
    }
    let labels: Vec<&str> = s.split(['.', '\u{3002}', '\u{FF0E}', '\u{FF61}']).collect();
    let mut enc = Vec::with_capacity(labels.len());
    for l in labels {
        enc.push(f(l)?);
    }
    result.push_str(&enc.join("."));
    Some(result)
}

fn to_ascii(s: &str) -> Option<String> {
    map_domain(s, |label| {
        if label.chars().any(|c| (c as u32) > 0x7E) {
            Some(format!("xn--{}", puny_encode(label)?))
        } else {
            Some(label.to_string())
        }
    })
}

fn to_unicode(s: &str) -> Option<String> {
    map_domain(s, |label| {
        if let Some(rest) = label.strip_prefix("xn--") {
            puny_decode(&rest.to_lowercase())
        } else {
            Some(label.to_string())
        }
    })
}

const DIGITS: &[u8; 36] = b"abcdefghijklmnopqrstuvwxyz0123456789";

fn t_of(j: i128, bias: i128) -> i128 {
    (36 * (j + 1) - bias).clamp(1, 26)
}

fn adapt(mut delta: i128, first: bool, numchars: i128) -> i128 {
    delta = if first { delta / 700 } else { delta / 2 };
    delta += delta / numchars;
    let mut divisions = 0;
    while delta > 455 {
        delta /= 35;
        divisions += 36;
    }
    divisions + (36 * delta / (delta + 38))
}

fn puny_encode(s: &str) -> Option<String> {
    let chars: Vec<char> = s.chars().collect();
    let base: String = chars.iter().filter(|c| (**c as u32) < 128).collect();
    let mut extended: Vec<char> = chars.iter().copied().filter(|c| (*c as u32) >= 128).collect();
    extended.sort();
    extended.dedup();
    // insertion_unsort
    let mut deltas: Vec<i128> = Vec::new();
    let mut oldchar: i128 = 0x80;
    let mut oldindex: i128 = -1;
    for &c in &extended {
        let ch = c as i128;
        let curlen = chars.iter().filter(|x| (**x as i128) < ch).count() as i128;
        let mut delta = (curlen + 1) * (ch - oldchar);
        let mut index: i128 = -1;
        let mut pos: i128 = -1;
        loop {
            // selective_find
            let l = chars.len() as i128;
            let mut found = false;
            loop {
                pos += 1;
                if pos == l {
                    break;
                }
                let x = chars[pos as usize];
                if x == c {
                    index += 1;
                    found = true;
                    break;
                } else if x < c {
                    index += 1;
                }
            }
            if !found {
                break;
            }
            delta += index - oldindex;
            deltas.push(delta - 1);
            oldindex = index;
            delta = 0;
        }
        oldchar = ch;
    }
    let mut out = Vec::new();
    let mut bias = 72;
    let baselen = base.chars().count() as i128;
    for (points, &delta) in deltas.iter().enumerate() {
        let mut n = delta;
        let mut j = 0;
        loop {
            let t = t_of(j, bias);
            if n < t {
                out.push(DIGITS[n as usize]);
                break;
            }
            out.push(DIGITS[(t + ((n - t) % (36 - t))) as usize]);
            n = (n - t) / (36 - t);
            j += 1;
        }
        bias = adapt(delta, points == 0, baselen + points as i128 + 1);
    }
    let ext = String::from_utf8(out).ok()?;
    if base.is_empty() {
        Some(ext)
    } else {
        Some(format!("{}-{}", base, ext))
    }
}

fn puny_decode(s: &str) -> Option<String> {
    if !s.is_ascii() {
        return None;
    }
    let (base, extended) = match s.rfind('-') {
        None => (String::new(), s.to_uppercase()),
        Some(p) => (s[..p].to_string(), s[p + 1..].to_uppercase()),
    };
    let ext: Vec<u8> = extended.into_bytes();
    let mut out: Vec<char> = base.chars().collect();
    let mut ch: i128 = 0x80;
    let mut pos: i128 = -1;
    let mut bias: i128 = 72;
    let mut extpos = 0usize;
    while extpos < ext.len() {
        // decode_generalized_number
        let mut result: i128 = 0;
        let mut w: i128 = 1;
        let mut j: i128 = 0;
        let mut p = extpos;
        let delta = loop {
            let c = *ext.get(p)?;
            p += 1;
            let digit = match c {
                b'A'..=b'Z' => (c - b'A') as i128,
                b'0'..=b'9' => (c as i128) - 22,
                _ => return None,
            };
            let t = t_of(j, bias);
            result = result.checked_add(digit.checked_mul(w)?)?;
            if digit < t {
                break result;
            }
            w = w.checked_mul(36 - t)?;
            j += 1;
        };
        pos += delta + 1;
        let len1 = out.len() as i128 + 1;
        ch = ch.checked_add(pos / len1)?;
        if ch > 0x10FFFF {
            return None;
        }
        pos %= len1;
        let c = char::from_u32(ch as u32)?;
        out.insert(pos as usize, c);
        bias = adapt(delta, extpos == 0, out.len() as i128);
        extpos = p;
    }
    Some(out.into_iter().collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn punycode_roundtrip() {
        assert_eq!(puny_encode("bücher").unwrap(), "bcher-kva");
        assert_eq!(puny_decode("bcher-kva").unwrap(), "bücher");
        assert_eq!(normalize_link("https://bücher.example/ä?x=1#y"), "https://xn--bcher-kva.example/%C3%A4?x=1#y");
    }
}
