use chrono::{DateTime, Datelike, FixedOffset, TimeZone, Timelike, Utc};
use time::{Date, Month, OffsetDateTime, Time, UtcOffset};

pub(crate) fn convert_time_to_chrono(time: OffsetDateTime) -> DateTime<Utc> {
    let offset = FixedOffset::east_opt(time.offset().whole_seconds()).unwrap();
    offset
        .with_ymd_and_hms(
            time.year(),
            time.month() as u32,
            time.day().into(),
            time.hour().into(),
            time.minute().into(),
            time.second().into(),
        )
        .unwrap()
        .to_utc()
}

pub(crate) fn convert_chrono_to_time(dt: DateTime<Utc>) -> OffsetDateTime {
    OffsetDateTime::new_in_offset(
        Date::from_calendar_date(
            dt.year(),
            Month::try_from(dt.month() as u8).unwrap(),
            dt.day() as u8,
        )
        .unwrap(),
        Time::from_hms(dt.hour() as u8, dt.minute() as u8, dt.second() as u8).unwrap(),
        UtcOffset::from_whole_seconds(dt.fixed_offset().offset().local_minus_utc()).unwrap(),
    )
}
