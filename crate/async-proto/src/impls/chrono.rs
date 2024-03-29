use {
    chrono::{
        offset::LocalResult,
        prelude::*,
    },
    async_proto_derive::impl_protocol_for,
    crate::{
        Protocol,
        ReadErrorKind,
    },
};

#[derive(Protocol)]
#[async_proto(internal)]
struct NaiveDateProxy {
    num_days_from_ce: i32,
}

impl TryFrom<NaiveDateProxy> for NaiveDate {
    type Error = ReadErrorKind;

    fn try_from(NaiveDateProxy { num_days_from_ce }: NaiveDateProxy) -> Result<Self, ReadErrorKind> {
        Self::from_num_days_from_ce_opt(num_days_from_ce).ok_or_else(|| ReadErrorKind::Custom(format!("out-of-range date")))
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
    type Error = ReadErrorKind;

    fn try_from(FixedOffsetProxy { east }: FixedOffsetProxy) -> Result<Self, ReadErrorKind> {
        Self::east_opt(east).ok_or_else(|| ReadErrorKind::Custom(format!("FixedOffset::east out of bounds")))
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
    type Error = ReadErrorKind;

    fn try_from(DateTimeProxy { timezone, timestamp, timestamp_subsec_nanos }: DateTimeProxy<Tz>) -> Result<Self, ReadErrorKind> {
        match timezone.timestamp_opt(timestamp, timestamp_subsec_nanos) {
            LocalResult::Single(dt) => Ok(dt),
            LocalResult::None => Err(ReadErrorKind::Custom(format!("read a nonexistent timestamp"))),
            LocalResult::Ambiguous(dt1, dt2) => Err(ReadErrorKind::Custom(format!("read an ambiguous timestamp that could refer to {:?} or {:?}", dt1, dt2))),
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
    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "chrono")))))]
    struct Utc;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "chrono")))))]
    #[async_proto(via = NaiveDateProxy)]
    type NaiveDate;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "chrono")))))]
    #[async_proto(via = FixedOffsetProxy)]
    type FixedOffset;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "chrono")))))]
    #[async_proto(via = DateTimeProxy<Tz>, where(Tz: Protocol + TimeZone + Send + Sync + 'static, Tz::Offset: Sync))]
    type DateTime<Tz: TimeZone>;
}
