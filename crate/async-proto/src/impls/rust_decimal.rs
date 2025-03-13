use {
    rust_decimal::Decimal,
    async_proto_derive::impl_protocol_for,
    crate::Protocol,
};

#[derive(Protocol)]
#[async_proto(internal)]
struct DecimalProxy([u8; 16]);

impl From<DecimalProxy> for Decimal {
    fn from(DecimalProxy(bytes): DecimalProxy) -> Self {
        Self::deserialize(bytes)
    }
}

impl<'a> From<&'a Decimal> for DecimalProxy {
    fn from(value: &'a Decimal) -> Self {
        Self(value.serialize())
    }
}

impl_protocol_for! {
    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "rust_decimal")))))]
    #[async_proto(via = DecimalProxy)]
    type Decimal;
}
