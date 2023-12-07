use {
    std::{
        future::Future,
        io::prelude::*,
        pin::Pin,
    },
    bytes::Bytes,
    tokio::io::{
        AsyncRead,
        AsyncReadExt as _,
        AsyncWrite,
        AsyncWriteExt as _,
    },
    crate::{
        ErrorContext,
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
        Box::pin(async move {
            let len = u64::read(stream).await?;
            let mut buf = vec![0; usize::try_from(len).map_err(|e| ReadError {
                context: ErrorContext::BuiltIn { for_type: "bytes::Bytes" },
                kind: e.into(),
            })?];
            stream.read_exact(&mut buf).await.map_err(|e| ReadError {
                context: ErrorContext::BuiltIn { for_type: "bytes::Bytes" },
                kind: e.into(),
            })?;
            Ok(buf.into())
        })
    }

    fn write<'a, W: AsyncWrite + Unpin + Send + 'a>(&'a self, sink: &'a mut W) -> Pin<Box<dyn Future<Output = Result<(), WriteError>> + Send + 'a>> {
        Box::pin(async move {
            u64::try_from(self.len()).map_err(|e| WriteError {
                context: ErrorContext::BuiltIn { for_type: "bytes::Bytes" },
                kind: e.into(),
            })?.write(sink).await?;
            sink.write_all(self).await.map_err(|e| WriteError {
                context: ErrorContext::BuiltIn { for_type: "bytes::Bytes" },
                kind: e.into(),
            })?;
            Ok(())
        })
    }

    fn read_sync(stream: &mut impl Read) -> Result<Self, ReadError> {
        let len = u64::read_sync(stream)?;
        let mut buf = vec![0; usize::try_from(len).map_err(|e| ReadError {
            context: ErrorContext::BuiltIn { for_type: "bytes::Bytes" },
            kind: e.into(),
        })?];
        stream.read_exact(&mut buf).map_err(|e| ReadError {
            context: ErrorContext::BuiltIn { for_type: "bytes::Bytes" },
            kind: e.into(),
        })?;
        Ok(buf.into())
    }

    fn write_sync(&self, sink: &mut impl Write) -> Result<(), WriteError> {
        u64::try_from(self.len()).map_err(|e| WriteError {
            context: ErrorContext::BuiltIn { for_type: "bytes::Bytes" },
            kind: e.into(),
        })?.write_sync(sink)?;
        sink.write_all(self).map_err(|e| WriteError {
            context: ErrorContext::BuiltIn { for_type: "bytes::Bytes" },
            kind: e.into(),
        })?;
        Ok(())
    }
}
