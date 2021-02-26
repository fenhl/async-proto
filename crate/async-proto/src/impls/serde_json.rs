use {
    std::{
        convert::{
            TryFrom as _,
            TryInto as _,
        },
        fmt,
        future::Future,
        io,
        pin::Pin,
    },
    derive_more::From,
    tokio::io::{
        AsyncRead,
        AsyncWrite,
    },
    async_proto_derive::impl_protocol_for,
    crate::{
        Protocol,
        impls::MapReadError,
    },
};
#[cfg(feature = "blocking")] use std::io::prelude::*;

impl Protocol for serde_json::Map<String, serde_json::Value> {
    type ReadError = MapReadError<String, serde_json::Value>;

    fn read<'a, R: AsyncRead + Unpin + Send + 'a>(mut stream: R) -> Pin<Box<dyn Future<Output = Result<serde_json::Map<String, serde_json::Value>, MapReadError<String, serde_json::Value>>> + Send + 'a>> {
        Box::pin(async move {
            let len = u64::read(&mut stream).await.map_err(MapReadError::Io)?;
            let mut map = serde_json::Map::with_capacity(len.try_into().expect("tried to read map longer than usize::MAX"));
            for _ in 0..len {
                map.insert(String::read(&mut stream).await.map_err(|e| MapReadError::Key(Box::new(e)))?, serde_json::Value::read(&mut stream).await.map_err(|e| MapReadError::Value(Box::new(e)))?);
            }
            Ok(map)
        })
    }

    fn write<'a, W: AsyncWrite + Unpin + Send + 'a>(&'a self, mut sink: W) -> Pin<Box<dyn Future<Output = io::Result<()>> + Send + 'a>> {
        Box::pin(async move {
            u64::try_from(self.len()).expect("map was longer than u64::MAX").write(&mut sink).await?;
            for (k, v) in self {
                k.write(&mut sink).await?;
                v.write(&mut sink).await?;
            }
            Ok(())
        })
    }

    #[cfg(feature = "blocking")]
    fn read_sync<'a>(mut stream: impl Read + 'a) -> Result<serde_json::Map<String, serde_json::Value>, MapReadError<String, serde_json::Value>> {
        let len = u64::read_sync(&mut stream).map_err(MapReadError::Io)?;
        let mut map = serde_json::Map::with_capacity(len.try_into().expect("tried to read map longer than usize::MAX"));
        for _ in 0..len {
            map.insert(String::read_sync(&mut stream).map_err(|e| MapReadError::Key(Box::new(e)))?, serde_json::Value::read_sync(&mut stream).map_err(|e| MapReadError::Value(Box::new(e)))?);
        }
        Ok(map)
    }

    #[cfg(feature = "blocking")]
    fn write_sync<'a>(&self, mut sink: impl Write + 'a) -> io::Result<()> {
        u64::try_from(self.len()).expect("map was longer than u64::MAX").write_sync(&mut sink)?;
        for (k, v) in self {
            k.write_sync(&mut sink)?;
            v.write_sync(&mut sink)?;
        }
        Ok(())
    }
}

#[derive(Debug, From)]
pub enum NumberReadError {
    FloatNotFinite,
    Io(io::Error),
    Variant(u8),
}

impl fmt::Display for NumberReadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NumberReadError::FloatNotFinite => write!(f, "received an infinite or NaN f64 in a JSON number"),
            NumberReadError::Io(e) => e.fmt(f),
            NumberReadError::Variant(n) => write!(f, "unknown variant: {}", n),
        }
    }
}

impl Protocol for serde_json::Number {
    type ReadError = NumberReadError;

    fn read<'a, R: AsyncRead + Unpin + Send + 'a>(mut stream: R) -> Pin<Box<dyn Future<Output = Result<serde_json::Number, NumberReadError>> + Send + 'a>> {
        Box::pin(async move {
            Ok(match u8::read(&mut stream).await? {
                0 => serde_json::Number::from(u64::read(stream).await?),
                1 => serde_json::Number::from(i64::read(stream).await?),
                2 => serde_json::Number::from_f64(f64::read(stream).await?).ok_or(NumberReadError::FloatNotFinite)?,
                n => return Err(NumberReadError::Variant(n)),
            })
        })
    }

    fn write<'a, W: AsyncWrite + Unpin + Send + 'a>(&'a self, mut sink: W) -> Pin<Box<dyn Future<Output = io::Result<()>> + Send + 'a>> {
        Box::pin(async move {
            if let Some(value) = self.as_u64() {
                0u8.write(&mut sink).await?;
                value.write(sink).await
            } else if let Some(value) = self.as_i64() {
                1u8.write(&mut sink).await?;
                value.write(sink).await
            } else if let Some(value) = self.as_f64() {
                2u8.write(&mut sink).await?;
                value.write(sink).await
            } else {
                unreachable!("serde_json::Number is neither u64 nor i64 nor f64")
            }
        })
    }

    #[cfg(feature = "blocking")]
    fn read_sync<'a>(mut stream: impl Read + 'a) -> Result<serde_json::Number, NumberReadError> {
        Ok(match u8::read_sync(&mut stream)? {
            0 => serde_json::Number::from(u64::read_sync(stream)?),
            1 => serde_json::Number::from(i64::read_sync(stream)?),
            2 => serde_json::Number::from_f64(f64::read_sync(stream)?).ok_or(NumberReadError::FloatNotFinite)?,
            n => return Err(NumberReadError::Variant(n)),
        })
    }

    #[cfg(feature = "blocking")]
    fn write_sync<'a>(&self, mut sink: impl Write + 'a) -> io::Result<()> {
        if let Some(value) = self.as_u64() {
            0u8.write_sync(&mut sink)?;
            value.write_sync(sink)
        } else if let Some(value) = self.as_i64() {
            1u8.write_sync(&mut sink)?;
            value.write_sync(sink)
        } else if let Some(value) = self.as_f64() {
            2u8.write_sync(&mut sink)?;
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
