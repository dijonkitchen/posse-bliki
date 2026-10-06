//! Calendar dates without external crates: ISO parsing, weekday math and the
//! formats the feeds need (RFC 822 for RSS, RFC 3339 for JSON Feed).

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Date {
    pub year: i32,
    pub month: u32,
    pub day: u32,
}

/// Python `datetime.date.min`.
pub const MIN: Date = Date { year: 1, month: 1, day: 1 };
pub const EPOCH: Date = Date { year: 1970, month: 1, day: 1 };

fn is_leap(y: i32) -> bool {
    (y % 4 == 0 && y % 100 != 0) || y % 400 == 0
}

fn days_in_month(y: i32, m: u32) -> u32 {
    match m {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if is_leap(y) => 29,
        _ => 28,
    }
}

impl Date {
    /// `date.fromisoformat(s[:10])` for the `YYYY-MM-DD` form.
    pub fn parse(s: &str) -> Option<Date> {
        let head: String = s.chars().take(10).collect();
        let b = head.as_bytes();
        if b.len() != 10 || b[4] != b'-' || b[7] != b'-' {
            return None;
        }
        let num = |r: std::ops::Range<usize>| -> Option<u32> {
            let part = &head[r];
            if part.bytes().all(|c| c.is_ascii_digit()) {
                part.parse().ok()
            } else {
                None
            }
        };
        let (y, m, d) = (num(0..4)?, num(5..7)?, num(8..10)?);
        let y = y as i32;
        if y < 1 || !(1..=12).contains(&m) || d < 1 || d > days_in_month(y, m) {
            return None;
        }
        Some(Date { year: y, month: m, day: d })
    }

    /// Days since 1970-01-01 (Howard Hinnant's days_from_civil).
    fn days_from_epoch(&self) -> i64 {
        let y = self.year as i64 - if self.month <= 2 { 1 } else { 0 };
        let era = if y >= 0 { y } else { y - 399 } / 400;
        let yoe = y - era * 400;
        let m = self.month as i64;
        let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + self.day as i64 - 1;
        let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
        era * 146097 + doe - 719468
    }

    /// 0 = Monday … 6 = Sunday.
    fn weekday(&self) -> usize {
        // 1970-01-01 was a Thursday (3).
        (self.days_from_epoch() + 3).rem_euclid(7) as usize
    }

    /// `date.isoformat()`
    pub fn iso(&self) -> String {
        format!("{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }

    /// `date.strftime('%Y-%m-%d')` (glibc does not pad `%Y`).
    pub fn ymd(&self) -> String {
        format!("{}-{:02}-{:02}", self.year, self.month, self.day)
    }

    /// `email.utils.format_datetime` of midnight UTC.
    pub fn rfc822(&self) -> String {
        const DAYS: [&str; 7] = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];
        const MONTHS: [&str; 12] = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];
        format!(
            "{}, {:02} {} {:04} 00:00:00 +0000",
            DAYS[self.weekday()],
            self.day,
            MONTHS[(self.month - 1) as usize],
            self.year
        )
    }

    /// `datetime.combine(d, time(0, 0), tzinfo=timezone.utc).isoformat()`
    pub fn iso_datetime(&self) -> String {
        format!("{}T00:00:00+00:00", self.iso())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats() {
        let d = Date::parse("2026-05-10").unwrap();
        assert_eq!(d.rfc822(), "Sun, 10 May 2026 00:00:00 +0000");
        assert_eq!(Date::parse("2023-04-11").unwrap().rfc822(), "Tue, 11 Apr 2023 00:00:00 +0000");
        assert_eq!(EPOCH.rfc822(), "Thu, 01 Jan 1970 00:00:00 +0000");
        assert_eq!(Date::parse("0005-01-01").unwrap().rfc822(), "Sat, 01 Jan 0005 00:00:00 +0000");
        assert_eq!(Date::parse("0005-01-01").unwrap().ymd(), "5-01-01");
        assert_eq!(Date::parse("2000-02-29T10:00Z").unwrap().iso_datetime(), "2000-02-29T00:00:00+00:00");
        assert!(Date::parse("2021-02-29").is_none());
        assert!(Date::parse("0000-01-01").is_none());
    }
}
