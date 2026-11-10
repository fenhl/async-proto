use {
    jiff::Timestamp,
    async_proto_derive::impl_protocol_for,
    crate::{
        Protocol,
        ReadErrorKind,
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

impl_protocol_for! {
    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "jiff")))))]
    #[async_proto(attr(doc = "Uses the same representation as `chrono::DateTime<Utc>`."))]
    #[async_proto(via = TimestampProxy)]
    type Timestamp;
}
