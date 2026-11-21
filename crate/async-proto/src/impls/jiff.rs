use {
    jiff::{
        Span,
        Timestamp,
        ZonedDateTime,
        civil,
        tz::TimeZone,
    },
    async_proto_derive::impl_protocol_for,
    crate::{
        Protocol,
        ReadErrorKind,
        WriteErrorKind,
    },
};

#[derive(Protocol)]
#[async_proto(internal)]
struct TimestampProxy {
    timestamp: i64,
    timestamp_subsec_nanos: u32,
}

impl TryFrom<TimestampProxy> for Timestamp {
    type Error = ReadErrorKind;

    fn try_from(TimestampProxy { timestamp, timestamp_subsec_nanos }: TimestampProxy) -> Result<Self, ReadErrorKind> {
        Self::from_second(timestamp).map_err(|e| ReadErrorKind::Custom(e.to_string()))?
            .checked_add(std::time::Duration::from_nanos(timestamp_subsec_nanos.into()))
            .map_err(|e| ReadErrorKind::Custom(e.to_string()))
    }
}

impl<'a> From<&'a Timestamp> for TimestampProxy {
    fn from(timestamp: &Timestamp) -> Self {
        let mut secs = timestamp.as_second();
        let nanos = timestamp.subsec_nanosecond();
        let timestamp_subsec_nanos = if nanos < 0 {
            // does not panic or overflow because the minimum timestamp is `date(-9999, 1, 1).at(0, 0, 0, 0)`
            secs -= 1;
            // does not panic or overflow because `nanos` is in `-999_999_999..=999_999_999`
            (1_000_000_000 + nanos) as u32
        } else {
            nanos as u32
        };
        Self { timestamp: secs, timestamp_subsec_nanos }
    }
}

const CE: civil::Date = civil::date(0, 12, 31);

#[derive(Protocol)]
#[async_proto(internal)]
struct CivilDateProxy {
    num_days_from_ce: i32,
}

impl TryFrom<CivilDateProxy> for civil::Date {
    type Error = ReadErrorKind;

    fn try_from(CivilDateProxy { num_days_from_ce }: CivilDateProxy) -> Result<Self, ReadErrorKind> {
        CE.checked_add(Span::default().try_days(num_days_from_ce).map_err(|e| ReadErrorKind::Custom(e.to_string()))?).map_err(|e| ReadErrorKind::Custom(e.to_string()))
    }
}

impl<'a> From<&'a civil::Date> for CivilDateProxy {
    fn from(date: &civil::Date) -> Self {
        Self { num_days_from_ce: (*date - CE).get_days() }
    }
}

#[derive(Protocol)]
#[async_proto(internal)]
struct TimeZoneProxy {
    iana_name: String,
}

impl TryFrom<TimeZoneProxy> for TimeZone {
    type Error = ReadErrorKind;

    fn try_from(TimeZoneProxy { iana_name }: TimeZoneProxy) -> Result<Self, ReadErrorKind> {
        Self::get(&iana_name).map_err(|e| ReadErrorKind::Custom(e.to_string()))
    }
}

impl<'a> TryFrom<&'a TimeZone> for TimeZoneProxy {
    type Error = WriteErrorKind;

    fn try_from(tz: &TimeZone) -> Result<Self, WriteErrorKind> {
        Ok(Self { iana_name: tz.iana_name().ok_or_else(|| WriteErrorKind::Custom(format!("attempted to serialize timezone with no IANA name")))?.to_owned() })
    }
}

#[derive(Protocol)]
#[async_proto(internal)]
struct ZonedDateTimeProxy {
    timezone: TimeZone,
    timestamp: Timestamp,
}

impl From<ZonedDateTimeProxy> for ZonedDateTime {
    fn from(ZonedDateTimeProxy { timezone, timestamp }: ZonedDateTimeProxy) -> Self {
        Self::new(timestamp, timezone)
    }
}

impl<'a> From<&'a ZonedDateTime> for ZonedDateTimeProxy {
    fn from(zoned: &ZonedDateTime) -> Self {
        Self {
            timezone: zoned.time_zone().clone(),
            timestamp: zoned.timestamp(),
        }
    }
}

impl_protocol_for! {
    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "jiff")))))]
    #[async_proto(attr(doc = "Uses the same representation as `chrono::DateTime<Utc>`."))]
    #[async_proto(via = TimestampProxy)]
    type Timestamp;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "jiff")))))]
    #[async_proto(attr(doc = "Uses the same representation as `chrono::NaiveDate`."))]
    #[async_proto(via = CivilDateProxy)]
    type civil::Date;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "jiff")))))]
    #[async_proto(attr(doc = "Must have an IANA name. Uses the same representation as `chrono_tz::Tz`."))]
    #[async_proto(via = TimeZoneProxy)]
    type TimeZone;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "jiff")))))]
    #[async_proto(attr(doc = "Time zone must have an IANA name. Uses the same representation as `chrono::DateTime<chrono_tz::Tz>`."))]
    #[async_proto(via = ZonedDateTimeProxy)]
    type ZonedDateTime;
}

#[cfg(feature = "chrono")]
#[test]
fn test_chrono_compat() {
    use {
        jiff::civil,
        crate::Protocol as _,
    };

    for (year, month, day) in [
        (-1234, 5, 6),
        (0, 1, 1),
        (0, 12, 31),
        (1, 1, 1),
        (2, 1, 1),
        (1234, 5, 6),
        (1970, 1, 1),
        (2000, 2, 29),
        (2012, 10, 21),
        (2012, 10, 22),
    ] {
        let chrono_date = chrono::NaiveDate::from_ymd_opt(year.into(), month.try_into().unwrap(), day.try_into().unwrap()).unwrap();
        let jiff_date = civil::date(year, month, day);
        let mut chrono_buf = Vec::default();
        let mut jiff_buf = Vec::default();
        chrono_date.write_sync(&mut chrono_buf).unwrap();
        jiff_date.write_sync(&mut jiff_buf).unwrap();
        assert_eq!(chrono_buf, jiff_buf);
        assert_eq!(chrono::NaiveDate::read_sync(&mut &*chrono_buf).unwrap(), chrono_date);
        assert_eq!(civil::Date::read_sync(&mut &*jiff_buf).unwrap(), jiff_date);
    }
}
