use {
    std::{
        io::prelude::*,
        pin::Pin,
    },
    fallible_collections::FallibleVec as _,
    tokio::io::{
        AsyncRead,
        AsyncReadExt as _,
        AsyncWrite,
        AsyncWriteExt as _,
    },
    async_proto_derive::impl_protocol_for,
    crate::{
        ErrorContext,
        LengthPrefixed,
        Protocol,
        ReadError,
        ReadErrorKind,
        WriteError,
        WriteErrorKind,
    },
};

/// An [`nbt::Blob`] is Gzip-compressed and prefixed with the length of the blob after compression as a [`u64`].
#[cfg_attr(docsrs, doc(cfg(feature = "hematite-nbt")))]
impl Protocol for nbt::Blob {
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

#[cfg_attr(docsrs, doc(cfg(feature = "hematite-nbt")))]
impl LengthPrefixed for nbt::Blob {
    fn read_length_prefixed<'a, R: AsyncRead + Unpin + Send + 'a>(stream: &'a mut R, max_len: u64) -> Pin<Box<dyn Future<Output = Result<Self, ReadError>> + Send + 'a>> {
        Box::pin(async move {
            let len = super::read_len(stream, max_len, || ErrorContext::BuiltIn { for_type: "nbt::Blob" }).await?;
            let mut buf = Vec::default();
            buf.try_resize(len, 0).map_err(|e| ReadError {
                context: ErrorContext::BuiltIn { for_type: "nbt::Blob" },
                kind: e.into(),
            })?;
            stream.read_exact(&mut buf).await.map_err(|e| ReadError {
                context: ErrorContext::BuiltIn { for_type: "nbt::Blob" },
                kind: e.into(),
            })?;
            Self::from_gzip_reader(&mut &*buf).map_err(|e| ReadError {
                context: ErrorContext::BuiltIn { for_type: "nbt::Blob" },
                kind: ReadErrorKind::Custom(e.to_string()),
            })
        })
    }

    fn write_length_prefixed<'a, W: AsyncWrite + Unpin + Send + 'a>(&'a self, sink: &'a mut W, max_len: u64) -> Pin<Box<dyn Future<Output = Result<(), WriteError>> + Send + 'a>> {
        Box::pin(async move {
            let mut buf = Vec::default();
            self.to_gzip_writer(&mut buf).map_err(|e| WriteError {
                context: ErrorContext::BuiltIn { for_type: "nbt::Blob" },
                kind: WriteErrorKind::Custom(e.to_string()),
            })?;
            super::write_len(sink, buf.len(), max_len, || ErrorContext::BuiltIn { for_type: "nbt::Blob" }).await?;
            sink.write_all(&buf).await.map_err(|e| WriteError {
                context: ErrorContext::BuiltIn { for_type: "nbt::Blob" },
                kind: e.into(),
            })?;
            Ok(())
        })
    }

    fn read_length_prefixed_sync(stream: &mut impl Read, max_len: u64) -> Result<Self, ReadError> {
        let len = super::read_len_sync(stream, max_len, || ErrorContext::BuiltIn { for_type: "nbt::Blob" })?;
        let mut buf = Vec::default();
        buf.try_resize(len, 0).map_err(|e| ReadError {
            context: ErrorContext::BuiltIn { for_type: "nbt::Blob" },
            kind: e.into(),
        })?;
        stream.read_exact(&mut buf).map_err(|e| ReadError {
            context: ErrorContext::BuiltIn { for_type: "nbt::Blob" },
            kind: e.into(),
        })?;
        Self::from_gzip_reader(&mut &*buf).map_err(|e| ReadError {
            context: ErrorContext::BuiltIn { for_type: "nbt::Blob" },
            kind: ReadErrorKind::Custom(e.to_string()),
        })
    }

    fn write_length_prefixed_sync(&self, sink: &mut impl Write, max_len: u64) -> Result<(), WriteError> {
        let mut buf = Vec::default();
        self.to_gzip_writer(&mut buf).map_err(|e| WriteError {
            context: ErrorContext::BuiltIn { for_type: "nbt::Blob" },
            kind: WriteErrorKind::Custom(e.to_string()),
        })?;
        super::write_len_sync(sink, buf.len(), max_len, || ErrorContext::BuiltIn { for_type: "nbt::Blob" })?;
        sink.write_all(&buf).map_err(|e| WriteError {
            context: ErrorContext::BuiltIn { for_type: "nbt::Blob" },
            kind: e.into(),
        })?;
        Ok(())
    }
}

#[derive(Protocol)]
#[async_proto(internal)]
struct ValueProxy(nbt::Blob);

impl TryFrom<ValueProxy> for nbt::Value {
    type Error = ReadErrorKind;

    fn try_from(ValueProxy(blob): ValueProxy) -> Result<Self, Self::Error> {
        Ok(blob.get("").ok_or_else(|| ReadErrorKind::Custom(format!("NBT blob missing empty-string entry")))?.clone())
    }
}

impl TryFrom<nbt::Value> for ValueProxy {
    type Error = WriteErrorKind;

    fn try_from(value: nbt::Value) -> Result<Self, Self::Error> {
        let mut blob = nbt::Blob::default();
        blob.insert("", value).map_err(|e| WriteErrorKind::Custom(e.to_string()))?;
        Ok(Self(blob))
    }
}

impl_protocol_for! {
    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "hematite-nbt")))))]
    #[async_proto(attr(doc = "An [`nbt::Value`] is represented as an [`nbt::Blob`] with no name and a single entry with no name containing the original value."))]
    #[async_proto(via = ValueProxy, clone)]
    type nbt::Value;
}
