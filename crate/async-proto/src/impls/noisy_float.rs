use {
    std::convert::TryFrom, //TODO upgrade to Rust 2021?
    noisy_float::{
        FloatChecker,
        NoisyFloat,
        prelude::*,
    },
    async_proto_derive::impl_protocol_for,
    crate::{
        Protocol,
        ReadError,
    },
};

#[derive(Protocol)]
#[async_proto(internal)]
struct NoisyFloatProxy<F: Float> {
    raw: F,
}

impl<F: Float, C: FloatChecker<F>> TryFrom<NoisyFloatProxy<F>> for NoisyFloat<F, C> {
    type Error = ReadError;

    fn try_from(NoisyFloatProxy { raw }: NoisyFloatProxy<F>) -> Result<Self, ReadError> {
        Self::try_new(raw).ok_or_else(|| ReadError::Custom(format!("read an invalid noisy float")))
    }
}

impl<'a, F: Float, C: FloatChecker<F>> From<&'a NoisyFloat<F, C>> for NoisyFloatProxy<F> {
    fn from(float: &NoisyFloat<F, C>) -> Self {
        Self { raw: float.raw() }
    }
}

impl_protocol_for! {
    #[cfg_attr(docsrs, doc(cfg(feature = "noisy_float")))]
    /// A noisy float is represented like its underlying type. Reading an invalid float produces a [`ReadError::Custom`].
    #[async_proto(via = NoisyFloatProxy<F>)]
    type NoisyFloat<F: Float, C: FloatChecker<F>>;
}
