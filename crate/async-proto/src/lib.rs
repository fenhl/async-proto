#![deny(missing_docs, rust_2018_idioms, unused, unused_crate_dependencies, unused_import_braces, unused_lifetimes, unused_qualifications, warnings)]
#![forbid(unsafe_code)]

#![cfg_attr(docsrs, feature(doc_cfg))]

//! This is `async-proto`, a library crate facilitating simple binary network protocols with `async` support.
//!
//! The main feature is the [`Protocol`] trait, which allows reading a value of an implementing type from an async or sync stream, as well as writing one to an async or sync sink.
//!
//! [`Protocol`] can be derived for `enum`s and `struct`s if all fields implement [`Protocol`].
//!
//! # Features
//!
//! The following features can be enabled via Cargo:
//!
//! * `blocking`: Shorthand for enabling both `read-sync` and `write-sync`.
//! * `chrono-tz`: Adds a dependency on the [`chrono-tz`](https://docs.rs/chrono-tz) crate and implements [`Protocol`] for its [`Tz`](https://docs.rs/chrono-tz/latest/chrono_tz/enum.Tz.html) type.
//! * `noisy_float`: Adds a dependency on the [`noisy_float`](https://docs.rs/noisy_float) crate and implements [`Protocol`] for its [`NoisyFloat`](https://docs.rs/noisy_float/latest/noisy_float/struct.NoisyFloat.html) type.
//! * `read-sync`: Adds a blocking `read_sync` method to the [`Protocol`] trait.
//! * `serde_json`: Adds a dependency on the [`serde_json`](https://docs.rs/serde_json) crate and implements [`Protocol`] for its [`Value`](https://docs.rs/serde_json/latest/serde_json/enum.Value.html), [`Map`](https://docs.rs/serde_json/latest/serde_json/struct.Map.html), and [`Number`](https://docs.rs/serde_json/latest/serde_json/struct.Number.html) types.
//! * `tokio-tungstenite`: Adds a dependency on the [`tokio-tungstenite`](https://docs.rs/tokio-tungstenite) crate and convenience methods for reading/writing [`Protocol`] types from/to its websockets.
//! * `warp`: Adds a dependency on the [`warp`](https://docs.rs/warp) crate and convenience methods for reading/writing [`Protocol`] types from/to its websockets.
//! * `write-sync`: Adds a blocking `write_sync` method to the [`Protocol`] trait.

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
#[cfg(any(feature = "read-sync", feature = "write-sync"))] use std::io::prelude::*;
#[cfg(any(feature = "tokio-tungstenite", feature = "warp"))] use futures::{
    Sink,
    SinkExt as _,
    stream::{
        Stream,
        TryStreamExt as _,
    },
};
pub use async_proto_derive::{
    Protocol,
    bitflags,
};
#[doc(hidden)] pub use { // used in proc macro
    derive_more,
    tokio,
};

mod impls;

#[cfg_attr(not(feature = "read-sync"), doc = "The error returned from the [`read`](Protocol::read) method.")]
#[cfg_attr(feature = "read-sync", doc = "The error returned from the [`read`](Protocol::read) and [`read_sync`](Protocol::read_sync) methods.")]
#[derive(Debug, From, Clone)]
#[allow(missing_docs)]
pub enum ReadError {
    /// Received a buffer with more than [`usize::MAX`] elements
    BufSize(TryFromIntError),
    /// An error variant you can use when manually implementing [`Protocol`]
    Custom(String),
    /// The end of the stream was encountered before a complete value was read.
    ///
    /// Note that this error condition may also be represented as a [`ReadError::Io`] with [`kind`](io::Error::kind) [`UnexpectedEof`](io::ErrorKind::UnexpectedEof).
    EndOfStream,
    #[cfg(feature = "serde_json")]
    #[cfg_attr(docsrs, doc(cfg(feature = "serde_json")))]
    FloatNotFinite,
    Io(Arc<io::Error>),
    /// Attempted to read an empty type
    ReadNever,
    #[cfg(feature = "tokio-tungstenite")]
    #[cfg_attr(docsrs, doc(cfg(feature = "tokio-tungstenite")))]
    Tungstenite(Arc<dep_tokio_tungstenite::tungstenite::Error>),
    #[from(ignore)]
    UnknownVariant8(u8),
    #[from(ignore)]
    UnknownVariant16(u16),
    #[from(ignore)]
    UnknownVariant32(u32),
    #[from(ignore)]
    UnknownVariant64(u64),
    Utf8(FromUtf8Error),
    #[cfg(feature = "warp")]
    #[cfg_attr(docsrs, doc(cfg(feature = "warp")))]
    Warp(Arc<dep_warp::Error>),
}

