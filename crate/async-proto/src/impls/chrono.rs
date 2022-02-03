use {
    std::convert::TryFrom, //TODO upgrade to Rust 2021?
    chrono::{
        offset::LocalResult,
        prelude::*,
    },
    async_proto_derive::impl_protocol_for,
    crate::{
        Protocol,
        ReadError,
    },
};

#[derive(Protocol)]
#[async_proto(internal)]
struct NaiveDateProxy {
    num_days_from_ce: i32,
}

impl TryFrom<NaiveDateProxy> for NaiveDate {
    type Error = ReadError;

    fn try_from(NaiveDateProxy { num_days_from_ce }: NaiveDateProxy) -> Result<Self, ReadError> {
        Self::from_num_days_from_ce_opt(num_days_from_ce).ok_or_else(|| ReadError::Custom(format!("out-of-range date")))
    }
}

impl<'a> From<&'a NaiveDate> for NaiveDateProxy {
    fn from(date: &NaiveDate) -> Self {
        Self { num_days_from_ce: date.num_days_from_ce() }
    }
}

#[derive(Protocol)]
#[async_proto(internal)]
struct FixedOffsetProxy {
    east: i32,
}

impl TryFrom<FixedOffsetProxy> for FixedOffset {
    type Error = ReadError;

    fn try_from(FixedOffsetProxy { east }: FixedOffsetProxy) -> Result<Self, ReadError> {
        Self::east_opt(east).ok_or_else(|| ReadError::Custom(format!("FixedOffset::east out of bounds")))
    }
}

impl<'a> From<&'a FixedOffset> for FixedOffsetProxy {
    fn from(offset: &FixedOffset) -> Self {
        Self { east: offset.local_minus_utc() }
    }
}

#[derive(Protocol)]
#[async_proto(internal)]
struct DateTimeProxy<Tz: TimeZone> {
    timezone: Tz,
    timestamp: i64,
    timestamp_subsec_nanos: u32,
}

impl<Tz: TimeZone> TryFrom<DateTimeProxy<Tz>> for DateTime<Tz> {
    type Error = ReadError;

    fn try_from(DateTimeProxy { timezone, timestamp, timestamp_subsec_nanos }: DateTimeProxy<Tz>) -> Result<Self, ReadError> {
        match timezone.timestamp_opt(timestamp, timestamp_subsec_nanos) {
            LocalResult::Single(dt) => Ok(dt),
            LocalResult::None => Err(ReadError::Custom(format!("read a nonexistent timestamp"))),
            LocalResult::Ambiguous(dt1, dt2) => Err(ReadError::Custom(format!("read an ambiguous timestamp that could refer to {:?} or {:?}", dt1, dt2))),
        }
    }
}

impl<'a, Tz: TimeZone> From<&'a DateTime<Tz>> for DateTimeProxy<Tz> {
    fn from(date: &DateTime<Tz>) -> Self {
        Self {
            timezone: date.timezone(),
            timestamp: date.timestamp(),
            timestamp_subsec_nanos: date.timestamp_subsec_nanos(),
        }
    }
}

impl_protocol_for! {
    #[cfg_attr(docsrs, doc(cfg(feature = "chrono")))]
    struct Utc;

    #[cfg_attr(docsrs, doc(cfg(feature = "chrono")))]
    #[async_proto(via = NaiveDateProxy)]
    type NaiveDate;

    #[cfg_attr(docsrs, doc(cfg(feature = "chrono")))]
    #[async_proto(via = FixedOffsetProxy)]
    type FixedOffset;

    #[cfg_attr(docsrs, doc(cfg(feature = "chrono")))]
    #[async_proto(via = DateTimeProxy<Tz>, where(Tz: Protocol + TimeZone + Send + Sync + 'static, Tz::Offset: Sync))]
    type DateTime<Tz: TimeZone>;
}
