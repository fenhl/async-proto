use {
    std::{
        future::Future,
        pin::Pin,
    },
    chrono::{
        offset::LocalResult,
        prelude::*,
    },
    tokio::io::{
        AsyncRead,
        AsyncWrite,
    },
    async_proto_derive::impl_protocol_for,
    crate::{
        Protocol,
        ReadError,
        WriteError,
    },
};
#[cfg(any(feature = "read-sync", feature = "write-sync"))] use std::io::prelude::*;

impl_protocol_for! {
    #[cfg_attr(docsrs, doc(cfg(feature = "chrono")))]
    struct Utc;
}

#[cfg_attr(docsrs, doc(cfg(feature = "chrono")))]
impl Protocol for FixedOffset {
    fn read<'a, R: AsyncRead + Unpin + Send + 'a>(stream: &'a mut R) -> Pin<Box<dyn Future<Output = Result<Self, ReadError>> + Send + 'a>> {
        Box::pin(async move {
            Self::east_opt(i32::read(stream).await?).ok_or_else(|| ReadError::Custom(format!("read an invalid UTC offset")))
        })
    }

    fn write<'a, W: AsyncWrite + Unpin + Send + 'a>(&'a self, sink: &'a mut W) -> Pin<Box<dyn Future<Output = Result<(), WriteError>> + Send + 'a>> {
        Box::pin(async move {
            self.local_minus_utc().write(sink).await
        })
    }

    #[cfg(feature = "read-sync")]
    #[cfg_attr(docsrs, doc(cfg(feature = "read-sync")))]
    fn read_sync(stream: &mut impl Read) -> Result<Self, ReadError> {
        Self::east_opt(i32::read_sync(stream)?).ok_or_else(|| ReadError::Custom(format!("read an invalid UTC offset")))
    }

    #[cfg(feature = "write-sync")]
    #[cfg_attr(docsrs, doc(cfg(feature = "write-sync")))]
    fn write_sync(&self, sink: &mut impl Write) -> Result<(), WriteError> {
        self.local_minus_utc().write_sync(sink)
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "chrono")))]
impl<Tz: Protocol + TimeZone + Send + Sync> Protocol for DateTime<Tz>
where Tz::Offset: Sync {
    fn read<'a, R: AsyncRead + Unpin + Send + 'a>(stream: &'a mut R) -> Pin<Box<dyn Future<Output = Result<Self, ReadError>> + Send + 'a>> {
        Box::pin(async move {
            match Tz::read(stream).await?.timestamp_opt(i64::read(stream).await?, u32::read(stream).await?) {
                LocalResult::Single(dt) => Ok(dt),
                LocalResult::None => Err(ReadError::Custom(format!("read a nonexistent timestamp"))),
                LocalResult::Ambiguous(dt1, dt2) => Err(ReadError::Custom(format!("read an ambiguous timestamp that could refer to {:?} or {:?}", dt1, dt2))),
            }
        })
    }

    fn write<'a, W: AsyncWrite + Unpin + Send + 'a>(&'a self, sink: &'a mut W) -> Pin<Box<dyn Future<Output = Result<(), WriteError>> + Send + 'a>> {
        Box::pin(async move {
            self.timezone().write(sink).await?;
            self.timestamp().write(sink).await?;
            self.timestamp_subsec_nanos().write(sink).await?;
            Ok(())
        })
    }

    #[cfg(feature = "read-sync")]
    #[cfg_attr(docsrs, doc(cfg(feature = "read-sync")))]
    fn read_sync(stream: &mut impl Read) -> Result<Self, ReadError> {
        match Tz::read_sync(stream)?.timestamp_opt(i64::read_sync(stream)?, u32::read_sync(stream)?) {
            LocalResult::Single(dt) => Ok(dt),
            LocalResult::None => Err(ReadError::Custom(format!("read a nonexistent timestamp"))),
            LocalResult::Ambiguous(dt1, dt2) => Err(ReadError::Custom(format!("read an ambiguous timestamp that could refer to {:?} or {:?}", dt1, dt2))),
        }
}

    #[cfg(feature = "write-sync")]
    #[cfg_attr(docsrs, doc(cfg(feature = "write-sync")))]
    fn write_sync(&self, sink: &mut impl Write) -> Result<(), WriteError> {
        self.timezone().write_sync(sink)?;
        self.timestamp().write_sync(sink)?;
        self.timestamp_subsec_nanos().write_sync(sink)?;
        Ok(())
    }
}
