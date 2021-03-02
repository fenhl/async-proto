use {
    std::{
        convert::{
            TryFrom as _,
            TryInto as _,
        },
        future::Future,
        pin::Pin,
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

impl Protocol for serde_json::Map<String, serde_json::Value> {
    fn read<'a, R: AsyncRead + Unpin + Send + 'a>(stream: &'a mut R) -> Pin<Box<dyn Future<Output = Result<serde_json::Map<String, serde_json::Value>, ReadError>> + Send + 'a>> {
        Box::pin(async move {
            let len = u64::read(stream).await?;
            let mut map = serde_json::Map::with_capacity(len.try_into()?);
            for _ in 0..len {
                map.insert(String::read(stream).await?, serde_json::Value::read(stream).await?);
            }
            Ok(map)
        })
    }

    fn write<'a, W: AsyncWrite + Unpin + Send + 'a>(&'a self, sink: &'a mut W) -> Pin<Box<dyn Future<Output = Result<(), WriteError>> + Send + 'a>> {
        Box::pin(async move {
            u64::try_from(self.len())?.write(sink).await?;
            for (k, v) in self {
                k.write(sink).await?;
                v.write(sink).await?;
            }
            Ok(())
        })
    }

    #[cfg(feature = "read-sync")]
    fn read_sync(stream: &mut impl Read) -> Result<serde_json::Map<String, serde_json::Value>, ReadError> {
        let len = u64::read_sync(stream)?;
        let mut map = serde_json::Map::with_capacity(len.try_into()?);
        for _ in 0..len {
            map.insert(String::read_sync(stream)?, serde_json::Value::read_sync(stream)?);
        }
        Ok(map)
    }

    #[cfg(feature = "write-sync")]
    fn write_sync(&self, sink: &mut impl Write) -> Result<(), WriteError> {
        u64::try_from(self.len())?.write_sync(sink)?;
        for (k, v) in self {
            k.write_sync(sink)?;
            v.write_sync(sink)?;
        }
        Ok(())
    }
}

impl Protocol for serde_json::Number {
    fn read<'a, R: AsyncRead + Unpin + Send + 'a>(stream: &'a mut R) -> Pin<Box<dyn Future<Output = Result<serde_json::Number, ReadError>> + Send + 'a>> {
        Box::pin(async move {
            Ok(match u8::read(stream).await? {
                0 => serde_json::Number::from(u64::read(stream).await?),
                1 => serde_json::Number::from(i64::read(stream).await?),
                2 => serde_json::Number::from_f64(f64::read(stream).await?).ok_or(ReadError::FloatNotFinite)?,
                n => return Err(ReadError::UnknownVariant(n)),
            })
        })
    }

    fn write<'a, W: AsyncWrite + Unpin + Send + 'a>(&'a self, sink: &'a mut W) -> Pin<Box<dyn Future<Output = Result<(), WriteError>> + Send + 'a>> {
        Box::pin(async move {
            if let Some(value) = self.as_u64() {
                0u8.write(sink).await?;
                value.write(sink).await
            } else if let Some(value) = self.as_i64() {
                1u8.write(sink).await?;
                value.write(sink).await
            } else if let Some(value) = self.as_f64() {
                2u8.write(sink).await?;
                value.write(sink).await
            } else {
                unreachable!("serde_json::Number is neither u64 nor i64 nor f64")
            }
        })
    }

    #[cfg(feature = "read-sync")]
    fn read_sync(stream: &mut impl Read) -> Result<serde_json::Number, ReadError> {
        Ok(match u8::read_sync(stream)? {
            0 => serde_json::Number::from(u64::read_sync(stream)?),
            1 => serde_json::Number::from(i64::read_sync(stream)?),
            2 => serde_json::Number::from_f64(f64::read_sync(stream)?).ok_or(ReadError::FloatNotFinite)?,
            n => return Err(ReadError::UnknownVariant(n)),
        })
    }

    #[cfg(feature = "write-sync")]
    fn write_sync(&self, sink: &mut impl Write) -> Result<(), WriteError> {
        if let Some(value) = self.as_u64() {
            0u8.write_sync(sink)?;
            value.write_sync(sink)
        } else if let Some(value) = self.as_i64() {
            1u8.write_sync(sink)?;
            value.write_sync(sink)
        } else if let Some(value) = self.as_f64() {
            2u8.write_sync(sink)?;
            value.write_sync(sink)
        } else {
            unreachable!("serde_json::Number is neither u64 nor i64 nor f64")
        }
    }
}

impl_protocol_for! {
    enum serde_json::Value {
        Null,
        Bool(bool),
        Number(serde_json::Number),
        String(String),
        Array(Vec<serde_json::Value>),
        Object(serde_json::Map<String, serde_json::Value>),
    }
}
