use {
    uuid::Uuid,
    async_proto_derive::impl_protocol_for,
    crate::Protocol,
};

#[derive(Protocol)]
#[async_proto(internal)]
struct UuidProxy(uuid::Bytes);

impl From<UuidProxy> for Uuid {
    fn from(UuidProxy(bytes): UuidProxy) -> Self {
        Self::from_bytes(bytes)
    }
}

impl<'a> From<&'a Uuid> for UuidProxy {
    fn from(uuid: &Uuid) -> Self {
        Self(uuid.into_bytes())
    }
}

impl_protocol_for! {
    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "uuid")))))]
    #[async_proto(via = UuidProxy)]
    type Uuid;
}
