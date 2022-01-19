use {
    std::{
        future::Future,
        pin::Pin,
    },
    noisy_float::{
        FloatChecker,
        NoisyFloat,
        prelude::*,
    },
    tokio::io::{
        AsyncRead,
        AsyncWrite,
    },
    crate::{
        Protocol,
        ReadError,
        WriteError,
    },
};
#[cfg(any(feature = "read-sync", feature = "write-sync"))] use std::io::prelude::*;

#[cfg_attr(docsrs, doc(cfg(feature = "noisy_float")))]
/// A noisy float is represented like its underlying type. Reading an invalid float produces a [`ReadError::Custom`].
impl<F: Protocol + Float + Send + Sync, C: FloatChecker<F> + Send + Sync> Protocol for NoisyFloat<F, C> {
    fn read<'a, R: AsyncRead + Unpin + Send + 'a>(stream: &'a mut R) -> Pin<Box<dyn Future<Output = Result<Self, ReadError>> + Send + 'a>> {
        Box::pin(async move {
            Self::try_new(F::read(stream).await?).ok_or_else(|| ReadError::Custom(format!("read an invalid noisy float")))
        })
    }

    fn write<'a, W: AsyncWrite + Unpin + Send + 'a>(&'a self, sink: &'a mut W) -> Pin<Box<dyn Future<Output = Result<(), WriteError>> + Send + 'a>> {
        Box::pin(async move {
            self.raw().write(sink).await
        })
    }

    #[cfg(feature = "read-sync")]
    #[cfg_attr(docsrs, doc(cfg(feature = "read-sync")))]
    fn read_sync(stream: &mut impl Read) -> Result<Self, ReadError> {
        Self::try_new(F::read_sync(stream)?).ok_or_else(|| ReadError::Custom(format!("read an invalid noisy float")))
    }

    #[cfg(feature = "write-sync")]
    #[cfg_attr(docsrs, doc(cfg(feature = "write-sync")))]
    fn write_sync(&self, sink: &mut impl Write) -> Result<(), WriteError> {
        self.raw().write_sync(sink)
    }
}
