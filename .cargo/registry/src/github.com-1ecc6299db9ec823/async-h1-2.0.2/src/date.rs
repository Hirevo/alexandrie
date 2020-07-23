use std::fmt::{self, Display, Formatter};
use std::str::{from_utf8, FromStr};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use http_types::{bail, ensure, format_err};

const IMF_FIXDATE_LENGTH: usize = 29;
const RFC850_MAX_LENGTH: usize = 23;
const ASCTIME_LENGTH: usize = 24;

const YEAR_9999_SECONDS: u64 = 253402300800;
const SECONDS_IN_DAY: u64 = 86400;
const SECONDS_IN_HOUR: u64 = 3600;

/// Format using the `Display` trait.
/// Convert timestamp into/from `SytemTime` to use.
/// Supports comparison and sorting.
#[derive(Copy, Clone, Debug, Eq, Ord)]
pub struct HttpDate {
    /// 0...59
    second: u8,
    /// 0...59
    minute: u8,
    /// 0...23
    hour: u8,
    /// 1...31
    day: u8,
    /// 1...12
    month: u8,
    /// 1970...9999
    year: u16,
    /// 1...7
    week_day: u8,
}

/// Parse a date from an HTTP header field.
///
/// Supports the preferred IMF-fixdate and the legacy RFC 805 and
/// ascdate formats. Two digit years are mapped to dates between
/// 1970 and 2069.
#[allow(dead_code)]
pub(crate) fn parse_http_date(s: &str) -> http_types::Result<SystemTime> {
    s.parse::<HttpDate>().map(|d| d.into())
}

/// Format a date to be used in a HTTP header field.
///
/// Dates are formatted as IMF-fixdate: `Fri, 15 May 2015 15:34:21 GMT`.
pub(crate) fn fmt_http_date(d: SystemTime) -> String {
    format!("{}", HttpDate::from(d))
}

impl HttpDate {
    fn is_valid(self) -> bool {
        self.second < 60
            && self.minute < 60
            && self.hour < 24
            && self.day > 0
            && self.day < 32
            && self.month > 0
            && self.month <= 12
            && self.year >= 1970
            && self.year <= 9999
            && self.week_day >= 1
            && self.week_day < 8
    }
}

fn parse_imf_fixdate(s: &[u8]) -> http_types::Result<HttpDate> {
    // Example: `Sun, 06 Nov 1994 08:49:37 GMT`
    if s.len() != IMF_FIXDATE_LENGTH
        || &s[25..] != b" GMT"
        || s[16] != b' '
        || s[19] != b':'
        || s[22] != b':'
    {
        bail!("Date time not in imf fixdate format");
    }
    Ok(HttpDate {
        second: from_utf8(&s[23..25])?.parse()?,
        minute: from_utf8(&s[20..22])?.parse()?,
        hour: from_utf8(&s[17..19])?.parse()?,
        day: from_utf8(&s[5..7])?.parse()?,
        month: match &s[7..12] {
            b" Jan " => 1,
            b" Feb " => 2,
            b" Mar " => 3,
            b" Apr " => 4,
            b" May " => 5,
            b" Jun " => 6,
            b" Jul " => 7,
            b" Aug " => 8,
            b" Sep " => 9,
            b" Oct " => 10,
            b" Nov " => 11,
            b" Dec " => 12,
            _ => bail!("Invalid Month"),
        },
        year: from_utf8(&s[12..16])?.parse()?,
        week_day: match &s[..5] {
            b"Mon, " => 1,
            b"Tue, " => 2,
            b"Wed, " => 3,
            b"Thu, " => 4,
            b"Fri, " => 5,
            b"Sat, " => 6,
            b"Sun, " => 7,
            _ => bail!("Invalid Day"),
        },
    })
}