impl From<io::Error> for ReadError {
    fn from(e: io::Error) -> ReadError {
        ReadError::Io(Arc::new(e))
    }
}

#[cfg(feature = "tokio-tungstenite")]
#[cfg_attr(docsrs, doc(cfg(feature = "tokio-tungstenite")))]
impl From<dep_tokio_tungstenite::tungstenite::Error> for ReadError {
    fn from(e: dep_tokio_tungstenite::tungstenite::Error) -> ReadError {
        ReadError::Tungstenite(Arc::new(e))
    }
}

#[cfg(feature = "warp")]
#[cfg_attr(docsrs, doc(cfg(feature = "warp")))]
impl From<dep_warp::Error> for ReadError {
    fn from(e: dep_warp::Error) -> ReadError {
        ReadError::Warp(Arc::new(e))
    }
}

impl fmt::Display for ReadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ReadError::BufSize(e) => write!(f, "received a buffer with more than usize::MAX elements: {}", e),
            ReadError::Custom(msg) => msg.fmt(f),
            ReadError::EndOfStream => write!(f, "reached end of stream"),
            #[cfg(feature = "serde_json")]
            ReadError::FloatNotFinite => write!(f, "received an infinite or NaN JSON number"),
            ReadError::Io(e) => write!(f, "I/O error: {}", e),
            ReadError::ReadNever => write!(f, "attempted to read an empty type"),
            #[cfg(feature = "tokio-tungstenite")]
            ReadError::Tungstenite(e) => write!(f, "tungstenite error: {}", e),
            ReadError::UnknownVariant8(n) => write!(f, "unknown enum variant: {}", n),
            ReadError::UnknownVariant16(n) => write!(f, "unknown enum variant: {}", n),
            ReadError::UnknownVariant32(n) => write!(f, "unknown enum variant: {}", n),
            ReadError::UnknownVariant64(n) => write!(f, "unknown enum variant: {}", n),
            ReadError::Utf8(e) => e.fmt(f),
            #[cfg(feature = "warp")]
            ReadError::Warp(e) => write!(f, "warp error: {}", e),
        }
    }
}

impl std::error::Error for ReadError {} //TODO use thiserror for better sources?

#[cfg_attr(not(feature = "write-sync"), doc = "The error returned from the [`write`](Protocol::write) method.")]
#[cfg_attr(feature = "write-sync", doc = "The error returned from the [`write`](Protocol::write) and [`write_sync`](Protocol::write_sync) methods.")]
#[derive(Debug, From, Clone)]
#[allow(missing_docs)]
pub enum WriteError {
    /// Tried to send a buffer with more than [`u64::MAX`] elements
    BufSize(TryFromIntError),
    /// An error variant you can use when manually implementing [`Protocol`]
    Custom(String),
    Io(Arc<io::Error>),
    #[cfg(feature = "tokio-tungstenite")]
    #[cfg_attr(docsrs, doc(cfg(feature = "tokio-tungstenite")))]
    Tungstenite(Arc<dep_tokio_tungstenite::tungstenite::Error>),
    #[cfg(feature = "warp")]
    #[cfg_attr(docsrs, doc(cfg(feature = "warp")))]
    Warp(Arc<dep_warp::Error>),
}

impl From<io::Error> for WriteError {
    fn from(e: io::Error) -> WriteError {
        WriteError::Io(Arc::new(e))
    }
}

