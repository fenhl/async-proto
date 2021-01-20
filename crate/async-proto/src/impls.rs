//! [`Protocol`] implementations for primitive and [`std`] types.

#![allow(missing_docs)]

use {
    std::{
        collections::{
            BTreeMap,
            HashMap,
        },
        convert::{
            TryFrom as _,
            TryInto as _,
        },
        fmt,
        future::Future,
        hash::Hash,
        io,
        pin::Pin,
        string::FromUtf8Error,
    },

    derive_more::From,
    tokio::io::{
        AsyncRead,
        AsyncReadExt as _,
        AsyncWrite,
        AsyncWriteExt as _,
    },
    crate::Protocol,
};
#[cfg(feature = "blocking")] use {
    std::io::prelude::*,
    byteorder::{
        NetworkEndian,
        ReadBytesExt as _,
        WriteBytesExt as _,
    },
};

macro_rules! impl_protocol_primitive {
    ($ty:ty, $read:ident, $write:ident$(, $endian:ty)?) => {
        impl Protocol for $ty {
            type ReadError = io::Error;

            fn read<'a, R: AsyncRead + Unpin + Send + 'a>(mut stream: R) -> Pin<Box<dyn Future<Output = io::Result<$ty>> + Send + 'a>> {
                Box::pin(async move {
                    stream.$read().await
                })
            }

            fn write<'a, W: AsyncWrite + Unpin + Send + 'a>(&'a self, mut sink: W) -> Pin<Box<dyn Future<Output = io::Result<()>> + Send + 'a>> {
                Box::pin(async move {
                    sink.$write(*self).await
                })
            }

            #[cfg(feature = "blocking")]
            fn read_sync<'a>(mut stream: impl Read + 'a) -> io::Result<$ty> {
                stream.$read$(::<$endian>)?()
            }

            #[cfg(feature = "blocking")]
            fn write_sync<'a>(&self, mut sink: impl Write + 'a) -> io::Result<()> {
                sink.$write$(::<$endian>)?(*self)
            }
        }
    };
}

impl_protocol_primitive!(u8, read_u8, write_u8);
impl_protocol_primitive!(i8, read_i8, write_i8);
impl_protocol_primitive!(u16, read_u16, write_u16, NetworkEndian);
impl_protocol_primitive!(i16, read_i16, write_i16, NetworkEndian);
impl_protocol_primitive!(u32, read_u32, write_u32, NetworkEndian);
impl_protocol_primitive!(i32, read_i32, write_i32, NetworkEndian);
impl_protocol_primitive!(u64, read_u64, write_u64, NetworkEndian);
impl_protocol_primitive!(i64, read_i64, write_i64, NetworkEndian);
impl_protocol_primitive!(u128, read_u128, write_u128, NetworkEndian);
impl_protocol_primitive!(i128, read_i128, write_i128, NetworkEndian);

#[derive(Debug, From)]
pub enum BoolReadError {
    InvalidValue(u8),
    #[from]
    Io(io::Error),
}

impl fmt::Display for BoolReadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BoolReadError::InvalidValue(n) => write!(f, "invalid Boolean value: {} (expected 0 or 1)", n),
            BoolReadError::Io(e) => write!(f, "I/O error: {}", e),
        }
    }
}

impl Protocol for bool {
    type ReadError = BoolReadError;

    fn read<'a, R: AsyncRead + Unpin + Send + 'a>(stream: R) -> Pin<Box<dyn Future<Output = Result<bool, BoolReadError>> + Send + 'a>> {
        Box::pin(async move {
            Ok(match u8::read(stream).await? {
                0 => false,
                1 => true,
                n => return Err(BoolReadError::InvalidValue(n)),
            })
        })
    }

    fn write<'a, W: AsyncWrite + Unpin + Send + 'a>(&'a self, sink: W) -> Pin<Box<dyn Future<Output = io::Result<()>> + Send + 'a>> {
        Box::pin(async move {
            if *self { 1u8 } else { 0 }.write(sink).await
        })
    }

    #[cfg(feature = "blocking")]
    fn read_sync<'a>(stream: impl Read + 'a) -> Result<bool, BoolReadError> {
        Ok(match u8::read_sync(stream)? {
            0 => false,
            1 => true,
            n => return Err(BoolReadError::InvalidValue(n)),
        })
    }

    #[cfg(feature = "blocking")]
    fn write_sync<'a>(&self, sink: impl Write + 'a) -> io::Result<()> {
        if *self { 1u8 } else { 0 }.write_sync(sink)
    }
}

#[derive(Debug)]
pub enum OptionReadError<T: Protocol> {
    Variant(BoolReadError),
    Content(T::ReadError),
}

impl<T: Protocol> fmt::Display for OptionReadError<T>
where T::ReadError: fmt::Display {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OptionReadError::Variant(e) => e.fmt(f),
            OptionReadError::Content(e) => e.fmt(f),
        }
    }
}

impl<T: Protocol + Sync> Protocol for Option<T> {
    type ReadError = OptionReadError<T>;

