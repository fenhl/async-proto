use {
    std::{
        future::Future,
        io::prelude::*,
        pin::Pin,
    },
    bytes::Bytes,
    fallible_collections::FallibleVec as _,
    tokio::io::{
        AsyncRead,
        AsyncReadExt as _,
        AsyncWrite,
        AsyncWriteExt as _,
    },
    crate::{
        ErrorContext,
        LengthPrefixed,
        Protocol,
        ReadError,
        WriteError,
    },
};

/// A [`Bytes`] is prefixed with the length as a [`u64`].
///
/// Using [`Bytes`] is recommended for sending large amounts of data, since the [`Protocol`] implementation for `Vec<u8>` reads and writes each byte individually.
#[cfg_attr(docsrs, doc(cfg(feature = "bytes")))]
impl Protocol for Bytes {
    fn read<'a, R: AsyncRead + Unpin + Send + 'a>(stream: &'a mut R) -> Pin<Box<dyn Future<Output = Result<Self, ReadError>> + Send + 'a>> {
        Self::read_length_prefixed(stream, u64::MAX)
    }

    fn write<'a, W: AsyncWrite + Unpin + Send + 'a>(&'a self, sink: &'a mut W) -> Pin<Box<dyn Future<Output = Result<(), WriteError>> + Send + 'a>> {
        self.write_length_prefixed(sink, u64::MAX)
    }

    fn read_sync(stream: &mut impl Read) -> Result<Self, ReadError> {
        Self::read_length_prefixed_sync(stream, u64::MAX)
    }

    fn write_sync(&self, sink: &mut impl Write) -> Result<(), WriteError> {
        self.write_length_prefixed_sync(sink, u64::MAX)
    }
}

/// Using [`Bytes`] is recommended for sending large amounts of data, since the [`Protocol`] implementation for `Vec<u8>` reads and writes each byte individually.
#[cfg_attr(docsrs, doc(cfg(feature = "bytes")))]
impl LengthPrefixed for Bytes {
    fn read_length_prefixed<'a, R: AsyncRead + Unpin + Send + 'a>(stream: &'a mut R, max_len: u64) -> Pin<Box<dyn Future<Output = Result<Self, ReadError>> + Send + 'a>> {
        Box::pin(async move {
            let len = super::read_len(stream, max_len, || ErrorContext::BuiltIn { for_type: "bytes::Bytes" }).await?;
            let mut buf = Vec::default();
            buf.try_resize(len, 0).map_err(|e| ReadError {
                context: ErrorContext::BuiltIn { for_type: "bytes::Bytes" },
                kind: e.into(),
            })?;
            stream.read_exact(&mut buf).await.map_err(|e| ReadError {
                context: ErrorContext::BuiltIn { for_type: "bytes::Bytes" },
                kind: e.into(),
            })?;
            Ok(buf.into())
        })
    }

    fn write_length_prefixed<'a, W: AsyncWrite + Unpin + Send + 'a>(&'a self, sink: &'a mut W, max_len: u64) -> Pin<Box<dyn Future<Output = Result<(), WriteError>> + Send + 'a>> {
        Box::pin(async move {
            super::write_len(sink, self.len(), max_len, || ErrorContext::BuiltIn { for_type: "bytes::Bytes" }).await?;
            sink.write_all(self).await.map_err(|e| WriteError {
                context: ErrorContext::BuiltIn { for_type: "bytes::Bytes" },
                kind: e.into(),
            })?;
            Ok(())
        })
    }

    fn read_length_prefixed_sync(stream: &mut impl Read, max_len: u64) -> Result<Self, ReadError> {
        let len = super::read_len_sync(stream, max_len, || ErrorContext::BuiltIn { for_type: "bytes::Bytes" })?;
        let mut buf = Vec::default();
        buf.try_resize(len, 0).map_err(|e| ReadError {
            context: ErrorContext::BuiltIn { for_type: "bytes::Bytes" },
            kind: e.into(),
        })?;
        stream.read_exact(&mut buf).map_err(|e| ReadError {
            context: ErrorContext::BuiltIn { for_type: "bytes::Bytes" },
            kind: e.into(),
        })?;
        Ok(buf.into())
    }

    fn write_length_prefixed_sync(&self, sink: &mut impl Write, max_len: u64) -> Result<(), WriteError> {
        super::write_len_sync(sink, self.len(), max_len, || ErrorContext::BuiltIn { for_type: "bytes::Bytes" })?;
        sink.write_all(self).map_err(|e| WriteError {
            context: ErrorContext::BuiltIn { for_type: "bytes::Bytes" },
            kind: e.into(),
        })?;
        Ok(())
    }
}
