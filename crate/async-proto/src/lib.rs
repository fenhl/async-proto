//! This is `async-proto`, a library crate facilitating simple binary network protocols with `async` support.
//!
//! The main feature is the [`Protocol`] trait, which allows reading a value of an implementing type from an async or sync stream, as well as writing one to an async or sync sink.
//!
//! `Protocol` can be derived for `enum`s and `struct`s if all fields implement `Protocol`.

#![deny(missing_docs, rust_2018_idioms, unused, unused_crate_dependencies, unused_import_braces, unused_lifetimes, unused_qualifications, warnings)]
#![forbid(unsafe_code)]

use {
    std::{
        convert::{
            TryFrom as _,
            TryInto as _,
        },
        future::Future,
        io::{
            self,
            prelude::*,
        },
        pin::Pin,
        string::FromUtf8Error,
    },
    byteorder::{
        NetworkEndian,
        ReadBytesExt as _,
        WriteBytesExt as _,
    },
    derive_more::From,
    tokio::io::{
        AsyncRead,
        AsyncReadExt as _,
        AsyncWrite,
        AsyncWriteExt as _,
    },
};
pub use async_proto_derive::Protocol;
#[doc(hidden)] pub use { // used in proc macro
    async_trait,
    derive_more,
    tokio,
};

/// This trait allows reading a value of an implementing type from an async or sync stream, as well as writing one to an async or sync sink.
pub trait Protocol: Sized {
    /// The error returned from the `read` and `read_sync` methods.
    ///
    /// It can be an [`io::Error`] or an error representing malformed data.
    type ReadError;

    /// Reads a value of this type from an async stream.
    fn read<'a, R: AsyncRead + Unpin + Send + 'a>(stream: R) -> Pin<Box<dyn Future<Output = Result<Self, Self::ReadError>> + Send + 'a>>;
    /// Writes a value of this type to an async sink.
    fn write<'a, W: AsyncWrite + Unpin + Send + 'a>(&'a self, sink: W) -> Pin<Box<dyn Future<Output = io::Result<()>> + Send + 'a>>;
    /// Reads a value of this type from a sync stream.
    fn read_sync<'a>(stream: impl Read + 'a) -> Result<Self, Self::ReadError>;
    /// Writes a value of this type to a sync sink.
    fn write_sync<'a>(&self, sink: impl Write + 'a) -> io::Result<()>;
}

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

            fn read_sync<'a>(mut stream: impl Read + 'a) -> io::Result<$ty> {
                stream.$read$(::<$endian>)?()
            }

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
#[allow(missing_docs)]
pub enum BoolReadError {
    InvalidValue(u8),
    #[from]
    Io(io::Error),
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

    fn read_sync<'a>(stream: impl Read + 'a) -> Result<bool, BoolReadError> {
        Ok(match u8::read_sync(stream)? {
            0 => false,
            1 => true,
            n => return Err(BoolReadError::InvalidValue(n)),
        })
    }

    fn write_sync<'a>(&self, sink: impl Write + 'a) -> io::Result<()> {
        if *self { 1u8 } else { 0 }.write_sync(sink)
    }
}

#[derive(Debug)]
#[allow(missing_docs)]
pub enum OptionReadError<T: Protocol> {
    Variant(BoolReadError),
    Content(T::ReadError),
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

    fn read_sync<'a>(mut stream: impl Read + 'a) -> Result<Option<T>, OptionReadError<T>> {
        Ok(if bool::read_sync(&mut stream).map_err(OptionReadError::Variant)? {
            Some(T::read_sync(stream).map_err(OptionReadError::Content)?)
        } else {
            None
        })
    }

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
#[allow(missing_docs)]
pub enum VecReadError<T: Protocol> {
    Elt(T::ReadError),
    Io(io::Error),
}

impl<T: Protocol + Send + Sync> Protocol for Vec<T> {
    type ReadError = VecReadError<T>;

    fn read<'a, R: AsyncRead + Unpin + Send + 'a>(mut stream: R) -> Pin<Box<dyn Future<Output = Result<Vec<T>, VecReadError<T>>> + Send + 'a>> {
        Box::pin(async move {
            let len = u32::read(&mut stream).await.map_err(VecReadError::Io)?;
            let mut buf = Vec::with_capacity(len.try_into().expect("tried to read vector longer than usize::MAX"));
            for _ in 0..len {
                buf.push(T::read(&mut stream).await.map_err(VecReadError::Elt)?);
            }
            Ok(buf)
        })
    }

    fn write<'a, W: AsyncWrite + Unpin + Send + 'a>(&'a self, mut sink: W) -> Pin<Box<dyn Future<Output = io::Result<()>> + Send + 'a>> {
        Box::pin(async move {
            u32::try_from(self.len()).expect("vector was longer than u32::MAX").write(&mut sink).await?;
            for elt in self {
                elt.write(&mut sink).await?;
            }
            Ok(())
        })
    }

    fn read_sync<'a>(mut stream: impl Read + 'a) -> Result<Vec<T>, VecReadError<T>> {
        let len = u32::read_sync(&mut stream).map_err(VecReadError::Io)?;
        let mut buf = Vec::with_capacity(len.try_into().expect("tried to read vector longer than usize::MAX"));
        for _ in 0..len {
            buf.push(T::read_sync(&mut stream).map_err(VecReadError::Elt)?);
        }
        Ok(buf)
    }

    fn write_sync<'a>(&self, mut sink: impl Write + 'a) -> io::Result<()> {
        u32::try_from(self.len()).expect("vector was longer than u32::MAX").write_sync(&mut sink)?;
        for elt in self {
            elt.write_sync(&mut sink)?;
        }
        Ok(())
    }
}

#[derive(Debug, From)]
#[allow(missing_docs)]
pub enum StringReadError {
    Utf8(FromUtf8Error),
    Vec(VecReadError<u8>),
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

    fn read_sync<'a>(stream: impl Read + 'a) -> Result<String, StringReadError> {
        let buf = Vec::read_sync(stream)?;
        Ok(String::from_utf8(buf)?)
    }

    fn write_sync<'a>(&self, mut sink: impl Write + 'a) -> io::Result<()> {
        u32::try_from(self.len()).expect("string was longer than u32::MAX bytes").write_sync(&mut sink)?;
        sink.write(self.as_bytes())?;
        Ok(())
    }
}
