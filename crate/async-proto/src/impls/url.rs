use {
    async_proto_derive::impl_protocol_for,
    crate::ReadErrorKind,
};

impl_protocol_for! {
    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "url")))))]
    #[async_proto(attr(doc = "A URL is represented as a string."))]
    #[async_proto(as_string, map_err = |e: url::ParseError| ReadErrorKind::Custom(e.to_string()))]
    type url::Url;
}
