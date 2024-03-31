use {
    std::{
        future::Future,
        io::prelude::*,
        pin::Pin,
    },
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
        ReadErrorKind,
        WriteError,
    },
};

/// A git object ID uses its native binary representation, a sequence of 20 bytes.
#[cfg_attr(docsrs, doc(cfg(feature = "git2")))]
impl Protocol for git2::Oid {
    fn read<'a, R: AsyncRead + Unpin + Send + 'a>(stream: &'a mut R) -> Pin<Box<dyn Future<Output = Result<Self, ReadError>> + Send + 'a>> {
        Box::pin(async move {
            let mut buf = [0; 20];
            stream.read_exact(&mut buf).await.map_err(|e| ReadError {
                context: ErrorContext::BuiltIn { for_type: "git2::Oid" },
                kind: e.into(),
            })?;
            Self::from_bytes(&buf).map_err(|e| ReadError {
                context: ErrorContext::BuiltIn { for_type: "git2::Oid" },
                kind: ReadErrorKind::Custom(e.to_string()),
            })
        })
    }

    fn write<'a, W: AsyncWrite + Unpin + Send + 'a>(&'a self, sink: &'a mut W) -> Pin<Box<dyn Future<Output = Result<(), WriteError>> + Send + 'a>> {
        Box::pin(async move {
            sink.write_all(self.as_bytes()).await.map_err(|e| WriteError {
                context: ErrorContext::BuiltIn { for_type: "git2::Oid" },
                kind: e.into(),
            })?;
            Ok(())
        })
    }

    fn read_sync(stream: &mut impl Read) -> Result<Self, ReadError> {
        let mut buf = [0; 20];
        stream.read_exact(&mut buf).map_err(|e| ReadError {
            context: ErrorContext::BuiltIn { for_type: "git2::Oid" },
            kind: e.into(),
        })?;
        Self::from_bytes(&buf).map_err(|e| ReadError {
            context: ErrorContext::BuiltIn { for_type: "git2::Oid" },
            kind: ReadErrorKind::Custom(e.to_string()),
        })
    }

    fn write_sync(&self, sink: &mut impl Write) -> Result<(), WriteError> {
        sink.write_all(self.as_bytes()).map_err(|e| WriteError {
            context: ErrorContext::BuiltIn { for_type: "git2::Oid" },
            kind: e.into(),
        })?;
        Ok(())
    }
}
