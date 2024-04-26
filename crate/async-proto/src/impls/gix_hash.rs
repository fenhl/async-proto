use async_proto_derive::impl_protocol_for;

impl_protocol_for! {
    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "gix-hash")))))]
    enum gix_hash::ObjectId {
        Sha1([u8; 20]),
    }
}
