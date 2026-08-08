#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

use std::fmt;

#[cfg(feature = "chrono")]
use chrono::TimeZone;

/// Which field of the cron expression caused a parse error.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CronFieldName {
    /// Seconds (6-field cron only).
    Seconds,
    /// Minutes field.
    Minutes,
    /// Hours field.
    Hours,
    /// Day-of-month field.
    DayOfMonth,
    /// Month field.
    Month,
    /// Day-of-week field.
    DayOfWeek,
}

impl fmt::Display for CronFieldName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Seconds => "seconds",
            Self::Minutes => "minutes",
            Self::Hours => "hours",
            Self::DayOfMonth => "day-of-month",
            Self::Month => "month",
            Self::DayOfWeek => "day-of-week",
        };
        f.write_str(s)
    }
}

/// A parse error with exact location information.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CronError {
    /// Byte offset in the input where the error occurred.
    pub position: usize,
    /// Which cron field was being parsed when the error occurred.
    pub field: CronFieldName,
    /// Description of what was expected.
    pub expected: &'static str,
    /// The character found, or `None` if the input ended unexpectedly.
    pub found: Option<char>,
}

impl fmt::Display for CronError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.found {
            Some(c) => write!(
                f,
                "cron parse error in {} field at position {}: expected {}, found {:?}",
                self.field, self.position, self.expected, c
            ),
            None => write!(
                f,
                "cron parse error in {} field at position {}: expected {}, found end of input",
                self.field, self.position, self.expected
            ),
        }
    }
}

impl std::error::Error for CronError {}

/// A single cron field value (wildcard, number, range, step, or list).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum CronField {
    /// Wildcard `*` — matches all values.
    Any,
    /// A specific numeric value, e.g. `5`.
    Value(u8),
    /// An inclusive range, e.g. `1-5`.
    Range(u8, u8),
    /// A step expression, e.g. `*/2` or `1-5/2`.
    Step(Box<CronField>, u8),
    /// A comma-separated list, e.g. `1,3,5`.
    List(Vec<CronField>),
}

/// A parsed cron expression (5-field or 6-field with seconds).
///
/// # Example
///
/// ```
/// use tpt_cron_parse::CronExpr;
///
/// let expr = CronExpr::parse("0 9 * * 1").unwrap();
/// assert_eq!(expr.to_human_readable(), "Every Monday at 9:00 AM");
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CronExpr {
    /// Seconds field — `Some` for 6-field cron, `None` for 5-field.
    pub seconds: Option<CronField>,
    /// Minutes field.
    pub minutes: CronField,
    /// Hours field.
    pub hours: CronField,
    /// Day-of-month field.
    pub dom: CronField,
    /// Month field.
    pub month: CronField,
    /// Day-of-week field.
    pub dow: CronField,
}

impl CronExpr {
    /// Parse a cron expression string (5-field or 6-field).
    ///
    /// Fields may be separated by spaces or tabs. Every value is validated
    /// against the range of its field (minutes 0-59, hours 0-23, day-of-month
    /// 1-31, month 1-12, day-of-week 0-7 where both 0 and 7 are Sunday, seconds
    /// 0-59), month names `JAN`-`DEC` and day names `SUN`-`SAT` are accepted
    /// case-insensitively, and the shorthand aliases `@yearly`, `@annually`,
    /// `@monthly`, `@weekly`, `@daily`, `@midnight` and `@hourly` are expanded
    /// before parsing. `@reboot` is not a periodic schedule and is rejected.
    ///
    /// Vixie-cron extensions `L` (last day of month/week), `W` (nearest weekday),
    /// `#` (nth weekday) and `?` (unspecified field) are intentionally not
    /// supported and are rejected as parse errors — this parser only understands
    /// numeric values, `*`/`-`/`/`/`,` and the named months/days above.
    ///
    /// # Example
    ///
    /// ```
    /// use tpt_cron_parse::CronExpr;
    ///
    /// let expr = CronExpr::parse("*/5 * * * *").unwrap();
    /// assert_eq!(expr.to_human_readable(), "Every 5 minutes");
    ///
    /// let expr = CronExpr::parse("@daily").unwrap();
    /// assert_eq!(expr.to_human_readable(), "Every day at 12:00 AM");
    ///
    /// let expr = CronExpr::parse("0 9 * JAN MON").unwrap();
    /// assert_eq!(expr.month, CronExpr::parse("0 9 * 1 1").unwrap().month);
    ///
    /// assert!(CronExpr::parse("0 99 * * *").is_err());
    /// assert!(CronExpr::parse("* * * * 1garbage").is_err());
    /// ```
    pub fn parse(s: &str) -> Result<CronExpr, CronError> {
        parse_expr(s)
    }

    /// Returns `true` if this is a 6-field cron expression (with seconds).
    pub fn is_6_field(&self) -> bool {
        self.seconds.is_some()
    }

    /// Convert this cron expression to a human-readable English description.
    ///
    /// # Example
    ///
    /// ```
    /// use tpt_cron_parse::CronExpr;
    ///
    /// assert_eq!(CronExpr::parse("* * * * *").unwrap().to_human_readable(), "Every minute");
    /// assert_eq!(CronExpr::parse("0 * * * *").unwrap().to_human_readable(), "Every hour");
    /// assert_eq!(CronExpr::parse("0 9 * * *").unwrap().to_human_readable(), "Every day at 9:00 AM");
    /// assert_eq!(CronExpr::parse("0 0 1 1 *").unwrap().to_human_readable(), "At 12:00 AM on January 1st");
    /// ```
    pub fn to_human_readable(&self) -> String {
        human_readable(self)
    }

    /// Return the first time strictly after `after` that this schedule fires.
    ///
    /// Only available with the `chrono` feature (the crate stays dependency-free
    /// by default). The search is bounded to at least 8 years ahead — the
    /// maximum period of a cron schedule, since 2096 is a leap year and the next
    /// leap February is 2104, an 8-year gap.
    ///
    /// ```rust,ignore
    /// use tpt_cron_parse::CronExpr;
    /// use chrono::{TimeZone, Utc};
    /// let expr = CronExpr::parse("0 9 * * 1-5").unwrap();
    /// let after = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();
    /// let next = expr.next_after(after).unwrap(); // next weekday at 09:00
    /// ```
    #[cfg(feature = "chrono")]
    pub fn next_after(
        &self,
        after: chrono::DateTime<chrono::Utc>,
    ) -> Option<chrono::DateTime<chrono::Utc>> {
        let next = self.next_after_naive(after.naive_utc())?;
        Some(chrono::Utc.from_utc_datetime(&next))
    }

