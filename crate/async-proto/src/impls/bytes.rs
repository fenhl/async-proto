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
        Protocol,
        ReadError,
        WriteError,
    },
};

/// A [`Bytes`] is prefixed with the length as a [`u64`].
impl Protocol for Bytes {
    fn read<'a, R: AsyncRead + Unpin + Send + 'a>(stream: &'a mut R) -> Pin<Box<dyn Future<Output = Result<Self, ReadError>> + Send + 'a>> {
        Box::pin(async move {
            let len = u64::read(stream).await?;
            let mut buf = vec![0; len.try_into()?];
            stream.read_exact(&mut buf).await?;
            Ok(buf.into())
        })
    }

    fn write<'a, W: AsyncWrite + Unpin + Send + 'a>(&'a self, sink: &'a mut W) -> Pin<Box<dyn Future<Output = Result<(), WriteError>> + Send + 'a>> {
        Box::pin(async move {
            u64::try_from(self.len())?.write(sink).await?;
            sink.write_all(self).await?;
            Ok(())
        })
    }

    fn read_sync(stream: &mut impl Read) -> Result<Self, ReadError> {
        let len = u64::read_sync(stream)?;
        let mut buf = vec![0; len.try_into()?];
        stream.read_exact(&mut buf)?;
        Ok(buf.into())
    }

    fn write_sync(&self, sink: &mut impl Write) -> Result<(), WriteError> {
        u64::try_from(self.len())?.write_sync(sink)?;
        sink.write_all(self)?;
        Ok(())
    }
}
