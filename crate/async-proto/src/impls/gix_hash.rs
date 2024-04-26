use {
    gix_hash::ObjectId,
    async_proto_derive::impl_protocol_for,
    crate::Protocol,
};

#[derive(Protocol)]
#[async_proto(internal)]
struct OidProxy([u8; 20]);

impl From<OidProxy> for ObjectId {
    fn from(OidProxy(sha1): OidProxy) -> Self {
        Self::Sha1(sha1)
    }
}

impl<'a> From<&'a ObjectId> for OidProxy {
    fn from(oid: &ObjectId) -> Self {
        match *oid {
            ObjectId::Sha1(sha1) => Self(sha1),
        }
    }
}

impl_protocol_for! {
    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "gix-hash")))))]
    #[async_proto(via = OidProxy)]
    type ObjectId;
}
