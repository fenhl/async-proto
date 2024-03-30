use {
    enumset::{
        EnumSet,
        EnumSetType,
        EnumSetTypeWithRepr,
    },
    async_proto_derive::impl_protocol_for,
    crate::Protocol,
};

#[derive(Protocol)]
#[async_proto(internal, where())]
struct EnumSetProxy<T: EnumSetType + EnumSetTypeWithRepr>(<T as EnumSetTypeWithRepr>::Repr)
where <T as EnumSetTypeWithRepr>::Repr: Protocol + Send + Sync;

impl<T: EnumSetType + EnumSetTypeWithRepr> From<EnumSetProxy<T>> for EnumSet<T>
where <T as EnumSetTypeWithRepr>::Repr: Protocol + Send + Sync {
    fn from(EnumSetProxy(repr): EnumSetProxy<T>) -> Self {
        Self::from_repr_truncated(repr)
    }
}

impl<'a, T: EnumSetType + EnumSetTypeWithRepr> From<&'a EnumSet<T>> for EnumSetProxy<T>
where <T as EnumSetTypeWithRepr>::Repr: Protocol + Send + Sync {
    fn from(set: &EnumSet<T>) -> Self {
        Self(set.as_repr())
    }
}

impl_protocol_for! {
    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "enumset")))))]
    #[async_proto(attr(doc = "The type will be read via [`from_repr_truncated`](enumset::EnumSet::from_repr_truncated), ignoring invalid variants."))]
    #[async_proto(via = EnumSetProxy<T>, where())]
    type EnumSet<T: EnumSetType + EnumSetTypeWithRepr>
    where <T as EnumSetTypeWithRepr>::Repr: Protocol + Send + Sync;
}