    fn read<'a, R: AsyncRead + Unpin + Send + 'a>(mut stream: R) -> Pin<Box<dyn Future<Output = Result<Option<T>, OptionReadError<T>>> + Send + 'a>> {
        Box::pin(async move {
            Ok(if bool::read(&mut stream).await.map_err(OptionReadError::Variant)? {
                Some(T::read(stream).await.map_err(OptionReadError::Content)?)
            } else {
                None
            })
        })
    }

    fn write<'a, W: AsyncWrite + Unpin + Send + 'a>(&'a self, mut sink: W) -> Pin<Box<dyn Future<Output = io::Result<()>> + Send + 'a>> {
        Box::pin(async move {
            if let Some(value) = self {
                true.write(&mut sink).await?;
                value.write(sink).await?;
            } else {
                false.write(sink).await?;
            }
            Ok(())
        })
    }

    #[cfg(feature = "blocking")]
    fn read_sync<'a>(mut stream: impl Read + 'a) -> Result<Option<T>, OptionReadError<T>> {
        Ok(if bool::read_sync(&mut stream).map_err(OptionReadError::Variant)? {
            Some(T::read_sync(stream).map_err(OptionReadError::Content)?)
        } else {
            None
        })
    }

    #[cfg(feature = "blocking")]
    fn write_sync<'a>(&self, mut sink: impl Write + 'a) -> io::Result<()> {
        if let Some(value) = self {
            true.write_sync(&mut sink)?;
            value.write_sync(sink)?;
        } else {
            false.write_sync(sink)?;
        }
        Ok(())
    }
}

#[derive(Debug)]
pub enum VecReadError<T: Protocol> {
    Elt(T::ReadError),
    Io(io::Error),
}

impl<T: Protocol> fmt::Display for VecReadError<T>
where T::ReadError: fmt::Display {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VecReadError::Elt(e) => e.fmt(f),
            VecReadError::Io(e) => write!(f, "I/O error: {}", e),
        }
    }
}

impl<T: Protocol + Send + Sync> Protocol for Vec<T> {
    type ReadError = VecReadError<T>;

    fn read<'a, R: AsyncRead + Unpin + Send + 'a>(mut stream: R) -> Pin<Box<dyn Future<Output = Result<Vec<T>, VecReadError<T>>> + Send + 'a>> {
        Box::pin(async move {
            let len = u64::read(&mut stream).await.map_err(VecReadError::Io)?;
            let mut buf = Vec::with_capacity(len.try_into().expect("tried to read vector longer than usize::MAX"));
            for _ in 0..len {
                buf.push(T::read(&mut stream).await.map_err(VecReadError::Elt)?);
            }
            Ok(buf)
        })
    }

    fn write<'a, W: AsyncWrite + Unpin + Send + 'a>(&'a self, mut sink: W) -> Pin<Box<dyn Future<Output = io::Result<()>> + Send + 'a>> {
        Box::pin(async move {
            u64::try_from(self.len()).expect("vector was longer than u64::MAX").write(&mut sink).await?;
            for elt in self {
                elt.write(&mut sink).await?;
            }
            Ok(())
        })
    }

    #[cfg(feature = "blocking")]
    fn read_sync<'a>(mut stream: impl Read + 'a) -> Result<Vec<T>, VecReadError<T>> {
        let len = u64::read_sync(&mut stream).map_err(VecReadError::Io)?;
        let mut buf = Vec::with_capacity(len.try_into().expect("tried to read vector longer than usize::MAX"));
        for _ in 0..len {
            buf.push(T::read_sync(&mut stream).map_err(VecReadError::Elt)?);
        }
        Ok(buf)
    }

    #[cfg(feature = "blocking")]
    fn write_sync<'a>(&self, mut sink: impl Write + 'a) -> io::Result<()> {
        u64::try_from(self.len()).expect("vector was longer than u32::MAX").write_sync(&mut sink)?;
        for elt in self {
            elt.write_sync(&mut sink)?;
        }
        Ok(())
    }
}

#[derive(Debug, From)]
pub enum StringReadError {
    Utf8(FromUtf8Error),
    Vec(VecReadError<u8>),
}

impl fmt::Display for StringReadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StringReadError::Utf8(e) => e.fmt(f),
            StringReadError::Vec(e) => e.fmt(f),
        }
    }
}

impl Protocol for String {
    type ReadError = StringReadError;

    fn read<'a, R: AsyncRead + Unpin + Send + 'a>(stream: R) -> Pin<Box<dyn Future<Output = Result<String, StringReadError>> + Send + 'a>> {
        Box::pin(async move {
            let buf = Vec::read(stream).await?;
            Ok(String::from_utf8(buf)?)
        })
    }

    fn write<'a, W: AsyncWrite + Unpin + Send + 'a>(&'a self, mut sink: W) -> Pin<Box<dyn Future<Output = io::Result<()>> + Send + 'a>> {
        Box::pin(async move {
            u32::try_from(self.len()).expect("string was longer than u32::MAX bytes").write(&mut sink).await?;
            sink.write(self.as_bytes()).await?;
            Ok(())
        })
    }

    #[cfg(feature = "blocking")]
    fn read_sync<'a>(stream: impl Read + 'a) -> Result<String, StringReadError> {
        let buf = Vec::read_sync(stream)?;
        Ok(String::from_utf8(buf)?)
    }

    #[cfg(feature = "blocking")]
    fn write_sync<'a>(&self, mut sink: impl Write + 'a) -> io::Result<()> {
        u32::try_from(self.len()).expect("string was longer than u32::MAX bytes").write_sync(&mut sink)?;
        sink.write(self.as_bytes())?;
        Ok(())
    }
}

