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
        Protocol,
        ReadError,
        ReadErrorKind,
        WriteError,
    },
};

/// A [`BitVec`] is prefixed with the length (in bits) as a [`u64`].
#[cfg_attr(docsrs, doc(cfg(feature = "bitvec")))]
impl Protocol for BitVec<u8, Lsb0> {
    fn read<'a, R: AsyncRead + Unpin + Send + 'a>(stream: &'a mut R) -> Pin<Box<dyn Future<Output = Result<Self, ReadError>> + Send + 'a>> {
        Box::pin(async move {
            let bit_len = usize::try_from(u64::read(stream).await?).map_err(|e| ReadError {
                context: ErrorContext::BuiltIn { for_type: "bitvec::vec::BitVec<u8, Lsb0>" },
                kind: e.into(),
            })?;
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

    fn write<'a, W: AsyncWrite + Unpin + Send + 'a>(&'a self, sink: &'a mut W) -> Pin<Box<dyn Future<Output = Result<(), WriteError>> + Send + 'a>> {
        Box::pin(async move {
            u64::try_from(self.len()).map_err(|e| WriteError {
                context: ErrorContext::BuiltIn { for_type: "bitvec::vec::BitVec<u8, Lsb0>" },
                kind: e.into(),
            })?.write(sink).await?;
            sink.write_all(self.as_raw_slice()).await.map_err(|e| WriteError {
                context: ErrorContext::BuiltIn { for_type: "bitvec::vec::BitVec<u8, Lsb0>" },
                kind: e.into(),
            })?;
            Ok(())
        })
    }

    fn read_sync(stream: &mut impl Read) -> Result<Self, ReadError> {
        let bit_len = usize::try_from(u64::read_sync(stream)?).map_err(|e| ReadError {
            context: ErrorContext::BuiltIn { for_type: "bitvec::vec::BitVec<u8, Lsb0>" },
            kind: e.into(),
        })?;
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

    fn write_sync(&self, sink: &mut impl Write) -> Result<(), WriteError> {
        u64::try_from(self.len()).map_err(|e| WriteError {
            context: ErrorContext::BuiltIn { for_type: "bitvec::vec::BitVec<u8, Lsb0>" },
            kind: e.into(),
        })?.write_sync(sink)?;
        sink.write_all(self.as_raw_slice()).map_err(|e| WriteError {
            context: ErrorContext::BuiltIn { for_type: "bitvec::vec::BitVec<u8, Lsb0>" },
            kind: e.into(),
        })?;
        Ok(())
    }
}