#[cfg(feature = "tokio-tungstenite")]
#[cfg_attr(docsrs, doc(cfg(feature = "tokio-tungstenite")))]
impl From<dep_tokio_tungstenite::tungstenite::Error> for WriteError {
    fn from(e: dep_tokio_tungstenite::tungstenite::Error) -> WriteError {
        WriteError::Tungstenite(Arc::new(e))
    }
}

#[cfg(feature = "warp")]
#[cfg_attr(docsrs, doc(cfg(feature = "warp")))]
impl From<dep_warp::Error> for WriteError {
    fn from(e: dep_warp::Error) -> WriteError {
        WriteError::Warp(Arc::new(e))
    }
}

impl fmt::Display for WriteError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WriteError::BufSize(e) => write!(f, "tried to send a buffer with more than u64::MAX elements: {}", e),
            WriteError::Custom(msg) => msg.fmt(f),
            WriteError::Io(e) => write!(f, "I/O error: {}", e),
            #[cfg(feature = "tokio-tungstenite")]
            WriteError::Tungstenite(e) => write!(f, "tungstenite error: {}", e),
            #[cfg(feature = "warp")]
            WriteError::Warp(e) => write!(f, "warp error: {}", e),
        }
    }
}

impl std::error::Error for WriteError {} //TODO use thiserror for better sources?

