use {
    async_proto_derive::impl_protocol_for,
    crate::ReadErrorKind,
};

impl_protocol_for! {
    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "chrono-tz")))))]
    #[async_proto(attr(doc = "A timezone is represented as an [IANA timezone identifier](https://data.iana.org/time-zones/theory.html#naming)."))]
    #[async_proto(as_string, map_err = |e: chrono_tz::ParseError| ReadErrorKind::Custom(e.to_string()))]
    type chrono_tz::Tz;
}
