//! This is `async-proto`, a library crate facilitating simple binary network protocols with `async` support.
//!
//! The main feature is the [`Protocol`] trait, which allows reading a value of an implementing type from an async or sync stream, as well as writing one to an async or sync sink.
//!
//! `Protocol` can be derived for `enum`s and `struct`s if all fields implement `Protocol`.
//!
//! # Features
//!
//! The following features can be enabled via Cargo:
//!
//! * `blocking`: Adds blocking `read_sync` and `write_sync` methods to the `Protocol` trait.
//! * `serde_json`: Adds a dependency on the [`serde_json`](https://docs.rs/serde_json) crate and implements `Protocol` for its `Value`, `Map`, and `Number` types.

#![deny(missing_docs, rust_2018_idioms, unused, unused_crate_dependencies, unused_import_braces, unused_lifetimes, unused_qualifications, warnings)]
#![forbid(unsafe_code)]

use {
    std::{
        fmt,
        future::Future,
        io,
        num::TryFromIntError,
        pin::Pin,
        string::FromUtf8Error,
        sync::Arc,
    },
    derive_more::From,
    tokio::io::{
        AsyncRead,
        AsyncWrite,
    },
};
#[cfg(feature = "blocking")] use std::io::prelude::*;
pub use async_proto_derive::Protocol;
#[doc(hidden)] pub use { // used in proc macro
    derive_more,
    tokio,
};

mod impls;

/// The error returned from the `read` and `read_sync` methods.
#[derive(Debug, From, Clone)]
#[allow(missing_docs)]
pub enum ReadError {
    /// Received a buffer with more than `usize::MAX` elements
    BufSize(TryFromIntError),
    /// An error variant you can use when manually implementing [`Protocol`]
    Custom(String),
    /// Used in the `serde_json` feature
    FloatNotFinite,
    Io(Arc<io::Error>),
    /// Attempted to read an empty type
    ReadNever,
    #[from(ignore)]
    UnknownVariant(u8),
    Utf8(FromUtf8Error),
}

impl From<io::Error> for ReadError {
    fn from(e: io::Error) -> ReadError {
        ReadError::Io(Arc::new(e))
    }
}

impl fmt::Display for ReadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ReadError::BufSize(e) => write!(f, "received a buffer with more than usize::MAX elements: {}", e),
            ReadError::Custom(msg) => msg.fmt(f),
            ReadError::FloatNotFinite => write!(f, "received an infinite or NaN JSON number"),
            ReadError::Io(e) => write!(f, "I/O error: {}", e),
            ReadError::ReadNever => write!(f, "attempted to read an empty type"),
            ReadError::UnknownVariant(n) => write!(f, "unknown enum variant: {}", n),
            ReadError::Utf8(e) => e.fmt(f),
        }
    }
}

/// The error returned from the `read` and `read_sync` methods.
#[derive(Debug, From, Clone)]
#[allow(missing_docs)]
pub enum WriteError {
    /// Tried to send a buffer with more than `u64::MAX` elements
    BufSize(TryFromIntError),
    /// An error variant you can use when manually implementing [`Protocol`]
    Custom(String),
    Io(Arc<io::Error>),
}

impl From<io::Error> for WriteError {
    fn from(e: io::Error) -> WriteError {
        WriteError::Io(Arc::new(e))
    }
}

impl fmt::Display for WriteError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WriteError::BufSize(e) => write!(f, "tried to send a buffer with more than u64::MAX elements: {}", e),
            WriteError::Custom(msg) => msg.fmt(f),
            WriteError::Io(e) => write!(f, "I/O error: {}", e),
        }
    }
}

/// This trait allows reading a value of an implementing type from an async or sync stream, as well as writing one to an async or sync sink.
pub trait Protocol: Sized {
    /// Reads a value of this type from an async stream.
    fn read<'a, R: AsyncRead + Unpin + Send + 'a>(stream: &'a mut R) -> Pin<Box<dyn Future<Output = Result<Self, ReadError>> + Send + 'a>>;
    /// Writes a value of this type to an async sink.
    fn write<'a, W: AsyncWrite + Unpin + Send + 'a>(&'a self, sink: &'a mut W) -> Pin<Box<dyn Future<Output = Result<(), WriteError>> + Send + 'a>>;
    #[cfg(feature = "blocking")]
    /// Reads a value of this type from a sync stream.
    fn read_sync(stream: &mut impl Read) -> Result<Self, ReadError>;
    #[cfg(feature = "blocking")]
    /// Writes a value of this type to a sync sink.
    fn write_sync(&self, sink: &mut impl Write) -> Result<(), WriteError>;
}
