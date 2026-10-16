//! Pixel size of the images under `content/images/`, read from the file
//! header (PNG, GIF, JPEG, WebP), so pages can give every `<img>` a `width`
//! and `height` and the browser can reserve its space before it loads
//! (spec/performance.md). Standard library only (ADR 0007).

/// `(width, height)` in pixels, or `None` for a format this doesn't read.
pub fn size(b: &[u8]) -> Option<(u32, u32)> {
    if b.starts_with(b"\x89PNG\r\n\x1a\n") {
        // IHDR is always the first chunk: width and height are big-endian u32.
        return Some((be32(b, 16)?, be32(b, 20)?));
    }
    if b.starts_with(b"GIF87a") || b.starts_with(b"GIF89a") {
        return Some((le16(b, 6)?, le16(b, 8)?));
    }
    if b.starts_with(b"\xff\xd8") {
        return jpeg(b);
    }
    if b.len() >= 30 && &b[0..4] == b"RIFF" && &b[8..12] == b"WEBP" {
        return webp(b);
    }
    None
}

/// Walk the JPEG markers to the first start-of-frame, which holds the size.
fn jpeg(b: &[u8]) -> Option<(u32, u32)> {
    let mut i = 2;
    while i + 4 <= b.len() {
        if b[i] != 0xff {
            return None;
        }
        let marker = b[i + 1];
        if marker == 0xff {
            i += 1; // fill byte
            continue;
        }
        let len = be16(b, i + 2)? as usize;
        // SOF0..SOF15, except DHT (c4), JPG (c8) and DAC (cc).
        if (0xc0..=0xcf).contains(&marker) && !matches!(marker, 0xc4 | 0xc8 | 0xcc) {
            return Some((be16(b, i + 7)?, be16(b, i + 5)?));
        }
        i += 2 + len;
    }
    None
}

fn webp(b: &[u8]) -> Option<(u32, u32)> {
    match &b[12..16] {
        b"VP8 " => Some((le16(b, 26)? & 0x3fff, le16(b, 28)? & 0x3fff)),
        b"VP8L" => {
            let bits = u32::from_le_bytes(b.get(21..25)?.try_into().ok()?);
            Some(((bits & 0x3fff) + 1, ((bits >> 14) & 0x3fff) + 1))
        }
        b"VP8X" => Some((le24(b, 24)? + 1, le24(b, 27)? + 1)),
        _ => None,
    }
}

fn be16(b: &[u8], i: usize) -> Option<u32> {
    Some(u32::from(u16::from_be_bytes(b.get(i..i + 2)?.try_into().ok()?)))
}

fn be32(b: &[u8], i: usize) -> Option<u32> {
    Some(u32::from_be_bytes(b.get(i..i + 4)?.try_into().ok()?))
}

fn le16(b: &[u8], i: usize) -> Option<u32> {
    Some(u32::from(u16::from_le_bytes(b.get(i..i + 2)?.try_into().ok()?)))
}

fn le24(b: &[u8], i: usize) -> Option<u32> {
    let s = b.get(i..i + 3)?;
    Some(u32::from(s[0]) | u32::from(s[1]) << 8 | u32::from(s[2]) << 16)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn png() {
        let mut b = b"\x89PNG\r\n\x1a\n\0\0\0\x0dIHDR".to_vec();
        b.extend_from_slice(&1062u32.to_be_bytes());
        b.extend_from_slice(&218u32.to_be_bytes());
        assert_eq!(size(&b), Some((1062, 218)));
    }

    #[test]
    fn gif() {
        assert_eq!(size(b"GIF89a\x20\x03\x00\x02"), Some((800, 512)));
    }

    #[test]
    fn jpeg_skips_to_the_frame_header() {
        // SOI, an APP0 segment of length 4, then SOF2 (progressive).
        let b = b"\xff\xd8\xff\xe0\x00\x04\x00\x00\xff\xc2\x00\x11\x08\x02\x00\x03\x20";
        assert_eq!(size(b), Some((800, 512)));
    }

    #[test]
    fn unknown_is_none() {
        assert_eq!(size(b"not an image"), None);
    }
}
