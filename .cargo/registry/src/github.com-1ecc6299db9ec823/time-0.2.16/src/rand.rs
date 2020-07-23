//! Implementation of `Distribution` for various structs.

use crate::{
    date::{MAX_YEAR, MIN_YEAR},
    internal_prelude::*,
    internals,
};
use rand::{
    distributions::{Distribution, Standard},
    Rng,
};

impl Distribution<Time> for Standard {
    #[inline]
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Time {
        Time {
            hour: rng.gen_range(0, 24),
            minute: rng.gen_range(0, 60),
            second: rng.gen_range(0, 60),
            nanosecond: rng.gen_range(0, 1_000_000_000),
        }
    }
}

impl Distribution<Date> for Standard {
    #[inline]
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Date {
        /// The minimum date allowed to be represented.
        const MIN_DATE: Date = internals::Date::from_ymd_unchecked(MIN_YEAR, 1, 1);
        /// The maximum date allowed to be represented.
        const MAX_DATE: Date = internals::Date::from_ymd_unchecked(MAX_YEAR, 12, 31);

        Date::from_julian_day(rng.gen_range(MIN_DATE.julian_day(), MAX_DATE.julian_day() + 1))
    }
}

impl Distribution<UtcOffset> for Standard {
    #[inline]
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> UtcOffset {
        UtcOffset {
            seconds: rng.gen_range(-86_399, 86_400),
        }
    }
}

impl Distribution<PrimitiveDateTime> for Standard {
    #[inline]
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> PrimitiveDateTime {
        PrimitiveDateTime {
            date: Standard.sample(rng),
            time: Standard.sample(rng),
        }
    }
}

impl Distribution<OffsetDateTime> for Standard {
    #[inline]
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> OffsetDateTime {
        OffsetDateTime {
            utc_datetime: Standard.sample(rng),
            offset: Standard.sample(rng),
        }
    }
}

impl Distribution<Duration> for Standard {
    #[inline]
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Duration {
        let seconds = Standard.sample(rng);
        Duration {
            seconds,
            nanoseconds: seconds.signum() as i32 * rng.gen_range(0, 1_000_000_000),
        }
    }
}

impl Distribution<Weekday> for Standard {
    #[inline]
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Weekday {
        use Weekday::*;

        match rng.gen_range(0, 7) {
            0 => Monday,
            1 => Tuesday,
            2 => Wednesday,
            3 => Thursday,
            4 => Friday,
            5 => Saturday,
            6 => Sunday,
            _ => unreachable!("values are 0 to 6 inclusive"),
        }
    }
}
