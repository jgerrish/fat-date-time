//! Functions for parsing DOS FAT filesystem date and time values.
//! This crate provides functions for parsing DOS FAT dates and times.
//!
//! It provides two main functions, parse_fat_date and parse_fat_time.
//! These functions return Date and Time structures from the time crate.
#![warn(missing_docs)]
#![warn(unsafe_code)]

use time::{Date, Month, Time};

/// Parse a FAT DOS time.
/// Assume a value of zero is an invalid date / reserved field
/// Return None if the time is invalid
///
/// From FAT: General Overview of On-Disk Format \
/// MS-DOS epoch is 01/01/1980 \
/// Bits 0-4: 2-second count, valid value range 0-29 inclusive (0 - 58 seconds). \
/// Bits 5-10: Minutes, valid value range 0-59 inclusive. \
/// Bits 11-15: Hours, valid value range 0-23 inclusive. \
///
/// # Examples
///
/// ```
/// use fat_date_time::parse_fat_time;
///
/// let time = parse_fat_time(0xbf7d);
///
/// assert!(time.is_some());
/// assert_eq!(time.unwrap().hour(), 23);
/// assert_eq!(time.unwrap().minute(), 59);
/// assert_eq!(time.unwrap().second(), 58);
/// ```
pub fn parse_fat_time(dos_time: u16) -> Option<Time> {
    // Assume a value of zero is an "invalid" time and the field is a
    // "reserved" field
    // This isn't always true, some utilities may not write a time
    let hours = ((dos_time >> 11) as u8) & 0x1F;
    if hours > 23 {
        return None;
    }
    let minutes = ((dos_time >> 5) as u8) & 0x3F;
    if minutes > 59 {
        return None;
    }
    let seconds = (dos_time & 0x1F) as u8;
    if seconds > 29 {
        return None;
    }

    let time = Time::from_hms(hours, minutes, seconds * 2);

    match time {
        Ok(t) => Some(t),
        Err(e) => panic!("Couldn't parse time: {}", e),
    }
}

