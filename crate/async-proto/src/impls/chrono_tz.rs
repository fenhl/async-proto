use {
    std::{
        future::Future,
        pin::Pin,
    },
    crate::{
        Protocol,
        ReadError,
        WriteError,
    },
    tokio::io::{
        AsyncRead,
        AsyncWrite,
    },
};
#[cfg(any(feature = "read-sync", feature = "write-sync"))] use std::io::prelude::*;

#[cfg_attr(docsrs, doc(cfg(feature = "chrono-tz")))]
/// A timezone is represented as an [IANA timezone identifier](https://data.iana.org/time-zones/theory.html#naming).
impl Protocol for chrono_tz::Tz {
    fn read<'a, R: AsyncRead + Unpin + Send + 'a>(stream: &'a mut R) -> Pin<Box<dyn Future<Output = Result<Self, ReadError>> + Send + 'a>> {
        Box::pin(async move {
            Ok(String::read(stream).await?.parse().map_err(ReadError::Custom)?)
        })
    }

    fn write<'a, W: AsyncWrite + Unpin + Send + 'a>(&'a self, sink: &'a mut W) -> Pin<Box<dyn Future<Output = Result<(), WriteError>> + Send + 'a>> {
        Box::pin(async move {
            self.name().to_owned().write(sink).await
        })
    }

    #[cfg(feature = "read-sync")]
    #[cfg_attr(docsrs, doc(cfg(feature = "read-sync")))]
    fn read_sync(stream: &mut impl Read) -> Result<Self, ReadError> {
        Ok(String::read_sync(stream)?.parse().map_err(ReadError::Custom)?)
    }

    #[cfg(feature = "write-sync")]
    #[cfg_attr(docsrs, doc(cfg(feature = "write-sync")))]
    fn write_sync(&self, sink: &mut impl Write) -> Result<(), WriteError> {
        self.name().to_owned().write_sync(sink)
    }
}