    /// Return the first time strictly after `after` that this schedule fires,
    /// interpreting the cron fields as wall-clock time in the timezone of
    /// `after`.
    ///
    /// This is the timezone-aware variant of [`CronExpr::next_after`]: the
    /// schedule is evaluated against the local time in `Tz`, so `0 9 * * *`
    /// next fires at 09:00 in `Tz`, not at 09:00 UTC. Only available with the
    /// `chrono` feature.
    ///
    /// ```rust,ignore
    /// use tpt_cron_parse::CronExpr;
    /// use chrono::{FixedOffset, TimeZone};
    /// let expr = CronExpr::parse("0 9 * * *").unwrap();
    /// let tz = FixedOffset::east_opt(8 * 3600).unwrap();
    /// let after = tz.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();
    /// let next = expr.next_after_tz(after).unwrap(); // 09:00 in +08:00
    /// ```
    #[cfg(feature = "chrono")]
    pub fn next_after_tz<Tz: chrono::TimeZone>(
        &self,
        after: chrono::DateTime<Tz>,
    ) -> Option<chrono::DateTime<Tz>> {
        let tz = after.timezone();
        let next_naive = self.next_after_naive(after.naive_local())?;
        tz.from_local_datetime(&next_naive)
            .single()
            .or_else(|| tz.from_local_datetime(&next_naive).earliest())
    }

    /// Search the wall-clock schedule forward from `after` (a naive local time)
    /// and return the next matching `NaiveDateTime`. Shared by [`CronExpr::next_after`]
    /// (mapped through UTC) and [`CronExpr::next_after_tz`].
    #[cfg(feature = "chrono")]
    fn next_after_naive(&self, after: chrono::NaiveDateTime) -> Option<chrono::NaiveDateTime> {
        use chrono::{Datelike, Timelike};
        let seconds_set = self.seconds.as_ref().map(|s| s.expand(0, 59));
        let minutes = self.minutes.expand(0, 59);
        let hours = self.hours.expand(0, 23);
        let doms = self.dom.expand(1, 31);
        let months = self.month.expand(1, 12);
        let dows = self.dow.expand(0, 6);
        // Vixie cron OR-rule: day-of-month and day-of-week are OR-ed only when
        // *both* are explicitly restricted. A bare `*` or a step on `*` (e.g.
        // `*/2`) counts as unrestricted ("star"), in which case that field is
        // AND-ed with the other. This matches `0 0 */2 * MON` firing only on
        // Mondays, not on every even day too.
        let dom_unrestricted = field_is_unrestricted(&self.dom);
        let dow_unrestricted = field_is_unrestricted(&self.dow);
        let start = after.with_nanosecond(0)? + chrono::Duration::seconds(1);
        let first_minute = start.with_second(0)?;
        // The longest possible gap between matching dates is bounded by the
        // leap-year cycle: 2096 is a leap year and the next leap February is
        // 2104, an 8-year span, so search at least 8*366+1 days ahead.
        let limit = after + chrono::Duration::days(8 * 366 + 1);
        let mut cur = first_minute;

        while cur <= limit {
            let m = cur.minute() as u8;
            let h = cur.hour() as u8;
            let dom = cur.day() as u8;
            let mon = cur.month() as u8;
            let dow = match cur.weekday() {
                chrono::Weekday::Sun => 0,
                chrono::Weekday::Mon => 1,
                chrono::Weekday::Tue => 2,
                chrono::Weekday::Wed => 3,
                chrono::Weekday::Thu => 4,
                chrono::Weekday::Fri => 5,
                chrono::Weekday::Sat => 6,
            };
            let day_ok = match (dom_unrestricted, dow_unrestricted) {
                (true, true) => true,
                (false, true) => doms.contains(&dom),
                (true, false) => dows.contains(&dow),
                (false, false) => doms.contains(&dom) || dows.contains(&dow),
            };

            if minutes.contains(&m) && hours.contains(&h) && months.contains(&mon) && day_ok {
                match &seconds_set {
                    Some(secs) => {
                        let floor = if cur == first_minute {
                            start.second() as u8
                        } else {
                            0
                        };
                        if let Some(&s) = secs.iter().find(|&&s| s >= floor) {
                            return cur.with_second(s as u32);
                        }
                    }
                    None => {
                        if cur >= start {
                            return Some(cur);
                        }
                    }
                }
            }

            cur += chrono::Duration::minutes(1);
        }
        None
    }
}

/// An iterator over the firing times of a [`CronExpr`], strictly after a starting
/// point. Each call to [`Iterator::next`] advances past the previously yielded time.
///
/// Only available with the `chrono` feature.
#[cfg(feature = "chrono")]
pub struct Upcoming {
    expr: CronExpr,
    after: chrono::DateTime<chrono::Utc>,
}

#[cfg(feature = "chrono")]
impl Iterator for Upcoming {
    type Item = chrono::DateTime<chrono::Utc>;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.expr.next_after(self.after)?;
        self.after = next;
        Some(next)
    }
}

#[cfg(feature = "chrono")]
impl CronExpr {
    /// Return an [`Upcoming`] iterator yielding the firing times of this schedule
    /// strictly after `after`.
    ///
    /// ```rust,ignore
    /// use tpt_cron_parse::CronExpr;
    /// use chrono::{TimeZone, Utc};
    /// let expr = CronExpr::parse("0 9 * * *").unwrap();
    /// let after = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();
    /// let next_three: Vec<_> = expr.upcoming(after).take(3).collect();
    /// ```
    pub fn upcoming(&self, after: chrono::DateTime<chrono::Utc>) -> Upcoming {
        Upcoming {
            expr: self.clone(),
            after,
        }
    }
}

impl CronField {
    /// Expand this field into the sorted, de-duplicated set of values it permits
    /// within the inclusive `[min, max]` bounds. Values outside the bounds are
    /// clamped away, and a zero step yields no values.
    ///
    /// # Example
    ///
    /// ```
    /// use tpt_cron_parse::CronExpr;
    ///
    /// let expr = CronExpr::parse("0 1-5/2 * * *").unwrap();
    /// assert_eq!(expr.hours.expand(0, 23), vec![1, 3, 5]);
    /// ```
    pub fn expand(&self, min: u8, max: u8) -> Vec<u8> {
        let mut out = Vec::new();
        self.expand_into(min, max, &mut out);
        out.sort_unstable();
        out.dedup();
        out
    }

