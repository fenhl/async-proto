use async_proto_derive::impl_protocol_for;

impl_protocol_for! {
    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "os_info")))))]
    enum os_info::Version {
        Unknown,
        Semantic(u64, u64, u64),
        Rolling(Option<String>),
        Custom(String),
    }
}