#[derive(Debug)]
pub enum MapReadError<K: Protocol, V: Protocol> {
    Io(io::Error),
    Key(K::ReadError),
    Value(V::ReadError),
}

impl<K: Protocol, V: Protocol> fmt::Display for MapReadError<K, V>
where K::ReadError: fmt::Display, V::ReadError: fmt::Display {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MapReadError::Io(e) => write!(f, "I/O error: {}", e),
            MapReadError::Key(e) => e.fmt(f),
            MapReadError::Value(e) => e.fmt(f),
        }
    }
}

impl<K: Protocol + Ord + Send + Sync + 'static, V: Protocol + Send + Sync + 'static> Protocol for BTreeMap<K, V>
where K::ReadError: Send, V::ReadError: Send {
    type ReadError = MapReadError<K, V>;

    fn read<'a, R: AsyncRead + Unpin + Send + 'a>(mut stream: R) -> Pin<Box<dyn Future<Output = Result<BTreeMap<K, V>, MapReadError<K, V>>> + Send + 'a>> {
        Box::pin(async move {
            let len = u64::read(&mut stream).await.map_err(MapReadError::Io)?;
            let mut map = BTreeMap::default();
            for _ in 0..len {
                map.insert(K::read(&mut stream).await.map_err(MapReadError::Key)?, V::read(&mut stream).await.map_err(MapReadError::Value)?);
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
    fn read_sync<'a>(mut stream: impl Read + 'a) -> Result<BTreeMap<K, V>, MapReadError<K, V>> {
        let len = u64::read_sync(&mut stream).map_err(MapReadError::Io)?;
        let mut map = BTreeMap::default();
        for _ in 0..len {
            map.insert(K::read_sync(&mut stream).map_err(MapReadError::Key)?, V::read_sync(&mut stream).map_err(MapReadError::Value)?);
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

impl<K: Protocol + Eq + Hash + Send + Sync, V: Protocol + Send + Sync> Protocol for HashMap<K, V>
where K::ReadError: Send, V::ReadError: Send {
    type ReadError = MapReadError<K, V>;

    fn read<'a, R: AsyncRead + Unpin + Send + 'a>(mut stream: R) -> Pin<Box<dyn Future<Output = Result<HashMap<K, V>, MapReadError<K, V>>> + Send + 'a>> {
        Box::pin(async move {
            let len = u64::read(&mut stream).await.map_err(MapReadError::Io)?;
            let mut map = HashMap::with_capacity(len.try_into().expect("tried to read map longer than usize::MAX"));
            for _ in 0..len {
                map.insert(K::read(&mut stream).await.map_err(MapReadError::Key)?, V::read(&mut stream).await.map_err(MapReadError::Value)?);
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
    fn read_sync<'a>(mut stream: impl Read + 'a) -> Result<HashMap<K, V>, MapReadError<K, V>> {
        let len = u64::read_sync(&mut stream).map_err(MapReadError::Io)?;
        let mut map = HashMap::with_capacity(len.try_into().expect("tried to read map longer than usize::MAX"));
        for _ in 0..len {
            map.insert(K::read_sync(&mut stream).map_err(MapReadError::Key)?, V::read_sync(&mut stream).map_err(MapReadError::Value)?);
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

impl Protocol for std::time::Duration {
    type ReadError = io::Error;

    fn read<'a, R: AsyncRead + Unpin + Send + 'a>(mut stream: R) -> Pin<Box<dyn Future<Output = io::Result<std::time::Duration>> + Send + 'a>> {
        Box::pin(async move {
            Ok(std::time::Duration::new(u64::read(&mut stream).await?, u32::read(&mut stream).await?))
        })
    }

    fn write<'a, W: AsyncWrite + Unpin + Send + 'a>(&'a self, mut sink: W) -> Pin<Box<dyn Future<Output = io::Result<()>> + Send + 'a>> {
        Box::pin(async move {
            self.as_secs().write(&mut sink).await?;
            self.subsec_nanos().write(sink).await?;
            Ok(())
        })
    }

    #[cfg(feature = "blocking")]
    fn read_sync<'a>(mut stream: impl Read + 'a) -> io::Result<std::time::Duration> {
        Ok(std::time::Duration::new(u64::read_sync(&mut stream)?, u32::read_sync(&mut stream)?))
    }

    #[cfg(feature = "blocking")]
    fn write_sync<'a>(&self, mut sink: impl Write + 'a) -> io::Result<()> {
        self.as_secs().write_sync(&mut sink)?;
        self.subsec_nanos().write_sync(sink)?;
        Ok(())
    }
}