fn parse_rfc850_date(s: &[u8]) -> http_types::Result<HttpDate> {
    // Example: `Sunday, 06-Nov-94 08:49:37 GMT`
    ensure!(
        s.len() >= RFC850_MAX_LENGTH,
        "Date time not in rfc850 format"
    );

    fn week_day<'a>(s: &'a [u8], week_day: u8, name: &'static [u8]) -> Option<(u8, &'a [u8])> {
        if &s[0..name.len()] == name {
            return Some((week_day, &s[name.len()..]));
        }
        None
    }
    let (week_day, s) = week_day(s, 1, b"Monday, ")
        .or_else(|| week_day(s, 2, b"Tuesday, "))
        .or_else(|| week_day(s, 3, b"Wednesday, "))
        .or_else(|| week_day(s, 4, b"Thursday, "))
        .or_else(|| week_day(s, 5, b"Friday, "))
        .or_else(|| week_day(s, 6, b"Saturday, "))
        .or_else(|| week_day(s, 7, b"Sunday, "))
        .ok_or_else(|| format_err!("Invalid day"))?;
    if s.len() != 22 || s[12] != b':' || s[15] != b':' || &s[18..22] != b" GMT" {
        bail!("Date time not in rfc950 fmt");
    }
    let mut year = from_utf8(&s[7..9])?.parse::<u16>()?;
    if year < 70 {
        year += 2000;
    } else {
        year += 1900;
    }
    Ok(HttpDate {
        second: from_utf8(&s[16..18])?.parse()?,
        minute: from_utf8(&s[13..15])?.parse()?,
        hour: from_utf8(&s[10..12])?.parse()?,
        day: from_utf8(&s[0..2])?.parse()?,
        month: match &s[2..7] {
            b"-Jan-" => 1,
            b"-Feb-" => 2,
            b"-Mar-" => 3,
            b"-Apr-" => 4,
            b"-May-" => 5,
            b"-Jun-" => 6,
            b"-Jul-" => 7,
            b"-Aug-" => 8,
            b"-Sep-" => 9,
            b"-Oct-" => 10,
            b"-Nov-" => 11,
            b"-Dec-" => 12,
            _ => bail!("Invalid month"),
        },
        year,
        week_day,
    })
}

fn parse_asctime(s: &[u8]) -> http_types::Result<HttpDate> {
    // Example: `Sun Nov  6 08:49:37 1994`
    if s.len() != ASCTIME_LENGTH || s[10] != b' ' || s[13] != b':' || s[16] != b':' || s[19] != b' '
    {
        bail!("Date time not in asctime format");
    }
    Ok(HttpDate {
        second: from_utf8(&s[17..19])?.parse()?,
        minute: from_utf8(&s[14..16])?.parse()?,
        hour: from_utf8(&s[11..13])?.parse()?,
        day: {
            let x = &s[8..10];
            from_utf8(if x[0] == b' ' { &x[1..2] } else { x })?.parse()?
        },
        month: match &s[4..8] {
            b"Jan " => 1,
            b"Feb " => 2,
            b"Mar " => 3,
            b"Apr " => 4,
            b"May " => 5,
            b"Jun " => 6,
            b"Jul " => 7,
            b"Aug " => 8,
            b"Sep " => 9,
            b"Oct " => 10,
            b"Nov " => 11,
            b"Dec " => 12,
            _ => bail!("Invalid month"),
        },
        year: from_utf8(&s[20..24])?.parse()?,
        week_day: match &s[0..4] {
            b"Mon " => 1,
            b"Tue " => 2,
            b"Wed " => 3,
            b"Thu " => 4,
            b"Fri " => 5,
            b"Sat " => 6,
            b"Sun " => 7,
            _ => bail!("Invalid day"),
        },
    })
}