/// This trait allows reading a value of an implementing type from an async or sync stream, as well as writing one to an async or sync sink.
pub trait Protocol: Sized {
    /// Reads a value of this type from an async stream.
    fn read<'a, R: AsyncRead + Unpin + Send + 'a>(stream: &'a mut R) -> Pin<Box<dyn Future<Output = Result<Self, ReadError>> + Send + 'a>>;
    /// Writes a value of this type to an async sink.
    fn write<'a, W: AsyncWrite + Unpin + Send + 'a>(&'a self, sink: &'a mut W) -> Pin<Box<dyn Future<Output = Result<(), WriteError>> + Send + 'a>>;
    #[cfg(feature = "read-sync")]
    #[cfg_attr(docsrs, doc(cfg(feature = "read-sync")))]
    /// Reads a value of this type from a sync stream.
    fn read_sync(stream: &mut impl Read) -> Result<Self, ReadError>;
    #[cfg(feature = "write-sync")]
    #[cfg_attr(docsrs, doc(cfg(feature = "write-sync")))]
    /// Writes a value of this type to a sync sink.
    fn write_sync(&self, sink: &mut impl Write) -> Result<(), WriteError>;

    #[cfg(feature = "read-sync")]
    #[cfg_attr(docsrs, doc(cfg(feature = "read-sync")))]
    /// Attempts to read a value of this type from a prefix in a buffer and a suffix in a sync stream.
    ///
    /// If [`io::ErrorKind::WouldBlock`] is encountered, `Ok(None)` is returned and the portion read successfully is appended to `buf`. Otherwise, the prefix representing the returned value is removed from `buf`.
    ///
    /// Callers, not implementations, should ensure that `stream` is non-blocking if desired.
    ///
    /// # Example
    ///
    /// ```
    /// use {
    ///     std::net::TcpStream,
    ///     async_proto::Protocol,
    /// };
    ///
    /// struct Client {
    ///     tcp_stream: TcpStream,
    ///     buf: Vec<u8>,
    /// }
    ///
    /// impl Client {
    ///     fn new(tcp_stream: TcpStream) -> Self {
    ///         Self {
    ///             tcp_stream,
    ///             buf: Vec::default(),
    ///         }
    ///     }
    ///
    ///     fn try_read<T: Protocol>(&mut self) -> Result<Option<T>, async_proto::ReadError> {
    ///         self.tcp_stream.set_nonblocking(true)?;
    ///         T::try_read(&mut self.tcp_stream, &mut self.buf)
    ///     }
    ///
    ///     fn write<T: Protocol>(&mut self, msg: &T) -> Result<(), async_proto::WriteError> {
    ///         self.tcp_stream.set_nonblocking(false)?;
    ///         msg.write_sync(&mut self.tcp_stream)
    ///     }
    /// }
    /// ```
    fn try_read(stream: &mut impl Read, buf: &mut Vec<u8>) -> Result<Option<Self>, ReadError> {
        let mut temp_buf = vec![0; 8];
        loop {
            let mut slice = &mut &**buf;
            match Self::read_sync(&mut slice) {
                Ok(value) => {
                    let value_len = slice.len();
                    drop(slice);
                    buf.drain(..buf.len() - value_len);
                    return Ok(Some(value))
                }
                Err(ReadError::Io(e)) if e.kind() == io::ErrorKind::UnexpectedEof => {}
                Err(e) => return Err(e),
            }
            match stream.read(&mut temp_buf) {
                Ok(0) => return Err(ReadError::EndOfStream),
                Ok(n) => buf.extend_from_slice(&temp_buf[..n]),
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => return Ok(None),
                Err(e) => return Err(e.into()),
            }
        }
    }

    #[cfg(feature = "tokio-tungstenite")]
    #[cfg_attr(docsrs, doc(cfg(feature = "tokio-tungstenite")))]
    /// Reads a value of this type from a [`tokio-tungstenite`](dep_tokio_tungstenite) websocket.
    fn read_ws<'a, R: Stream<Item = Result<dep_tokio_tungstenite::tungstenite::Message, dep_tokio_tungstenite::tungstenite::Error>> + Unpin + Send + 'a>(stream: &'a mut R) -> Pin<Box<dyn Future<Output = Result<Self, ReadError>> + Send + 'a>> {
        Box::pin(async move {
            let packet = stream.try_next().await?.ok_or(ReadError::EndOfStream)?;
            Self::read(&mut &*packet.into_data()).await
        })
    }

    #[cfg(feature = "tokio-tungstenite")]
    #[cfg_attr(docsrs, doc(cfg(feature = "tokio-tungstenite")))]
    /// Writes a value of this type to a [`tokio-tungstenite`](dep_tokio_tungstenite) websocket.
    fn write_ws<'a, W: Sink<dep_tokio_tungstenite::tungstenite::Message, Error = dep_tokio_tungstenite::tungstenite::Error> + Unpin + Send + 'a>(&'a self, sink: &'a mut W) -> Pin<Box<dyn Future<Output = Result<(), WriteError>> + Send + 'a>>
    where Self: Sync {
        Box::pin(async move {
            let mut buf = Vec::default();
            self.write(&mut buf).await?;
            sink.send(dep_tokio_tungstenite::tungstenite::Message::binary(buf)).await?;
            Ok(())
        })
    }

    #[cfg(feature = "warp")]
    #[cfg_attr(docsrs, doc(cfg(feature = "warp")))]
    /// Reads a value of this type from a [`warp`](dep_warp) websocket.
    fn read_warp<'a, R: Stream<Item = Result<dep_warp::filters::ws::Message, dep_warp::Error>> + Unpin + Send + 'a>(stream: &'a mut R) -> Pin<Box<dyn Future<Output = Result<Self, ReadError>> + Send + 'a>> {
        Box::pin(async move {
            let packet = stream.try_next().await?.ok_or(ReadError::EndOfStream)?;
            Self::read(&mut packet.as_bytes()).await
        })
    }

    #[cfg(feature = "warp")]
    #[cfg_attr(docsrs, doc(cfg(feature = "warp")))]
    /// Writes a value of this type to a [`warp`](dep_warp) websocket.
    fn write_warp<'a, W: Sink<dep_warp::filters::ws::Message, Error = dep_warp::Error> + Unpin + Send + 'a>(&'a self, sink: &'a mut W) -> Pin<Box<dyn Future<Output = Result<(), WriteError>> + Send + 'a>>
    where Self: Sync {
        Box::pin(async move {
            let mut buf = Vec::default();
            self.write(&mut buf).await?;
            sink.send(dep_warp::filters::ws::Message::binary(buf)).await?;
            Ok(())
        })
    }
}