    fn expand_into(&self, min: u8, max: u8, out: &mut Vec<u8>) {
        match self {
            CronField::Any => out.extend(min..=max),
            CronField::Value(n) => {
                if *n >= min && *n <= max {
                    out.push(*n);
                }
            }
            CronField::Range(a, b) => {
                let (lo, hi) = ((*a).max(min), (*b).min(max));
                if lo <= hi {
                    out.extend(lo..=hi);
                }
            }
            CronField::Step(base, step) => {
                if *step == 0 {
                    return;
                }
                let (start, end) = match base.as_ref() {
                    CronField::Any => (min, max),
                    CronField::Value(n) => ((*n).max(min), max),
                    CronField::Range(a, b) => ((*a).max(min), (*b).min(max)),
                    other => {
                        let vals = other.expand(min, max);
                        out.extend(vals.into_iter().step_by(*step as usize));
                        return;
                    }
                };
                let mut v = start;
                while v <= end {
                    out.push(v);
                    match v.checked_add(*step) {
                        Some(n) => v = n,
                        None => break,
                    }
                }
            }
            CronField::List(items) => {
                for it in items {
                    it.expand_into(min, max, out);
                }
            }
        }
    }
}

// ---- Parser internals ----

const MONTH_NAMES: [&str; 12] = [
    "JAN", "FEB", "MAR", "APR", "MAY", "JUN", "JUL", "AUG", "SEP", "OCT", "NOV", "DEC",
];

const DOW_NAMES: [&str; 7] = ["SUN", "MON", "TUE", "WED", "THU", "FRI", "SAT"];

const SHORTHANDS: [(&str, &str); 7] = [
    ("@yearly", "0 0 1 1 *"),
    ("@annually", "0 0 1 1 *"),
    ("@monthly", "0 0 1 * *"),
    ("@weekly", "0 0 * * 0"),
    ("@daily", "0 0 * * *"),
    ("@midnight", "0 0 * * *"),
    ("@hourly", "0 * * * *"),
];

fn field_bounds(field: CronFieldName) -> (u8, u8) {
    match field {
        CronFieldName::Seconds | CronFieldName::Minutes => (0, 59),
        CronFieldName::Hours => (0, 23),
        CronFieldName::DayOfMonth => (1, 31),
        CronFieldName::Month => (1, 12),
        // Day-of-week runs 0-6 (Sun-Sat). `7` is a cron alias for Sunday (`0`)
        // and is normalised to `0` at parse time, so the upper bound here is 6.
        CronFieldName::DayOfWeek => (0, 6),
    }
}

fn field_expected(field: CronFieldName) -> &'static str {
    match field {
        CronFieldName::Seconds | CronFieldName::Minutes => "value 0-59",
        CronFieldName::Hours => "value 0-23",
        CronFieldName::DayOfMonth => "value 1-31",
        CronFieldName::Month => "value 1-12 or name JAN-DEC",
        CronFieldName::DayOfWeek => "value 0-7 or name SUN-SAT",
    }
}

fn named_value(field: CronFieldName, name: &str) -> Option<u8> {
    match field {
        CronFieldName::Month => MONTH_NAMES
            .iter()
            .position(|n| name.eq_ignore_ascii_case(n))
            .map(|i| i as u8 + 1),
        CronFieldName::DayOfWeek => DOW_NAMES
            .iter()
            .position(|n| name.eq_ignore_ascii_case(n))
            .map(|i| i as u8),
        _ => None,
    }
}

fn split_fields(input: &str) -> Vec<(usize, &str)> {
    let bytes = input.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        while i < bytes.len() && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        let start = i;
        while i < bytes.len() && !bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        if i > start {
            out.push((start, &input[start..i]));
        }
    }
    out
}

fn expand_shorthand(token: &str, position: usize) -> Result<&'static str, CronError> {
    if let Some((_, expansion)) = SHORTHANDS
        .iter()
        .find(|(name, _)| token.eq_ignore_ascii_case(name))
    {
        return Ok(expansion);
    }
    let expected = if token.eq_ignore_ascii_case("@reboot") {
        "a periodic schedule (@reboot has no next run time and is not supported)"
    } else {
        "one of @yearly, @annually, @monthly, @weekly, @daily, @midnight, @hourly"
    };
    Err(CronError {
        position,
        field: CronFieldName::Minutes,
        expected,
        found: token.chars().next(),
    })
}

fn parse_expr(input: &str) -> Result<CronExpr, CronError> {
    let fields = split_fields(input);

    if let Some(&(offset, token)) = fields.first() {
        if token.starts_with('@') {
            if let Some(&(pos, rest)) = fields.get(1) {
                return Err(CronError {
                    position: pos,
                    field: CronFieldName::Minutes,
                    expected: "end of input after a shorthand alias",
                    found: rest.chars().next(),
                });
            }
            return parse_expr(expand_shorthand(token, offset)?);
        }
    }

    if fields.len() != 5 && fields.len() != 6 {
        return Err(CronError {
            position: 0,
            field: CronFieldName::Minutes,
            expected: "5 or 6 whitespace-separated fields",
            found: None,
        });
    }

    let is_6 = fields.len() == 6;
    let base = usize::from(is_6);
    let seconds = if is_6 {
        Some(parse_one(fields[0], CronFieldName::Seconds)?)
    } else {
        None
    };
    let minutes = parse_one(fields[base], CronFieldName::Minutes)?;
    let hours = parse_one(fields[base + 1], CronFieldName::Hours)?;
    let dom = parse_one(fields[base + 2], CronFieldName::DayOfMonth)?;
    let month = parse_one(fields[base + 3], CronFieldName::Month)?;
    let dow = parse_one(fields[base + 4], CronFieldName::DayOfWeek)?;

    Ok(CronExpr {
        seconds,
        minutes,
        hours,
        dom,
        month,
        dow,
    })
}

fn parse_one((offset, text): (usize, &str), field: CronFieldName) -> Result<CronField, CronError> {
    FieldParser {
        text,
        offset,
        pos: 0,
        field,
    }
    .parse()
}

struct FieldParser<'a> {
    text: &'a str,
    offset: usize,
    pos: usize,
    field: CronFieldName,
}

