use {
    async_proto_derive::impl_protocol_for,
    crate::{
        Protocol,
        ReadErrorKind,
        WriteErrorKind,
    },
};

#[derive(Protocol)]
#[async_proto(internal)]
struct TypeProxy(String);

impl TryFrom<TypeProxy> for os_info::Type {
    type Error = ReadErrorKind;

    fn try_from(TypeProxy(s): TypeProxy) -> Result<Self, Self::Error> {
        serde_plain::from_str(&s).map_err(|e| ReadErrorKind::Custom(e.to_string()))
    }
}

impl<'a> TryFrom<&'a os_info::Type> for TypeProxy {
    type Error = WriteErrorKind;

    fn try_from(ty: &os_info::Type) -> Result<Self, Self::Error> {
        Ok(Self(serde_plain::to_string(ty).map_err(|e| WriteErrorKind::Custom(e.to_string()))?))
    }
}

impl_protocol_for! {
    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "os_info")))))]
    #[async_proto(attr(doc = "An OS type is represented as its enum variant name."))]
    #[async_proto(via = TypeProxy)]
    type os_info::Type;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "os_info")))))]
    enum os_info::Version {
        Unknown,
        Semantic(u64, u64, u64),
        Rolling(Option<String>),
        Custom(String),
    }
}
