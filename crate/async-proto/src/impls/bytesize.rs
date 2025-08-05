use async_proto_derive::impl_protocol_for;

impl_protocol_for! {
    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "bytesize")))))]
    struct bytesize::ByteSize(u64);
}
