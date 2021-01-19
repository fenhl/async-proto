//! This is `async-proto`, a library crate facilitating simple binary network protocols with `async` support.
//!
//! The main feature is the [`Protocol`] trait, which allows reading a value of an implementing type from an async or sync stream, as well as writing one to an async or sync sink.
//!
//! `Protocol` can be derived for `enum`s and `struct`s if all fields implement `Protocol`.

#![deny(missing_docs, rust_2018_idioms, unused, unused_crate_dependencies, unused_import_braces, unused_lifetimes, unused_qualifications, warnings)]
#![forbid(unsafe_code)]

use {
    std::{
        future::Future,
        io::{
            self,
            prelude::*,
        },
        pin::Pin,
    },
    byteorder::{
        NetworkEndian,
        ReadBytesExt as _,
        WriteBytesExt as _,
    },
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
