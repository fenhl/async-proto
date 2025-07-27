use {
    std::{
        future::Future,
        io::prelude::*,
        pin::Pin,
    },
    bitvec::{
        order::Lsb0,
        vec::BitVec,
    },
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
        ReadErrorKind,
        WriteError,
    },
};

/// A [`BitVec`] is prefixed with the length in bits as a [`u64`].
#[cfg_attr(docsrs, doc(cfg(feature = "bitvec")))]
impl Protocol for BitVec<u8, Lsb0> {
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

/// A [`BitVec`] is prefixed with the length in bits.
#[cfg_attr(docsrs, doc(cfg(feature = "bitvec")))]
impl LengthPrefixed for BitVec<u8, Lsb0> {
    fn read_length_prefixed<'a, R: AsyncRead + Unpin + Send + 'a>(stream: &'a mut R, max_len: u64) -> Pin<Box<dyn Future<Output = Result<Self, ReadError>> + Send + 'a>> {
        Box::pin(async move {
            let bit_len = super::read_len(stream, max_len, || ErrorContext::BuiltIn { for_type: "bitvec::vec::BitVec<u8, Lsb0>" }).await?;
            let byte_len = bit_len.div_ceil(8);
            let mut buf = Vec::default();
            buf.try_resize(byte_len, 0).map_err(|e| ReadError {
                context: ErrorContext::BuiltIn { for_type: "bitvec::vec::BitVec<u8, Lsb0>" },
                kind: e.into(),
            })?;
            stream.read_exact(&mut buf).await.map_err(|e| ReadError {
                context: ErrorContext::BuiltIn { for_type: "bitvec::vec::BitVec<u8, Lsb0>" },
                kind: e.into(),
            })?;
            let mut this = Self::try_from_vec(buf).map_err(|_| ReadError {
                context: ErrorContext::BuiltIn { for_type: "bitvec::vec::BitVec<u8, Lsb0>" },
                kind: ReadErrorKind::Custom(format!("too long to view as a bit-slice")),
            })?;
            this.truncate(bit_len);
            Ok(this)
        })
    }

    fn write_length_prefixed<'a, W: AsyncWrite + Unpin + Send + 'a>(&'a self, sink: &'a mut W, max_len: u64) -> Pin<Box<dyn Future<Output = Result<(), WriteError>> + Send + 'a>> {
        Box::pin(async move {
            super::write_len(sink, self.len(), max_len, || ErrorContext::BuiltIn { for_type: "bitvec::vec::BitVec<u8, Lsb0>" }).await?;
            sink.write_all(self.as_raw_slice()).await.map_err(|e| WriteError {
                context: ErrorContext::BuiltIn { for_type: "bitvec::vec::BitVec<u8, Lsb0>" },
                kind: e.into(),
            })?;
            Ok(())
        })
    }

    fn read_length_prefixed_sync(stream: &mut impl Read, max_len: u64) -> Result<Self, ReadError> {
        let bit_len = super::read_len_sync(stream, max_len, || ErrorContext::BuiltIn { for_type: "bitvec::vec::BitVec<u8, Lsb0>" })?;
        let byte_len = bit_len.div_ceil(8);
        let mut buf = Vec::default();
        buf.try_resize(byte_len, 0).map_err(|e| ReadError {
            context: ErrorContext::BuiltIn { for_type: "bitvec::vec::BitVec<u8, Lsb0>" },
            kind: e.into(),
        })?;
        stream.read_exact(&mut buf).map_err(|e| ReadError {
            context: ErrorContext::BuiltIn { for_type: "bitvec::vec::BitVec<u8, Lsb0>" },
            kind: e.into(),
        })?;
        let mut this = Self::try_from_vec(buf).map_err(|_| ReadError {
            context: ErrorContext::BuiltIn { for_type: "bitvec::vec::BitVec<u8, Lsb0>" },
            kind: ReadErrorKind::Custom(format!("too long to view as a bit-slice")),
        })?;
        this.truncate(bit_len);
        Ok(this)
    }

    fn write_length_prefixed_sync(&self, sink: &mut impl Write, max_len: u64) -> Result<(), WriteError> {
        super::write_len_sync(sink, self.len(), max_len, || ErrorContext::BuiltIn { for_type: "bitvec::vec::BitVec<u8, Lsb0>" })?;
        sink.write_all(self.as_raw_slice()).map_err(|e| WriteError {
            context: ErrorContext::BuiltIn { for_type: "bitvec::vec::BitVec<u8, Lsb0>" },
            kind: e.into(),
        })?;
        Ok(())
    }
}