impl From<SystemTime> for HttpDate {
    fn from(system_time: SystemTime) -> Self {
        let dur = system_time
            .duration_since(UNIX_EPOCH)
            .expect("all times should be after the epoch");
        let secs_since_epoch = dur.as_secs();

        if secs_since_epoch >= YEAR_9999_SECONDS {
            // year 9999
            panic!("date must be before year 9999");
        }

        /* 2000-03-01 (mod 400 year, immediately after feb29 */
        const LEAPOCH: i64 = 11017;
        const DAYS_PER_400Y: i64 = 365 * 400 + 97;
        const DAYS_PER_100Y: i64 = 365 * 100 + 24;
        const DAYS_PER_4Y: i64 = 365 * 4 + 1;

        let days = (secs_since_epoch / SECONDS_IN_DAY) as i64 - LEAPOCH;
        let secs_of_day = secs_since_epoch % SECONDS_IN_DAY;

        let mut qc_cycles = days / DAYS_PER_400Y;
        let mut remdays = days % DAYS_PER_400Y;

        if remdays < 0 {
            remdays += DAYS_PER_400Y;
            qc_cycles -= 1;
        }

        let mut c_cycles = remdays / DAYS_PER_100Y;
        if c_cycles == 4 {
            c_cycles -= 1;
        }
        remdays -= c_cycles * DAYS_PER_100Y;

        let mut q_cycles = remdays / DAYS_PER_4Y;
        if q_cycles == 25 {
            q_cycles -= 1;
        }
        remdays -= q_cycles * DAYS_PER_4Y;

        let mut remyears = remdays / 365;
        if remyears == 4 {
            remyears -= 1;
        }
        remdays -= remyears * 365;

        let mut year = 2000 + remyears + 4 * q_cycles + 100 * c_cycles + 400 * qc_cycles;

        let months = [31, 30, 31, 30, 31, 31, 30, 31, 30, 31, 31, 29];
        let mut month = 0;
        for month_len in months.iter() {
            month += 1;
            if remdays < *month_len {
                break;
            }
            remdays -= *month_len;
        }
        let mday = remdays + 1;
        let month = if month + 2 > 12 {
            year += 1;
            month - 10
        } else {
            month + 2
        };

        let mut week_day = (3 + days) % 7;
        if week_day <= 0 {
            week_day += 7
        };

        HttpDate {
            second: (secs_of_day % 60) as u8,
            minute: ((secs_of_day % SECONDS_IN_HOUR) / 60) as u8,
            hour: (secs_of_day / SECONDS_IN_HOUR) as u8,
            day: mday as u8,
            month: month as u8,
            year: year as u16,
            week_day: week_day as u8,
        }
    }
}

impl From<HttpDate> for SystemTime {
    fn from(http_date: HttpDate) -> Self {
        let leap_years = ((http_date.year - 1) - 1968) / 4 - ((http_date.year - 1) - 1900) / 100
            + ((http_date.year - 1) - 1600) / 400;
        let mut ydays = match http_date.month {
            1 => 0,
            2 => 31,
            3 => 59,
            4 => 90,
            5 => 120,
            6 => 151,
            7 => 181,
            8 => 212,
            9 => 243,
            10 => 273,
            11 => 304,
            12 => 334,
            _ => unreachable!(),
        } + http_date.day as u64
            - 1;
        if is_leap_year(http_date.year) && http_date.month > 2 {
            ydays += 1;
        }
        let days = (http_date.year as u64 - 1970) * 365 + leap_years as u64 + ydays;
        UNIX_EPOCH
            + Duration::from_secs(
                http_date.second as u64
                    + http_date.minute as u64 * 60
                    + http_date.hour as u64 * SECONDS_IN_HOUR
                    + days * SECONDS_IN_DAY,
            )
    }
}

impl FromStr for HttpDate {
    type Err = http_types::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        ensure!(s.is_ascii(), "String slice is not valid ASCII");
        let x = s.trim().as_bytes();
        let date = parse_imf_fixdate(x)
            .or_else(|_| parse_rfc850_date(x))
            .or_else(|_| parse_asctime(x))?;
        ensure!(date.is_valid(), "Invalid date time");
        Ok(date)
    }
}

impl Display for HttpDate {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let week_day = match self.week_day {
            1 => b"Mon",
            2 => b"Tue",
            3 => b"Wed",
            4 => b"Thu",
            5 => b"Fri",
            6 => b"Sat",
            7 => b"Sun",
            _ => unreachable!(),
        };
        let month = match self.month {
            1 => b"Jan",
            2 => b"Feb",
            3 => b"Mar",
            4 => b"Apr",
            5 => b"May",
            6 => b"Jun",
            7 => b"Jul",
            8 => b"Aug",
            9 => b"Sep",
            10 => b"Oct",
            11 => b"Nov",
            12 => b"Dec",
            _ => unreachable!(),
        };
        let mut buf: [u8; 29] = [
            // Too long to write as: b"Thu, 01 Jan 1970 00:00:00 GMT"
            b' ', b' ', b' ', b',', b' ', b'0', b'0', b' ', b' ', b' ', b' ', b' ', b'0', b'0',
            b'0', b'0', b' ', b'0', b'0', b':', b'0', b'0', b':', b'0', b'0', b' ', b'G', b'M',
            b'T',
        ];
        buf[0] = week_day[0];
        buf[1] = week_day[1];
        buf[2] = week_day[2];
        buf[5] = b'0' + (self.day / 10) as u8;
        buf[6] = b'0' + (self.day % 10) as u8;
        buf[8] = month[0];
        buf[9] = month[1];
        buf[10] = month[2];
        buf[12] = b'0' + (self.year / 1000) as u8;
        buf[13] = b'0' + (self.year / 100 % 10) as u8;
        buf[14] = b'0' + (self.year / 10 % 10) as u8;
        buf[15] = b'0' + (self.year % 10) as u8;
        buf[17] = b'0' + (self.hour / 10) as u8;
        buf[18] = b'0' + (self.hour % 10) as u8;
        buf[20] = b'0' + (self.minute / 10) as u8;
        buf[21] = b'0' + (self.minute % 10) as u8;
        buf[23] = b'0' + (self.second / 10) as u8;
        buf[24] = b'0' + (self.second % 10) as u8;
        f.write_str(from_utf8(&buf[..]).unwrap())
    }
}