impl FieldParser<'_> {
    fn byte(&self) -> Option<u8> {
        self.text.as_bytes().get(self.pos).copied()
    }

    fn error(&self, expected: &'static str) -> CronError {
        self.error_at(self.pos, expected)
    }

    fn error_at(&self, position: usize, expected: &'static str) -> CronError {
        CronError {
            position: self.offset + position,
            field: self.field,
            expected,
            found: self.text[position..].chars().next(),
        }
    }

    fn parse_number(&mut self, expected: &'static str) -> Result<u8, CronError> {
        let start = self.pos;
        while self.byte().is_some_and(|b| b.is_ascii_digit()) {
            self.pos += 1;
        }
        if self.pos == start {
            return Err(self.error(expected));
        }
        self.text[start..self.pos]
            .parse::<u8>()
            .map_err(|_| self.error_at(start, expected))
    }

    fn parse_value(&mut self) -> Result<u8, CronError> {
        let expected = field_expected(self.field);
        let start = self.pos;

        if self.byte().is_some_and(|b| b.is_ascii_alphabetic()) {
            while self.byte().is_some_and(|b| b.is_ascii_alphabetic()) {
                self.pos += 1;
            }
            let name = &self.text[start..self.pos];
            return match named_value(self.field, name) {
                Some(v) => Ok(v),
                None => {
                    self.pos = start;
                    Err(self.error(expected))
                }
            };
        }

        let value = self.parse_number(expected)?;
        // Day-of-week `7` is a cron alias for Sunday (`0`; both are 0-based
        // Sunday). Normalise it so the stored value is always in 0-6.
        let value = if self.field == CronFieldName::DayOfWeek && value == 7 {
            0
        } else {
            value
        };
        let (min, max) = field_bounds(self.field);
        if value < min || value > max {
            return Err(self.error_at(start, expected));
        }
        Ok(value)
    }

    fn parse_item(&mut self) -> Result<CronField, CronError> {
        let base = if self.byte() == Some(b'*') {
            self.pos += 1;
            CronField::Any
        } else {
            let n_start = self.pos;
            let n = self.parse_value()?;
            if self.byte() == Some(b'-') {
                self.pos += 1;
                let end = self.parse_value()?;
                if n > end {
                    return Err(CronError {
                        position: self.offset + n_start,
                        field: self.field,
                        expected: "ascending range (start <= end)",
                        found: Some('-'),
                    });
                }
                CronField::Range(n, end)
            } else {
                CronField::Value(n)
            }
        };

        if self.byte() == Some(b'/') {
            self.pos += 1;
            let step_start = self.pos;
            let step = self.parse_number("non-zero step value")?;
            if step == 0 {
                return Err(self.error_at(step_start, "non-zero step value"));
            }
            return Ok(CronField::Step(Box::new(base), step));
        }
        Ok(base)
    }

    fn parse(mut self) -> Result<CronField, CronError> {
        let mut items: Vec<CronField> = Vec::new();
        loop {
            let item = self.parse_item()?;
            items.push(item);
            if self.byte() == Some(b',') {
                self.pos += 1;
            } else {
                break;
            }
        }
        if self.pos != self.text.len() {
            return Err(self.error("',' or end of field"));
        }
        if items.len() == 1 {
            Ok(items.remove(0))
        } else {
            Ok(CronField::List(items))
        }
    }
}

// ---- Human-readable conversion ----

/// Returns `true` if the field is the wildcard `*` or a step rooted at `*` (e.g.
/// `*/2`). Such fields are treated as "unrestricted" for the Vixie cron
/// day-of-month/day-of-week OR-rule.
fn field_is_unrestricted(f: &CronField) -> bool {
    match f {
        CronField::Any => true,
        CronField::Step(base, _) => matches!(base.as_ref(), CronField::Any),
        _ => false,
    }
}

fn is_any(f: &CronField) -> bool {
    matches!(f, CronField::Any)
}

/// Returns `Some("1st, 3rd, 5th")` when `f` is an explicit list of distinct day
/// numbers (e.g. `1,3,5`), used to render `0 9 * * 1,3,5` as
/// "At 9:00 AM on the 1st, 3rd, 5th of every month". Returns `None` for any
/// other shape (ranges, single values, wildcards, steps).
fn list_days_phrase(f: &CronField) -> Option<String> {
    let CronField::List(items) = f else {
        return None;
    };
    if items.is_empty() || !items.iter().all(|i| matches!(i, CronField::Value(_))) {
        return None;
    }
    let mut sorted = items.clone();
    sorted.sort();
    let parts: Vec<String> = sorted
        .iter()
        .map(|i| match i {
            CronField::Value(d) => ordinal(*d),
            _ => unreachable!(),
        })
        .collect();
    Some(parts.join(", "))
}

/// Returns `Some(name)` if the field denotes Monday–Friday (days 1–5) as either
/// a contiguous range `1-5` or an explicit list `1,2,3,4,5`. Used to render
/// `0 9 * * 1-5` as "Every weekday at 9:00 AM" instead of a raw reconstruction.
fn weekday_field(f: &CronField) -> Option<&'static str> {
    match f {
        CronField::Range(1, 5) => Some("weekday"),
        CronField::List(items) if items.len() == 5 => {
            let mut sorted = items.clone();
            sorted.sort();
            if sorted
                == [
                    CronField::Value(1),
                    CronField::Value(2),
                    CronField::Value(3),
                    CronField::Value(4),
                    CronField::Value(5),
                ]
            {
                Some("weekday")
            } else {
                None
            }
        }
        _ => None,
    }
}

fn is_zero(f: &CronField) -> bool {
    matches!(f, CronField::Value(0))
}

fn format_time(
    seconds: Option<&CronField>,
    hours: &CronField,
    minutes: &CronField,
) -> Option<String> {
    if let (CronField::Value(h), CronField::Value(m)) = (hours, minutes) {
        let period = if *h < 12 { "AM" } else { "PM" };
        let h12 = match h {
            0 => 12,
            h if *h <= 12 => *h as u32,
            h => (*h - 12) as u32,
        };
        let sec = match seconds {
            Some(CronField::Value(s)) if *s != 0 => format!(":{:02}", s),
            _ => String::new(),
        };
        Some(format!("{}:{:02}{} {}", h12, m, sec, period))
    } else {
        None
    }
}

fn ordinal(n: u8) -> String {
    let s = match n % 10 {
        1 if n % 100 != 11 => "st",
        2 if n % 100 != 12 => "nd",
        3 if n % 100 != 13 => "rd",
        _ => "th",
    };
    format!("{}{}", n, s)
}

fn month_name(m: u8) -> &'static str {
    match m {
        1 => "January",
        2 => "February",
        3 => "March",
        4 => "April",
        5 => "May",
        6 => "June",
        7 => "July",
        8 => "August",
        9 => "September",
        10 => "October",
        11 => "November",
        12 => "December",
        _ => "unknown",
    }
}

fn dow_name(d: u8) -> &'static str {
    match d {
        0 | 7 => "Sunday",
        1 => "Monday",
        2 => "Tuesday",
        3 => "Wednesday",
        4 => "Thursday",
        5 => "Friday",
        6 => "Saturday",
        _ => "unknown",
    }
}

