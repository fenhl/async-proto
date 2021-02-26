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
        future::Future,
        io,
        pin::Pin,
    },
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

pub mod impls;

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
    #[cfg(feature = "blocking")]
    /// Reads a value of this type from a sync stream.
    fn read_sync<'a>(stream: impl Read + 'a) -> Result<Self, Self::ReadError>;
    #[cfg(feature = "blocking")]
    /// Writes a value of this type to a sync sink.
    fn write_sync<'a>(&self, sink: impl Write + 'a) -> io::Result<()>;
}