/// Parse a FAT DOS date.
/// If a date is invalid, a value of None is returned.
///
/// From FAT: General Overview of On-Disk Format \
/// The valid time range is from Midnight 00:00:00 to 23:59:58. \
/// Bits 0-4: Day of month, valid value range 1-31 inclusive. \
/// Bits 5-8: Month of year, 1 = January, valid value range 1-12 inclusive. \
/// Bits 9-15: Count of years from 1980, valid value range 0-127 inclusive (1980-2107). \
///
/// # Examples
///
/// ```
/// use fat_date_time::parse_fat_date;
/// use time::Month;
///
/// let date = parse_fat_date(0xff9f);
///
/// assert!(date.is_some());
/// assert_eq!(date.unwrap().year(), 2107);
/// assert_eq!(date.unwrap().month(), Month::December);
/// assert_eq!(date.unwrap().day(), 31);
///
/// ```
pub fn parse_fat_date(dos_date: u16) -> Option<Date> {
    // Assume a value of zero is an "invalid" date and the field is a
    // "reserved" field
    // This isn't always true, some utilities may not write a date
    if dos_date == 0 {
        return None;
    }

    let year: i32 = ((dos_date >> 9) & 0x7F) as i32;
    // equivalent to (year < 0) || (year > 127)
    if !(0..=127).contains(&year) {
        return None;
    }

    let year = year + 1980;

    let month = (dos_date >> 5) & 0x0f;

    let month = match month {
        1 => Month::January,
        2 => Month::February,
        3 => Month::March,
        4 => Month::April,
        5 => Month::May,
        6 => Month::June,
        7 => Month::July,
        8 => Month::August,
        9 => Month::September,
        10 => Month::October,
        11 => Month::November,
        12 => Month::December,
        _ => return None,
    };

    let day = (dos_date & 0x1F) as u8;

    // Check that the day value is in range
    // equivalent to (day < 1) || (day > 31)
    if !(1..=31).contains(&day) {
        return None;
    }

    let date = Date::from_calendar_date(year, month, day);

    match date {
        Ok(d) => Some(d),
        Err(e) => panic!("Couldn't parse date: {}", e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_fat_date_works() {
        // Date value of zero
        let date = parse_fat_date(0);
        assert!(date.is_none());

        // The earliest possible "valid" date, given the specification in
        // FAT: General Overview of On-Disk Format
        let date = parse_fat_date(0b0000000000100001);

        assert!(date.is_some());
        assert_eq!(date.unwrap().year(), 1980);
        assert_eq!(date.unwrap().month(), Month::January);
        assert_eq!(date.unwrap().day(), 1);

        // The latest possible date
        let date = parse_fat_date(0b1111111110011111);

        assert!(date.is_some());
        assert_eq!(date.unwrap().year(), 2107);
        assert_eq!(date.unwrap().month(), Month::December);
        assert_eq!(date.unwrap().day(), 31);

        // The date with bit 1 set for year, month and day.
        let date = parse_fat_date(0b0000001000100001);

        assert!(date.is_some());
        assert_eq!(date.unwrap().year(), 1981);
        assert_eq!(date.unwrap().month(), Month::January);
        assert_eq!(date.unwrap().day(), 1);

        // Test date with day < 1
        let date = parse_fat_date(0b0000000000100000);
        assert!(date.is_none());

        // Test date with month < 1
        let date = parse_fat_date(0b0000000000000001);
        assert!(date.is_none());

        // Test date with month > 12
        let date = parse_fat_date(0b0000000110100001);
        assert!(date.is_none());
    }

    #[test]
    fn parse_fat_time_works() {
        // Test the earlier possible time
        let time = parse_fat_time(0);
        assert!(time.is_some());
        assert_eq!(time.unwrap().hour(), 0);
        assert_eq!(time.unwrap().minute(), 0);
        assert_eq!(time.unwrap().second(), 0);

        // Test the latest possible time
        let time = parse_fat_time(0b1011111101111101);
        assert!(time.is_some());
        assert_eq!(time.unwrap().hour(), 23);
        assert_eq!(time.unwrap().minute(), 59);
        assert_eq!(time.unwrap().second(), 58);

        // Test second value > 29
        let time = parse_fat_time(0b1011111101111110);
        assert!(time.is_none());

        // Test minute value > 59
        let time = parse_fat_time(0b1011111110011101);
        assert!(time.is_none());

        // Test hour value > 23
        let time = parse_fat_time(0b1100011101111101);
        assert!(time.is_none());
    }

    /// Tests from pyfatfs Python module
    #[test]
    fn external_tests_pass() {
        let date = parse_fat_date(0xFF9F);
        assert!(date.is_some());
        assert_eq!(date.unwrap().year(), 2107);
        assert_eq!(date.unwrap().month(), Month::December);
        assert_eq!(date.unwrap().day(), 31);

        // maximum time value
        let date = parse_fat_date(0xBF7D);
        assert!(date.is_some());
        assert_eq!(date.unwrap().year(), 2075);
        assert_eq!(date.unwrap().month(), Month::November);
        assert_eq!(date.unwrap().day(), 29);

        let time = parse_fat_time(0xBF7D);
        assert!(time.is_some());
        assert_eq!(time.unwrap().hour(), 23);
        assert_eq!(time.unwrap().minute(), 59);
        assert_eq!(time.unwrap().second(), 58);

        let date = parse_fat_date(0xFF9F);
        assert!(date.is_some());
        assert_eq!(date.unwrap().year(), 2107);
        assert_eq!(date.unwrap().month(), Month::December);
        assert_eq!(date.unwrap().day(), 31);

        let date = parse_fat_date(0x0021);
        assert!(date.is_some());
        assert_eq!(date.unwrap().year(), 1980);
        assert_eq!(date.unwrap().month(), Month::January);
        assert_eq!(date.unwrap().day(), 1);

        let time = parse_fat_time(0x477D);
        assert!(time.is_some());
        assert_eq!(time.unwrap().hour(), 8);
        assert_eq!(time.unwrap().minute(), 59);
        assert_eq!(time.unwrap().second(), 58);
    }
}