fn human_readable(expr: &CronExpr) -> String {
    let sec_any = expr.seconds.as_ref().map_or(true, is_any);
    let min_any = is_any(&expr.minutes);
    let hr_any = is_any(&expr.hours);
    let dom_any = is_any(&expr.dom);
    let mon_any = is_any(&expr.month);
    let dow_any = is_any(&expr.dow);

    // Every second: * * * * * *  (only when a 6-field expression with a
    // wildcard seconds field — a 5-field expression has no seconds field).
    if matches!(expr.seconds, Some(CronField::Any))
        && min_any
        && hr_any
        && dom_any
        && mon_any
        && dow_any
    {
        return "Every second".into();
    }

    // Every N seconds: */N * * * * *
    if let Some(CronField::Step(base, n)) = &expr.seconds {
        if matches!(base.as_ref(), CronField::Any)
            && min_any
            && hr_any
            && dom_any
            && mon_any
            && dow_any
        {
            return if *n == 1 {
                "Every second".into()
            } else {
                format!("Every {} seconds", n)
            };
        }
    }

    // Every minute
    if min_any && hr_any && dom_any && mon_any && dow_any {
        return "Every minute".into();
    }

    // Every N minutes: */N * * * *
    if let CronField::Step(base, n) = &expr.minutes {
        if matches!(base.as_ref(), CronField::Any)
            && sec_any
            && hr_any
            && dom_any
            && mon_any
            && dow_any
        {
            return if *n == 1 {
                "Every minute".into()
            } else {
                format!("Every {} minutes", n)
            };
        }
    }

    // Every hour: 0 * * * *
    if is_zero(&expr.minutes) && hr_any && dom_any && mon_any && dow_any {
        return "Every hour".into();
    }

    // Every N hours: 0 */N * * *
    if is_zero(&expr.minutes) {
        if let CronField::Step(base, n) = &expr.hours {
            if matches!(base.as_ref(), CronField::Any) && dom_any && mon_any && dow_any {
                return if *n == 1 {
                    "Every hour".into()
                } else {
                    format!("Every {} hours", n)
                };
            }
        }
    }

    // Build time part
    let time_str = format_time(expr.seconds.as_ref(), &expr.hours, &expr.minutes);

    // Specific day of week
    if dom_any && mon_any {
        if let Some(t) = &time_str {
            if weekday_field(&expr.dow).is_some() {
                return format!("Every weekday at {}", t);
            }
            if let CronField::Value(d) = expr.dow {
                return format!("Every {} at {}", dow_name(d), t);
            }
        }
    }

    // Specific days of week (explicit list), any month/day: 0 9 * * 1,3,5
    if dom_any && mon_any {
        if let Some(t) = &time_str {
            if let Some(days) = list_days_phrase(&expr.dow) {
                return format!("At {} on the {} of every month", t, days);
            }
        }
    }

    // Specific day of month, any month
    if dow_any && mon_any {
        if let CronField::Value(d) = expr.dom {
            if let Some(t) = &time_str {
                return format!("At {} on the {} of every month", t, ordinal(d));
            }
        }
    }

    // Specific month and day: 0 0 1 1 *
    if dow_any {
        if let (CronField::Value(d), CronField::Value(m)) = (&expr.dom, &expr.month) {
            if let Some(t) = &time_str {
                return format!("At {} on {} {}", t, month_name(*m), ordinal(*d));
            }
        }
    }

    // Every day at time
    if dom_any && mon_any && dow_any {
        if let Some(t) = &time_str {
            return format!("Every day at {}", t);
        }
    }

    // Fallback: reconstruct the expression (include seconds when present)
    let sec_str = expr
        .seconds
        .as_ref()
        .map(|s| format!("{} ", field_str(s)))
        .unwrap_or_default();
    format!(
        "At {} past {}{}on {} of {} ({})",
        field_str(&expr.minutes),
        field_str(&expr.hours),
        sec_str,
        field_str(&expr.dom),
        field_str(&expr.month),
        field_str(&expr.dow),
    )
}

