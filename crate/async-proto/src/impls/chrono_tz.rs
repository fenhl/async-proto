use {
    async_proto_derive::impl_protocol_for,
    crate::{
        Protocol,
        ReadErrorKind,
    },
};

#[derive(Protocol)]
#[async_proto(internal)]
struct TzProxy {
    name: String,
}

impl TryFrom<TzProxy> for chrono_tz::Tz {
    type Error = ReadErrorKind;

    fn try_from(TzProxy { name }: TzProxy) -> Result<Self, ReadErrorKind> {
        name.parse::<Self>().map_err(|e| ReadErrorKind::Custom(e.to_string()))
    }
}

impl<'a> From<&'a chrono_tz::Tz> for TzProxy {
    fn from(tz: &chrono_tz::Tz) -> Self {
        Self { name: tz.name().to_owned() }
    }
}

impl_protocol_for! {
    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "chrono-tz")))))]
    #[async_proto(attr(doc = "A timezone is represented as an [IANA timezone identifier](https://data.iana.org/time-zones/theory.html#naming)."))]
    #[async_proto(via = TzProxy)]
    type chrono_tz::Tz;
}
