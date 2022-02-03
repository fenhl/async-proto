use {
    std::convert::TryFrom, //TODO upgrade to Rust 2021?
    async_proto_derive::impl_protocol_for,
    crate::{
        Protocol,
        ReadError,
    },
};

#[derive(Protocol)]
#[async_proto(internal)]
struct TzProxy {
    name: String,
}

impl TryFrom<TzProxy> for chrono_tz::Tz {
    type Error = ReadError;

    fn try_from(TzProxy { name }: TzProxy) -> Result<Self, ReadError> {
        name.parse().map_err(ReadError::Custom)
    }
}

impl<'a> From<&'a chrono_tz::Tz> for TzProxy {
    fn from(tz: &chrono_tz::Tz) -> Self {
        Self { name: tz.name().to_owned() }
    }
}

impl_protocol_for! {
    #[cfg_attr(docsrs, doc(cfg(feature = "chrono-tz")))]
    /// A timezone is represented as an [IANA timezone identifier](https://data.iana.org/time-zones/theory.html#naming).
    #[async_proto(via = TzProxy)]
    type chrono_tz::Tz;
}