fn field_str(f: &CronField) -> String {
    match f {
        CronField::Any => "*".into(),
        CronField::Value(n) => n.to_string(),
        CronField::Range(a, b) => format!("{}-{}", a, b),
        CronField::Step(base, n) => format!("{}/{}", field_str(base), n),
        CronField::List(items) => items.iter().map(field_str).collect::<Vec<_>>().join(","),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_every_minute() {
        let e = CronExpr::parse("* * * * *").unwrap();
        assert_eq!(e.to_human_readable(), "Every minute");
    }

    #[test]
    fn parse_every_hour() {
        let e = CronExpr::parse("0 * * * *").unwrap();
        assert_eq!(e.to_human_readable(), "Every hour");
    }

    #[test]
    fn parse_every_day_9am() {
        let e = CronExpr::parse("0 9 * * *").unwrap();
        assert_eq!(e.to_human_readable(), "Every day at 9:00 AM");
    }

    #[test]
    fn parse_every_monday() {
        let e = CronExpr::parse("0 9 * * 1").unwrap();
        assert_eq!(e.to_human_readable(), "Every Monday at 9:00 AM");
    }

    #[test]
    fn parse_1st_of_month() {
        let e = CronExpr::parse("0 9 1 * *").unwrap();
        assert_eq!(
            e.to_human_readable(),
            "At 9:00 AM on the 1st of every month"
        );
    }

    #[test]
    fn parse_every_5_minutes() {
        let e = CronExpr::parse("*/5 * * * *").unwrap();
        assert_eq!(e.to_human_readable(), "Every 5 minutes");
    }

    #[test]
    fn parse_jan_1_midnight() {
        let e = CronExpr::parse("0 0 1 1 *").unwrap();
        assert_eq!(e.to_human_readable(), "At 12:00 AM on January 1st");
    }

    #[test]
    fn parse_6_field() {
        let e = CronExpr::parse("30 0 9 * * *").unwrap();
        assert!(e.is_6_field());
        assert_eq!(e.seconds, Some(CronField::Value(30)));
    }

    #[test]
    fn human_readable_includes_seconds_6field() {
        assert_eq!(
            CronExpr::parse("*/10 * * * * *")
                .unwrap()
                .to_human_readable(),
            "Every 10 seconds"
        );
        assert_eq!(
            CronExpr::parse("* * * * * *").unwrap().to_human_readable(),
            "Every second"
        );
        assert_eq!(
            CronExpr::parse("30 0 9 * * *").unwrap().to_human_readable(),
            "Every day at 9:00:30 AM"
        );
        assert_eq!(
            CronExpr::parse("0 0 9 * * *").unwrap().to_human_readable(),
            "Every day at 9:00 AM"
        );
    }

    #[test]
    fn human_readable_common_expressions_are_grammatical() {
        // These are the README/example expressions; previously they collapsed to
        // gibberish like "At 0 past 9 on * of * (1-5)".
        assert_eq!(
            CronExpr::parse("0 9 * * 1-5").unwrap().to_human_readable(),
            "Every weekday at 9:00 AM"
        );
        assert_eq!(
            CronExpr::parse("0 9 * * 1,3,5")
                .unwrap()
                .to_human_readable(),
            "At 9:00 AM on the 1st, 3rd, 5th of every month"
        );
        assert_eq!(
            CronExpr::parse("0 */2 * * *").unwrap().to_human_readable(),
            "Every 2 hours"
        );
        assert_eq!(
            CronExpr::parse("*/1 * * * *").unwrap().to_human_readable(),
            "Every minute"
        );
    }

    #[test]
    fn parse_range() {
        let e = CronExpr::parse("0 9-17 * * *").unwrap();
        assert_eq!(e.hours, CronField::Range(9, 17));
    }

    #[test]
    fn parse_list() {
        let e = CronExpr::parse("0 9 * * 1,3,5").unwrap();
        assert_eq!(
            e.dow,
            CronField::List(vec![
                CronField::Value(1),
                CronField::Value(3),
                CronField::Value(5)
            ])
        );
    }

    #[test]
    fn wrong_field_count_error() {
        let err = CronExpr::parse("* * *").unwrap_err();
        assert_eq!(err.expected, "5 or 6 whitespace-separated fields");
    }

    #[test]
    fn invalid_char_error() {
        let err = CronExpr::parse("x * * * *").unwrap_err();
        assert_eq!(err.field, CronFieldName::Minutes);
        assert_eq!(err.found, Some('x'));
    }

    #[test]
    fn pm_time() {
        let e = CronExpr::parse("0 14 * * *").unwrap();
        assert_eq!(e.to_human_readable(), "Every day at 2:00 PM");
    }

    #[test]
    fn noon() {
        let e = CronExpr::parse("0 12 * * *").unwrap();
        assert_eq!(e.to_human_readable(), "Every day at 12:00 PM");
    }

    #[cfg(feature = "chrono")]
    #[test]
    fn next_after_weekday_morning() {
        use chrono::{Datelike, TimeZone, Timelike, Utc, Weekday};
        let expr = CronExpr::parse("0 9 * * 1-5").unwrap();
        let after = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap(); // Monday
        let next = expr.next_after(after).unwrap();
        assert!(next > after);
        assert_eq!(next.hour(), 9);
        assert_eq!(next.minute(), 0);
        assert_eq!(next.weekday(), Weekday::Mon);
    }

    #[cfg(feature = "chrono")]
    #[test]
    fn next_after_steps_and_ranges() {
        use chrono::{TimeZone, Timelike, Utc};
        let expr = CronExpr::parse("*/15 * * * *").unwrap();
        let after = Utc.with_ymd_and_hms(2024, 6, 1, 10, 0, 0).unwrap();
        let next = expr.next_after(after).unwrap();
        assert_eq!(next.minute() % 15, 0);
        assert!(next > after);
    }

    #[cfg(feature = "chrono")]
    #[test]
    fn next_after_6field_seconds() {
        use chrono::{Datelike, TimeZone, Timelike, Utc};
        let expr = CronExpr::parse("30 0 9 * * *").unwrap();
        let after = Utc.with_ymd_and_hms(2024, 6, 1, 9, 0, 0).unwrap();
        let next = expr.next_after(after).unwrap();
        assert_eq!(next.day(), 1);
        assert_eq!(next.hour(), 9);
        assert_eq!(next.minute(), 0);
        assert_eq!(next.second(), 30);
    }

    #[cfg(feature = "chrono")]
    #[test]
    fn next_after_6field_every_second() {
        use chrono::{TimeZone, Utc};
        let expr = CronExpr::parse("*/1 * * * * *").unwrap();
        let after = Utc.with_ymd_and_hms(2024, 6, 1, 9, 0, 0).unwrap();
        let next = expr.next_after(after).unwrap();
        assert_eq!(next, Utc.with_ymd_and_hms(2024, 6, 1, 9, 0, 1).unwrap());
    }

    #[cfg(feature = "chrono")]
    #[test]
    fn next_after_6field_same_minute_later_second() {
        use chrono::{TimeZone, Utc};
        let expr = CronExpr::parse("0,30 * * * * *").unwrap();
        let after = Utc.with_ymd_and_hms(2024, 6, 1, 9, 0, 10).unwrap();
        let next = expr.next_after(after).unwrap();
        assert_eq!(next, Utc.with_ymd_and_hms(2024, 6, 1, 9, 0, 30).unwrap());
        let next = expr.next_after(next).unwrap();
        assert_eq!(next, Utc.with_ymd_and_hms(2024, 6, 1, 9, 1, 0).unwrap());
    }

    #[cfg(feature = "chrono")]
    #[test]
    fn next_after_5field_skips_current_minute() {
        use chrono::{TimeZone, Utc};
        let expr = CronExpr::parse("* * * * *").unwrap();
        let after = Utc.with_ymd_and_hms(2024, 6, 1, 9, 0, 0).unwrap();
        let next = expr.next_after(after).unwrap();
        assert_eq!(next, Utc.with_ymd_and_hms(2024, 6, 1, 9, 1, 0).unwrap());
    }

    #[cfg(feature = "chrono")]
    #[test]
    fn next_after_none_within_bound() {
        use chrono::{TimeZone, Utc};
        // February 30th can never occur, so a schedule pinned to it never fires.
        let expr = CronExpr::parse("0 0 30 2 *").unwrap();
        let after = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();
        assert!(expr.next_after(after).is_none());
    }

    #[cfg(feature = "chrono")]
    #[test]
    fn next_after_tz_uses_local_wall_clock() {
        use chrono::{FixedOffset, TimeZone, Utc};
        let expr = CronExpr::parse("0 9 * * *").unwrap();
        // Same UTC instant, but interpreted in +08:00: the next 09:00 local
        // time is 01:00 UTC the same day, whereas in UTC it is 09:00 UTC.
        let tz = FixedOffset::east_opt(8 * 3600).unwrap();
        let after_utc = Utc.with_ymd_and_hms(2024, 6, 1, 0, 30, 0).unwrap();
        let after_tz = tz.from_utc_datetime(&after_utc.naive_utc());

        let next_utc = expr.next_after(after_utc).unwrap();
        let next_tz = expr.next_after_tz(after_tz).unwrap();
        assert_eq!(next_utc, Utc.with_ymd_and_hms(2024, 6, 1, 9, 0, 0).unwrap());
        assert_eq!(next_tz, tz.with_ymd_and_hms(2024, 6, 1, 9, 0, 0).unwrap());
    }

    #[cfg(feature = "chrono")]
    #[test]
    fn next_after_tz_matches_next_after_for_utc() {
        use chrono::{TimeZone, Utc};
        let expr = CronExpr::parse("0 9 * * 1-5").unwrap();
        let after = Utc.with_ymd_and_hms(2024, 6, 1, 0, 0, 0).unwrap();
        assert_eq!(expr.next_after(after), expr.next_after_tz(after));
    }

    #[cfg(feature = "chrono")]
    #[test]
    fn upcoming_yields_multiple_runs() {
        use chrono::{Datelike, TimeZone, Timelike, Utc};
        let expr = CronExpr::parse("0 9 * * *").unwrap();
        let after = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();
        let times: Vec<_> = expr.upcoming(after).take(3).collect();
        assert_eq!(times.len(), 3);
        assert_eq!(times[0].day(), 1);
        assert_eq!(times[1].day(), 2);
        assert_eq!(times[2].day(), 3);
        assert!(times.iter().all(|t| t.hour() == 9 && t.minute() == 0));
    }

    #[test]
    fn descending_range_rejected() {
        let err = CronExpr::parse("0 9-5 * * *").unwrap_err();
        assert_eq!(err.expected, "ascending range (start <= end)");
    }

    #[test]
    fn zero_step_rejected() {
        let err = CronExpr::parse("*/0 * * * *").unwrap_err();
        assert_eq!(err.expected, "non-zero step value");
    }

    #[test]
    fn valid_ascending_range_ok() {
        let e = CronExpr::parse("0 5-9 * * *").unwrap();
        assert_eq!(e.hours, CronField::Range(5, 9));
    }

    #[test]
    fn out_of_range_minutes_rejected() {
        let err = CronExpr::parse("60 * * * *").unwrap_err();
        assert_eq!(err.field, CronFieldName::Minutes);
        assert_eq!(err.expected, "value 0-59");
        assert_eq!(err.position, 0);
    }

    #[test]
    fn out_of_range_hours_rejected() {
        let err = CronExpr::parse("0 99 * * *").unwrap_err();
        assert_eq!(err.field, CronFieldName::Hours);
        assert_eq!(err.expected, "value 0-23");
        assert_eq!(err.position, 2);
        assert_eq!(err.found, Some('9'));
    }

    #[test]
    fn out_of_range_day_of_month_rejected() {
        assert_eq!(
            CronExpr::parse("0 0 0 * *").unwrap_err().field,
            CronFieldName::DayOfMonth
        );
        assert_eq!(
            CronExpr::parse("0 0 32 * *").unwrap_err().field,
            CronFieldName::DayOfMonth
        );
    }

    #[test]
    fn out_of_range_month_rejected() {
        assert_eq!(
            CronExpr::parse("0 0 1 0 *").unwrap_err().field,
            CronFieldName::Month
        );
        assert_eq!(
            CronExpr::parse("0 0 1 13 *").unwrap_err().field,
            CronFieldName::Month
        );
    }

    #[test]
    fn out_of_range_day_of_week_rejected() {
        let err = CronExpr::parse("0 0 * * 8").unwrap_err();
        assert_eq!(err.field, CronFieldName::DayOfWeek);
        assert_eq!(err.expected, "value 0-7 or name SUN-SAT");
    }

    #[test]
    fn out_of_range_seconds_rejected() {
        let err = CronExpr::parse("60 0 0 * * *").unwrap_err();
        assert_eq!(err.field, CronFieldName::Seconds);
        assert_eq!(err.expected, "value 0-59");
    }

    #[test]
    fn out_of_range_inside_list_and_range_rejected() {
        assert!(CronExpr::parse("0 1,24 * * *").is_err());
        assert!(CronExpr::parse("0 20-24 * * *").is_err());
        assert!(CronExpr::parse("0 300 * * *").is_err());
    }

    #[test]
    fn day_of_week_seven_normalises_to_zero() {
        // Day-of-week `7` is a cron alias for Sunday (`0`); it is accepted on
        // input but normalised to `0` so the stored value is always in 0-6.
        let e = CronExpr::parse("0 0 * * 7").unwrap();
        assert_eq!(e.dow, CronField::Value(0));
        assert_eq!(e.to_human_readable(), "Every Sunday at 12:00 AM");
    }

    #[test]
    fn dow_wildcard_has_seven_distinct_days() {
        // `*` over day-of-week must yield exactly 7 values (Sun-Sat), not 8.
        let e = CronExpr::parse("* * * * *").unwrap();
        assert_eq!(e.dow.expand(0, 6), vec![0, 1, 2, 3, 4, 5, 6]);
    }

    #[test]
    fn vixie_or_rule_and_with_step_on_star() {
        // `*/2` on day-of-month is treated as unrestricted, so the schedule is
        // AND-ed with day-of-week: it fires only on Mondays, not every even day.
        // After Mon 2024-01-01 the next matching minute is Mon 2024-01-08
        // (an even day would fire first under the buggy OR behaviour).
        use chrono::{Datelike, TimeZone, Utc, Weekday};
        let expr = CronExpr::parse("0 0 */2 * MON").unwrap();
        let after = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap(); // Monday
        let next = expr.next_after(after).unwrap();
        assert_eq!(next.weekday(), Weekday::Mon);
        assert_eq!(next, Utc.with_ymd_and_hms(2024, 1, 8, 0, 0, 0).unwrap());
    }

    #[test]
    fn vixie_or_rule_union_when_both_restricted() {
        // When both dom and dow are explicitly restricted they are OR-ed.
        use chrono::{Datelike, TimeZone, Utc, Weekday};
        let expr = CronExpr::parse("0 0 1 * MON").unwrap();
        let after = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap(); // Monday, 1st
        let next = expr.next_after(after).unwrap();
        // The 1st is a Monday here, so it fires immediately after `after`.
        assert!(next.day() == 1 || next.weekday() == Weekday::Mon);
    }

    #[test]
    fn leap_year_gap_spans_eight_years() {
        // `0 0 29 2 *` must be found across the 2096→2104 leap-year gap.
        use chrono::{Datelike, TimeZone, Utc};
        let expr = CronExpr::parse("0 0 29 2 *").unwrap();
        let after = Utc.with_ymd_and_hms(2096, 3, 1, 0, 0, 0).unwrap();
        let next = expr.next_after(after).unwrap();
        assert_eq!((next.year(), next.month(), next.day()), (2104, 2, 29));
    }

    #[test]
    fn trailing_garbage_rejected() {
        let err = CronExpr::parse("* * * * 1garbage").unwrap_err();
        assert_eq!(err.field, CronFieldName::DayOfWeek);
        assert_eq!(err.expected, "',' or end of field");
        assert_eq!(err.position, 9);
        assert_eq!(err.found, Some('g'));
        assert!(CronExpr::parse("* * * * *DROP TABLE").is_err());
        assert!(CronExpr::parse("* * * * *;").is_err());
        assert!(CronExpr::parse("*/5 * * * *x").is_err());
    }

    #[test]
    fn tab_separated_fields_parse() {
        let tabbed = CronExpr::parse("0\t9\t*\t*\t1").unwrap();
        assert_eq!(tabbed, CronExpr::parse("0 9 * * 1").unwrap());
        assert_eq!(tabbed.to_human_readable(), "Every Monday at 9:00 AM");
        let mixed = CronExpr::parse(" 30\t0 \t9 * * *\t").unwrap();
        assert!(mixed.is_6_field());
        assert_eq!(mixed.seconds, Some(CronField::Value(30)));
    }

    #[test]
    fn step_over_range_stops_at_range_end() {
        let e = CronExpr::parse("0 1-5/2 * * *").unwrap();
        assert_eq!(e.hours.expand(0, 23), vec![1, 3, 5]);
    }

    #[test]
    fn step_over_value_runs_to_field_max() {
        let e = CronExpr::parse("5/15 * * * *").unwrap();
        assert_eq!(e.minutes.expand(0, 59), vec![5, 20, 35, 50]);
    }

    #[test]
    fn step_over_wildcard_covers_whole_field() {
        let e = CronExpr::parse("0 */6 * * *").unwrap();
        assert_eq!(e.hours.expand(0, 23), vec![0, 6, 12, 18]);
    }

    #[test]
    fn shorthand_aliases_expand() {
        assert_eq!(
            CronExpr::parse("@hourly").unwrap(),
            CronExpr::parse("0 * * * *").unwrap()
        );
        assert_eq!(
            CronExpr::parse("@daily").unwrap(),
            CronExpr::parse("0 0 * * *").unwrap()
        );
        assert_eq!(
            CronExpr::parse("@midnight").unwrap(),
            CronExpr::parse("0 0 * * *").unwrap()
        );
        assert_eq!(
            CronExpr::parse("@weekly").unwrap(),
            CronExpr::parse("0 0 * * 0").unwrap()
        );
        assert_eq!(
            CronExpr::parse("@monthly").unwrap(),
            CronExpr::parse("0 0 1 * *").unwrap()
        );
        assert_eq!(
            CronExpr::parse("@yearly").unwrap(),
            CronExpr::parse("0 0 1 1 *").unwrap()
        );
        assert_eq!(
            CronExpr::parse("@annually").unwrap(),
            CronExpr::parse("0 0 1 1 *").unwrap()
        );
    }

    #[test]
    fn shorthand_is_case_insensitive_and_trimmed() {
        let e = CronExpr::parse("  @DaIlY \t").unwrap();
        assert_eq!(e.to_human_readable(), "Every day at 12:00 AM");
    }

    #[test]
    fn reboot_shorthand_rejected() {
        let err = CronExpr::parse("@reboot").unwrap_err();
        assert_eq!(
            err.expected,
            "a periodic schedule (@reboot has no next run time and is not supported)"
        );
        assert_eq!(err.found, Some('@'));
        assert_eq!(err.position, 0);
    }

    #[test]
    fn unknown_shorthand_rejected() {
        let err = CronExpr::parse("@never").unwrap_err();
        assert_eq!(
            err.expected,
            "one of @yearly, @annually, @monthly, @weekly, @daily, @midnight, @hourly"
        );
        let err = CronExpr::parse("@daily 0 0 * * *").unwrap_err();
        assert_eq!(err.expected, "end of input after a shorthand alias");
    }

    #[test]
    fn named_months_parse() {
        let e = CronExpr::parse("0 0 1 JAN *").unwrap();
        assert_eq!(e.month, CronField::Value(1));
        assert_eq!(e.to_human_readable(), "At 12:00 AM on January 1st");
        let e = CronExpr::parse("0 0 1 dec *").unwrap();
        assert_eq!(e.month, CronField::Value(12));
        let e = CronExpr::parse("0 0 1 Jan-Mar *").unwrap();
        assert_eq!(e.month, CronField::Range(1, 3));
        let e = CronExpr::parse("0 0 1 JAN,JUL *").unwrap();
        assert_eq!(
            e.month,
            CronField::List(vec![CronField::Value(1), CronField::Value(7)])
        );
    }

    #[test]
    fn named_days_of_week_parse() {
        let e = CronExpr::parse("0 9 * * MON").unwrap();
        assert_eq!(e.dow, CronField::Value(1));
        assert_eq!(e.to_human_readable(), "Every Monday at 9:00 AM");
        let e = CronExpr::parse("0 9 * * sun").unwrap();
        assert_eq!(e.dow, CronField::Value(0));
        let e = CronExpr::parse("0 9 * * Mon-Fri").unwrap();
        assert_eq!(e.dow, CronField::Range(1, 5));
        let e = CronExpr::parse("0 9 * * SAT,SUN").unwrap();
        assert_eq!(
            e.dow,
            CronField::List(vec![CronField::Value(6), CronField::Value(0)])
        );
        let e = CronExpr::parse("0 9 * * MON-FRI/2").unwrap();
        assert_eq!(e.dow.expand(0, 7), vec![1, 3, 5]);
    }

    #[test]
    fn unknown_names_rejected() {
        let err = CronExpr::parse("0 0 1 FOO *").unwrap_err();
        assert_eq!(err.field, CronFieldName::Month);
        assert_eq!(err.expected, "value 1-12 or name JAN-DEC");
        assert_eq!(err.found, Some('F'));
        let err = CronExpr::parse("0 9 * * FUN").unwrap_err();
        assert_eq!(err.field, CronFieldName::DayOfWeek);
        assert_eq!(err.expected, "value 0-7 or name SUN-SAT");
    }

    #[test]
    fn names_not_allowed_in_numeric_fields() {
        let err = CronExpr::parse("MON 9 * * *").unwrap_err();
        assert_eq!(err.field, CronFieldName::Minutes);
        assert_eq!(err.found, Some('M'));
        assert!(CronExpr::parse("0 JAN * * *").is_err());
    }
}
