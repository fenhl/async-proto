use {
    noisy_float::{
        FloatChecker,
        NoisyFloat,
        prelude::*,
    },
    async_proto_derive::impl_protocol_for,
    crate::{
        Protocol,
        ReadErrorKind,
    },
};

#[derive(Protocol)]
#[async_proto(internal)]
struct NoisyFloatProxy<F: Float> {
    raw: F,
}

impl<F: Float, C: FloatChecker<F>> TryFrom<NoisyFloatProxy<F>> for NoisyFloat<F, C> {
    type Error = ReadErrorKind;

    fn try_from(NoisyFloatProxy { raw }: NoisyFloatProxy<F>) -> Result<Self, ReadErrorKind> {
        Self::try_new(raw).ok_or_else(|| ReadErrorKind::Custom(format!("read an invalid noisy float")))
    }
}

impl<'a, F: Float, C: FloatChecker<F>> From<&'a NoisyFloat<F, C>> for NoisyFloatProxy<F> {
    fn from(float: &NoisyFloat<F, C>) -> Self {
        Self { raw: float.raw() }
    }
}

impl_protocol_for! {
    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "noisy_float")))))]
    /// A noisy float is represented like its underlying type. Reading an invalid float produces a [`ReadErrorKind::Custom`].
    #[async_proto(via = NoisyFloatProxy<F>, where(F: Protocol + Float + Send + Sync + 'static, C: FloatChecker<F> + Send + Sync))]
    type NoisyFloat<F: Float, C: FloatChecker<F>>;
}
