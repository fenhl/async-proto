use {
    std::{
        future::Future,
        io::prelude::*,
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

#[cfg_attr(docsrs, doc(cfg(feature = "serde_json")))]
impl Protocol for serde_json::Map<String, serde_json::Value> {
    fn read<'a, R: AsyncRead + Unpin + Send + 'a>(stream: &'a mut R) -> Pin<Box<dyn Future<Output = Result<Self, ReadError>> + Send + 'a>> {
        Box::pin(async move {
            let len = u64::read(stream).await?;
            let mut map = Self::with_capacity(len.try_into()?);
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

    fn read_sync(stream: &mut impl Read) -> Result<Self, ReadError> {
        let len = u64::read_sync(stream)?;
        let mut map = Self::with_capacity(len.try_into()?);
        for _ in 0..len {
            map.insert(String::read_sync(stream)?, serde_json::Value::read_sync(stream)?);
        }
        Ok(map)
    }

    fn write_sync(&self, sink: &mut impl Write) -> Result<(), WriteError> {
        u64::try_from(self.len())?.write_sync(sink)?;
        for (k, v) in self {
            k.write_sync(sink)?;
            v.write_sync(sink)?;
        }
        Ok(())
    }
}

#[derive(Protocol)]
#[async_proto(internal)]
enum NumberProxy {
    U64(u64),
    I64(i64),
    F64(f64),
}

impl TryFrom<NumberProxy> for serde_json::Number {
    type Error = ReadError;

    fn try_from(number: NumberProxy) -> Result<Self, ReadError> {
        match number {
            NumberProxy::U64(n) => Ok(Self::from(n)),
            NumberProxy::I64(n) => Ok(Self::from(n)),
            NumberProxy::F64(n) => Self::from_f64(n).ok_or(ReadError::FloatNotFinite),
        }
    }
}

impl<'a> From<&'a serde_json::Number> for NumberProxy {
    fn from(number: &serde_json::Number) -> Self {
        if let Some(value) = number.as_u64() {
            Self::U64(value)
        } else if let Some(value) = number.as_i64() {
            Self::I64(value)
        } else if let Some(value) = number.as_f64() {
            Self::F64(value)
        } else {
            unreachable!("serde_json::Number is neither u64 nor i64 nor f64")
        }
    }
}

impl_protocol_for! {
    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "serde_json")))))]
    enum serde_json::Value {
        Null,
        Bool(bool),
        Number(serde_json::Number),
        String(String),
        Array(Vec<serde_json::Value>),
        Object(serde_json::Map<String, serde_json::Value>),
    }

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "serde_json")))))]
    #[async_proto(via = NumberProxy)]
    type serde_json::Number;
}