impl PartialEq for HttpDate {
    fn eq(&self, other: &HttpDate) -> bool {
        SystemTime::from(*self) == SystemTime::from(*other)
    }
}

impl PartialOrd for HttpDate {
    fn partial_cmp(&self, other: &HttpDate) -> Option<std::cmp::Ordering> {
        SystemTime::from(*self).partial_cmp(&SystemTime::from(*other))
    }
}

fn is_leap_year(year: u16) -> bool {
    year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, UNIX_EPOCH};

    use super::{fmt_http_date, parse_http_date, HttpDate, SECONDS_IN_DAY, SECONDS_IN_HOUR};

    #[test]
    fn test_rfc_example() {
        let d = UNIX_EPOCH + Duration::from_secs(784111777);
        assert_eq!(
            d,
            parse_http_date("Sun, 06 Nov 1994 08:49:37 GMT").expect("#1")
        );
        assert_eq!(
            d,
            parse_http_date("Sunday, 06-Nov-94 08:49:37 GMT").expect("#2")
        );
        assert_eq!(d, parse_http_date("Sun Nov  6 08:49:37 1994").expect("#3"));
    }

    #[test]
    fn test2() {
        let d = UNIX_EPOCH + Duration::from_secs(1475419451);
        assert_eq!(
            d,
            parse_http_date("Sun, 02 Oct 2016 14:44:11 GMT").expect("#1")
        );
        assert!(parse_http_date("Sun Nov 10 08:00:00 1000").is_err());
        assert!(parse_http_date("Sun Nov 10 08*00:00 2000").is_err());
        assert!(parse_http_date("Sunday, 06-Nov-94 08+49:37 GMT").is_err());
    }

    #[test]
    fn test3() {
        let mut d = UNIX_EPOCH;
        assert_eq!(d, parse_http_date("Thu, 01 Jan 1970 00:00:00 GMT").unwrap());
        d += Duration::from_secs(SECONDS_IN_HOUR);
        assert_eq!(d, parse_http_date("Thu, 01 Jan 1970 01:00:00 GMT").unwrap());
        d += Duration::from_secs(SECONDS_IN_DAY);
        assert_eq!(d, parse_http_date("Fri, 02 Jan 1970 01:00:00 GMT").unwrap());
        d += Duration::from_secs(2592000);
        assert_eq!(d, parse_http_date("Sun, 01 Feb 1970 01:00:00 GMT").unwrap());
        d += Duration::from_secs(2592000);
        assert_eq!(d, parse_http_date("Tue, 03 Mar 1970 01:00:00 GMT").unwrap());
        d += Duration::from_secs(31536005);
        assert_eq!(d, parse_http_date("Wed, 03 Mar 1971 01:00:05 GMT").unwrap());
        d += Duration::from_secs(15552000);
        assert_eq!(d, parse_http_date("Mon, 30 Aug 1971 01:00:05 GMT").unwrap());
        d += Duration::from_secs(6048000);
        assert_eq!(d, parse_http_date("Mon, 08 Nov 1971 01:00:05 GMT").unwrap());
        d += Duration::from_secs(864000000);
        assert_eq!(d, parse_http_date("Fri, 26 Mar 1999 01:00:05 GMT").unwrap());
    }

    #[test]
    fn test_fmt() {
        let d = UNIX_EPOCH;
        assert_eq!(fmt_http_date(d), "Thu, 01 Jan 1970 00:00:00 GMT");
        let d = UNIX_EPOCH + Duration::from_secs(1475419451);
        assert_eq!(fmt_http_date(d), "Sun, 02 Oct 2016 14:44:11 GMT");
    }

    #[test]
    fn size_of() {
        assert_eq!(::std::mem::size_of::<HttpDate>(), 8);
    }
}
