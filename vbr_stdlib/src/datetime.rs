use chrono::{DateTime as ChronoDateTime, Duration, Local,
             NaiveDateTime, Utc};

pub struct DateTime;

impl DateTime {

    /// Get current local date and time
    /// VBA equivalent: Now()
    pub fn now() -> ChronoDateTime<Local> {
        Local::now()
    }

    /// Get current UTC date and time
    /// VBA equivalent: Now() but UTC
    pub fn utc() -> ChronoDateTime<Utc> {
        Utc::now()
    }

    /// Format a datetime as a string
    /// VBA equivalent: Format(date, "pattern")
    pub fn format(dt: &ChronoDateTime<Local>, pattern: &str) -> String {
        dt.format(pattern).to_string()
    }

    /// Parse a datetime from a string
    /// VBA equivalent: CDate()
    pub fn parse(text: &str, pattern: &str)
        -> Result<NaiveDateTime, String> {
        NaiveDateTime::parse_from_str(text, pattern)
            .map_err(|e| e.to_string())
    }

    /// Add days to a datetime
    /// VBA equivalent: DateAdd("d", n, date)
    pub fn add_days(
        dt: ChronoDateTime<Local>,
        days: i64
    ) -> ChronoDateTime<Local> {
        dt + Duration::days(days)
    }

    /// Add hours to a datetime
    /// VBA equivalent: DateAdd("h", n, date)
    pub fn add_hours(
        dt: ChronoDateTime<Local>,
        hours: i64
    ) -> ChronoDateTime<Local> {
        dt + Duration::hours(hours)
    }

    /// Add minutes to a datetime
    /// VBA equivalent: DateAdd("n", n, date)
    pub fn add_minutes(
        dt: ChronoDateTime<Local>,
        minutes: i64
    ) -> ChronoDateTime<Local> {
        dt + Duration::minutes(minutes)
    }

    /// Difference in days between two datetimes
    /// VBA equivalent: DateDiff("d", date1, date2)
    pub fn diff_days(
        dt1: ChronoDateTime<Local>,
        dt2: ChronoDateTime<Local>
    ) -> i64 {
        (dt2 - dt1).num_days()
    }

    /// Difference in hours between two datetimes
    /// VBA equivalent: DateDiff("h", date1, date2)
    pub fn diff_hours(
        dt1: ChronoDateTime<Local>,
        dt2: ChronoDateTime<Local>
    ) -> i64 {
        (dt2 - dt1).num_hours()
    }

    /// Get year from datetime
    /// VBA equivalent: Year(date)
    pub fn year(dt: &ChronoDateTime<Local>) -> i32 {
        dt.format("%Y").to_string().parse().unwrap()
    }

    /// Get month from datetime
    /// VBA equivalent: Month(date)
    pub fn month(dt: &ChronoDateTime<Local>) -> u32 {
        dt.format("%m").to_string().parse().unwrap()
    }

    /// Get day from datetime
    /// VBA equivalent: Day(date)
    pub fn day(dt: &ChronoDateTime<Local>) -> u32 {
        dt.format("%d").to_string().parse().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_now_and_format() {
        let now = DateTime::now();
        let formatted = DateTime::format(&now, "%Y-%m-%d");
        assert_eq!(formatted.len(), 10);
    }

    #[test]
    fn test_add_days() {
        let now = DateTime::now();
        let tomorrow = DateTime::add_days(now, 1);
        assert_eq!(DateTime::diff_days(now, tomorrow), 1);
    }

    #[test]
    fn test_parse() {
        let dt = DateTime::parse(
            "2024-01-15 10:30:00",
            "%Y-%m-%d %H:%M:%S"
        ).unwrap();
        assert_eq!(dt.to_string(), "2024-01-15 10:30:00");
    }
}
