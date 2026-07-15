#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

use std::fmt;

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
#[derive(Debug, Clone, PartialEq, Eq)]
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
    /// # Example
    ///
    /// ```
    /// use tpt_cron_parse::CronExpr;
    ///
    /// let expr = CronExpr::parse("*/5 * * * *").unwrap();
    /// assert_eq!(expr.to_human_readable(), "Every 5 minutes");
    /// ```
    pub fn parse(s: &str) -> Result<CronExpr, CronError> {
        let parser = CronParser::new(s);
        parser.parse()
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
}

// ---- Parser internals ----

struct CronParser<'a> {
    input: &'a str,
    pos: usize,
}

impl<'a> CronParser<'a> {
    fn new(input: &'a str) -> Self {
        Self { input, pos: 0 }
    }

    fn peek(&self) -> Option<char> {
        self.input[self.pos..].chars().next()
    }

    fn skip_whitespace(&mut self) {
        while self.pos < self.input.len() && self.input.as_bytes()[self.pos] == b' ' {
            self.pos += 1;
        }
    }

    fn parse_u8(&mut self, field: CronFieldName) -> Result<u8, CronError> {
        let start = self.pos;
        while self.pos < self.input.len() && self.input.as_bytes()[self.pos].is_ascii_digit() {
            self.pos += 1;
        }
        if self.pos == start {
            return Err(CronError {
                position: self.pos,
                field,
                expected: "digit",
                found: self.peek(),
            });
        }
        self.input[start..self.pos]
            .parse::<u8>()
            .map_err(|_| CronError {
                position: start,
                field,
                expected: "number 0-255",
                found: None,
            })
    }

    fn parse_field(&mut self, field: CronFieldName) -> Result<CronField, CronError> {
        let mut items: Vec<CronField> = Vec::new();
        loop {
            let item = self.parse_item(field)?;
            items.push(item);
            if self.pos < self.input.len() && self.input.as_bytes()[self.pos] == b',' {
                self.pos += 1;
            } else {
                break;
            }
        }
        if items.len() == 1 {
            Ok(items.remove(0))
        } else {
            Ok(CronField::List(items))
        }
    }

    fn parse_item(&mut self, field: CronFieldName) -> Result<CronField, CronError> {
        let base = if self.pos < self.input.len() && self.input.as_bytes()[self.pos] == b'*' {
            self.pos += 1;
            CronField::Any
        } else {
            let n_start = self.pos;
            let n = self.parse_u8(field)?;
            if self.pos < self.input.len() && self.input.as_bytes()[self.pos] == b'-' {
                self.pos += 1;
                let end = self.parse_u8(field)?;
                if n > end {
                    return Err(CronError {
                        position: n_start,
                        field,
                        expected: "ascending range (start <= end)",
                        found: Some('-'),
                    });
                }
                CronField::Range(n, end)
            } else {
                CronField::Value(n)
            }
        };

        if self.pos < self.input.len() && self.input.as_bytes()[self.pos] == b'/' {
            self.pos += 1;
            let step_start = self.pos;
            let step = self.parse_u8(field)?;
            if step == 0 {
                return Err(CronError {
                    position: step_start,
                    field,
                    expected: "non-zero step value",
                    found: Some('0'),
                });
            }
            Ok(CronField::Step(Box::new(base), step))
        } else {
            Ok(base)
        }
    }

    fn parse(mut self) -> Result<CronExpr, CronError> {
        self.skip_whitespace();

        // Count fields to detect 5 vs 6
        let fields: Vec<&str> = self.input.split_whitespace().collect();
        if fields.len() != 5 && fields.len() != 6 {
            return Err(CronError {
                position: 0,
                field: CronFieldName::Minutes,
                expected: "5 or 6 whitespace-separated fields",
                found: None,
            });
        }

        let is_6 = fields.len() == 6;

        let seconds = if is_6 {
            let f = self.parse_field(CronFieldName::Seconds)?;
            self.skip_whitespace();
            Some(f)
        } else {
            None
        };

        let minutes = self.parse_field(CronFieldName::Minutes)?;
        self.skip_whitespace();
        let hours = self.parse_field(CronFieldName::Hours)?;
        self.skip_whitespace();
        let dom = self.parse_field(CronFieldName::DayOfMonth)?;
        self.skip_whitespace();
        let month = self.parse_field(CronFieldName::Month)?;
        self.skip_whitespace();
        let dow = self.parse_field(CronFieldName::DayOfWeek)?;

        Ok(CronExpr {
            seconds,
            minutes,
            hours,
            dom,
            month,
            dow,
        })
    }
}

// ---- Human-readable conversion ----

fn is_any(f: &CronField) -> bool {
    matches!(f, CronField::Any)
}

fn is_zero(f: &CronField) -> bool {
    matches!(f, CronField::Value(0))
}

fn format_time(hours: &CronField, minutes: &CronField) -> Option<String> {
    if let (CronField::Value(h), CronField::Value(m)) = (hours, minutes) {
        let period = if *h < 12 { "AM" } else { "PM" };
        let h12 = match h {
            0 => 12,
            h if *h <= 12 => *h as u32,
            h => (*h - 12) as u32,
        };
        Some(format!("{}:{:02} {}", h12, m, period))
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
    let min_any = is_any(&expr.minutes);
    let hr_any = is_any(&expr.hours);
    let dom_any = is_any(&expr.dom);
    let mon_any = is_any(&expr.month);
    let dow_any = is_any(&expr.dow);

    // Every minute
    if min_any && hr_any && dom_any && mon_any && dow_any {
        return "Every minute".into();
    }

    // Every N minutes: */N * * * *
    if let CronField::Step(base, n) = &expr.minutes {
        if matches!(base.as_ref(), CronField::Any) && hr_any && dom_any && mon_any && dow_any {
            return format!("Every {} minutes", n);
        }
    }

    // Every hour: 0 * * * *
    if is_zero(&expr.minutes) && hr_any && dom_any && mon_any && dow_any {
        return "Every hour".into();
    }

    // Build time part
    let time_str = format_time(&expr.hours, &expr.minutes);

    // Specific day of week
    if dom_any && mon_any {
        if let CronField::Value(d) = expr.dow {
            if let Some(t) = &time_str {
                return format!("Every {} at {}", dow_name(d), t);
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

    // Fallback: reconstruct the expression
    format!(
        "At {} past {} on {} of {} ({})",
        field_str(&expr.minutes),
        field_str(&expr.hours),
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
}
