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
//! This crate offers optional dependencies on the following crates to enable [`Protocol`] implementations for some of their types:
//!
//! * [`bytes`](https://docs.rs/bytes): [`Bytes`](https://docs.rs/bytes/latest/bytes/struct.Bytes.html)
//! * [`chrono`](https://docs.rs/chrono): [`NaiveDate`](https://docs.rs/chrono/latest/chrono/naive/struct.NaiveDate.html), [`DateTime`](https://docs.rs/chrono/latest/chrono/struct.DateTime.html), [`Utc`](https://docs.rs/chrono/latest/chrono/offset/struct.Utc.html), and [`FixedOffset`](https://docs.rs/chrono/latest/chrono/offset/struct.FixedOffset.html)
//! * [`chrono-tz`](https://docs.rs/chrono-tz): [`Tz`](https://docs.rs/chrono-tz/latest/chrono_tz/enum.Tz.html)
//! * [`either`](https://docs.rs/either): [`Either`](https://docs.rs/either/latest/either/enum.Either.html)
//! * [`noisy_float`](https://docs.rs/noisy_float): [`NoisyFloat`](https://docs.rs/noisy_float/latest/noisy_float/struct.NoisyFloat.html)
//! * [`semver`](https://docs.rs/semver): [`Version`](https://docs.rs/semver/latest/semver/struct.Version.html), [`Prerelease`](https://docs.rs/semver/latest/semver/struct.Prerelease.html), and [`BuildMetadata`](https://docs.rs/semver/latest/semver/struct.BuildMetadata.html)
//! * [`serde_json`](https://docs.rs/serde_json): [`Value`](https://docs.rs/serde_json/latest/serde_json/enum.Value.html), [`Map`](https://docs.rs/serde_json/latest/serde_json/struct.Map.html), and [`Number`](https://docs.rs/serde_json/latest/serde_json/struct.Number.html)
//! * [`uuid`](https://docs.rs/uuid): [`Uuid`](https://docs.rs/uuid/latest/uuid/struct.Uuid.html)
//!
//! Additionally, the following features can be enabled via Cargo:
//!
//! * `tokio-tungstenite`: Adds a dependency on the [`tokio-tungstenite`](https://docs.rs/tokio-tungstenite) crate and convenience methods for reading/writing [`Protocol`] types from/to its websockets.
//! * `warp`: Adds a dependency on the [`warp`](https://docs.rs/warp) crate and convenience methods for reading/writing [`Protocol`] types from/to its websockets.

use {
    std::{
        borrow::Cow,
        convert::Infallible,
        future::Future,
        io::{
            self,
            prelude::*,
        },
        num::TryFromIntError,
        pin::Pin,
        string::FromUtf8Error,
    },
    tokio::io::{
        AsyncRead,
        AsyncWrite,
    },
};
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
#[doc(hidden)] pub use tokio; // used in proc macro

mod impls;

/// The error returned from the [`read`](Protocol::read) and [`read_sync`](Protocol::read_sync) methods.
#[derive(Debug, thiserror::Error)]
#[allow(missing_docs)]
pub enum ReadError {
    /// Received a buffer with more than [`usize::MAX`] elements
    #[error("received a buffer with more than usize::MAX elements: {0}")]
    BufSize(#[from] TryFromIntError),
    /// An error variant you can use when manually implementing [`Protocol`]
    #[error("{0}")]
    Custom(String),
    /// The end of the stream was encountered before a complete value was read.
    ///
    /// Note that this error condition may also be represented as a [`ReadError::Io`] with [`kind`](io::Error::kind) [`UnexpectedEof`](io::ErrorKind::UnexpectedEof).
    #[error("reached end of stream")]
    EndOfStream,
    #[error("received an infinite or NaN number")]
    FloatNotFinite,
    /// Attempted to read an empty type
    #[error("attempted to read an empty type")]
    ReadNever,
    #[error("unknown enum variant: {0}")]
    UnknownVariant8(u8),
    #[error("unknown enum variant: {0}")]
    UnknownVariant16(u16),
    #[error("unknown enum variant: {0}")]
    UnknownVariant32(u32),
    #[error("unknown enum variant: {0}")]
    UnknownVariant64(u64),
    #[error("unknown enum variant: {0}")]
    UnknownVariant128(u128),
    #[error(transparent)] Io(#[from] io::Error),
    #[cfg(any(feature = "tokio-tungstenite", feature = "tungstenite"))]
    #[cfg_attr(docsrs, doc(cfg(any(feature = "tokio-tungstenite", feature = "tungstenite"))))]
    #[error(transparent)] Tungstenite(#[from] tungstenite::Error),
    #[error(transparent)] Utf8(#[from] FromUtf8Error),
    #[cfg(feature = "warp")]
    #[cfg_attr(docsrs, doc(cfg(feature = "warp")))]
    #[error(transparent)] Warp(#[from] warp::Error),
}

impl From<Infallible> for ReadError {
    fn from(never: Infallible) -> Self {
        match never {}
    }
}

impl From<String> for ReadError {
    fn from(s: String) -> Self {
        Self::Custom(s)
    }
}

impl<'a> From<&'a str> for ReadError {
    fn from(s: &str) -> Self {
        Self::Custom(s.to_owned())
    }
}

impl<'a> From<Cow<'a, str>> for ReadError {
    fn from(s: Cow<'a, str>) -> Self {
        Self::Custom(s.into_owned())
    }
}

impl From<ReadError> for io::Error {
    fn from(e: ReadError) -> Self {
        match e {
            ReadError::BufSize(e) => io::Error::new(io::ErrorKind::InvalidData, e),
            ReadError::Io(e) => e,
            #[cfg(any(feature = "tokio-tungstenite", feature = "tungstenite"))] ReadError::Tungstenite(e) => io::Error::new(io::ErrorKind::Other, e),
            ReadError::Utf8(e) => io::Error::new(io::ErrorKind::InvalidData, e),
            #[cfg(feature = "warp")] ReadError::Warp(e) => io::Error::new(io::ErrorKind::Other, e),
            ReadError::EndOfStream => io::Error::new(io::ErrorKind::UnexpectedEof, e),
            ReadError::FloatNotFinite |
            ReadError::UnknownVariant8(_) |
            ReadError::UnknownVariant16(_) |
            ReadError::UnknownVariant32(_) |
            ReadError::UnknownVariant64(_) |
            ReadError::UnknownVariant128(_) => io::Error::new(io::ErrorKind::InvalidData, e),
            ReadError::ReadNever => io::Error::new(io::ErrorKind::InvalidInput, e),
            ReadError::Custom(_) => io::Error::new(io::ErrorKind::Other, e),
        }
    }
}

/// The error returned from the [`write`](Protocol::write) and [`write_sync`](Protocol::write_sync) methods.
#[derive(Debug, thiserror::Error)]
#[allow(missing_docs)]
pub enum WriteError {
    /// Tried to send a buffer with more than [`u64::MAX`] elements
    #[error("tried to send a buffer with more than u64::MAX elements: {0}")]
    BufSize(#[from] TryFromIntError),
    /// An error variant you can use when manually implementing [`Protocol`]
    #[error("{0}")]
    Custom(String),
    #[error(transparent)] Io(#[from] io::Error),
    #[cfg(any(feature = "tokio-tungstenite", feature = "tungstenite"))]
    #[cfg_attr(docsrs, doc(cfg(any(feature = "tokio-tungstenite", feature = "tungstenite"))))]
    #[error(transparent)] Tungstenite(#[from] tungstenite::Error),
    #[cfg(feature = "warp")]
    #[cfg_attr(docsrs, doc(cfg(feature = "warp")))]
    #[error(transparent)] Warp(#[from] warp::Error),
}

impl From<Infallible> for WriteError {
    fn from(never: Infallible) -> Self {
        match never {}
    }
}

impl From<String> for WriteError {
    fn from(s: String) -> Self {
        Self::Custom(s)
    }
}

impl<'a> From<&'a str> for WriteError {
    fn from(s: &str) -> Self {
        Self::Custom(s.to_owned())
    }
}

impl<'a> From<Cow<'a, str>> for WriteError {
    fn from(s: Cow<'a, str>) -> Self {
        Self::Custom(s.into_owned())
    }
}

impl From<WriteError> for io::Error {
    fn from(e: WriteError) -> Self {
        match e {
            WriteError::BufSize(e) => io::Error::new(io::ErrorKind::InvalidData, e),
            WriteError::Io(e) => e,
            #[cfg(any(feature = "tokio-tungstenite", feature = "tungstenite"))] WriteError::Tungstenite(e) => io::Error::new(io::ErrorKind::Other, e),
            #[cfg(feature = "warp")] WriteError::Warp(e) => io::Error::new(io::ErrorKind::Other, e),
            WriteError::Custom(_) => io::Error::new(io::ErrorKind::Other, e),
        }
    }
}

/// This trait allows reading a value of an implementing type from an async or sync stream, as well as writing one to an async or sync sink.
pub trait Protocol: Sized {
    /// Reads a value of this type from an async stream.
    ///
    /// # Cancellation safety
    ///
    /// Implementations of this method are generally not cancellation safe.
    fn read<'a, R: AsyncRead + Unpin + Send + 'a>(stream: &'a mut R) -> Pin<Box<dyn Future<Output = Result<Self, ReadError>> + Send + 'a>>;
    /// Writes a value of this type to an async sink.
    ///
    /// # Cancellation safety
    ///
    /// Implementations of this method are generally not cancellation safe.
    fn write<'a, W: AsyncWrite + Unpin + Send + 'a>(&'a self, sink: &'a mut W) -> Pin<Box<dyn Future<Output = Result<(), WriteError>> + Send + 'a>>;
    /// Reads a value of this type from a sync stream.
    fn read_sync(stream: &mut impl Read) -> Result<Self, ReadError>;
    /// Writes a value of this type to a sync sink.
    fn write_sync(&self, sink: &mut impl Write) -> Result<(), WriteError>;

    /// Takes ownership of an async stream, reads a value of this type from it, then returns it along with the stream.
    ///
    /// This can be used to get around drop glue issues that might arise with `read`.
    fn read_owned<R: AsyncRead + Unpin + Send + 'static>(mut stream: R) -> Pin<Box<dyn Future<Output = Result<(R, Self), ReadError>> + Send>> {
        Box::pin(async move {
            let value = Self::read(&mut stream).await?;
            Ok((stream, value))
        })
    }

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
    /// Reads a value of this type from a `tokio-tungstenite` websocket.
    ///
    /// # Cancellation safety
    ///
    /// The default implementation of this method is cancellation safe.
    fn read_ws<'a, R: Stream<Item = Result<tungstenite::Message, tungstenite::Error>> + Unpin + Send + 'a>(stream: &'a mut R) -> Pin<Box<dyn Future<Output = Result<Self, ReadError>> + Send + 'a>> {
        Box::pin(async move {
            let packet = stream.try_next().await?.ok_or(ReadError::EndOfStream)?;
            Self::read_sync(&mut &*packet.into_data())
        })
    }

    #[cfg(feature = "tokio-tungstenite")]
    #[cfg_attr(docsrs, doc(cfg(feature = "tokio-tungstenite")))]
    /// Writes a value of this type to a `tokio-tungstenite` websocket.
    ///
    /// # Cancellation safety
    ///
    /// The default implementation of this method is not cancellation safe.
    fn write_ws<'a, W: Sink<tungstenite::Message, Error = tungstenite::Error> + Unpin + Send + 'a>(&'a self, sink: &'a mut W) -> Pin<Box<dyn Future<Output = Result<(), WriteError>> + Send + 'a>>
    where Self: Sync {
        Box::pin(async move {
            let mut buf = Vec::default();
            self.write(&mut buf).await?;
            sink.send(tungstenite::Message::binary(buf)).await?;
            Ok(())
        })
    }

    #[cfg(feature = "warp")]
    #[cfg_attr(docsrs, doc(cfg(feature = "warp")))]
    /// Reads a value of this type from a [`warp`] websocket.
    ///
    /// # Cancellation safety
    ///
    /// The default implementation of this method is cancellation safe.
    fn read_warp<'a, R: Stream<Item = Result<warp::filters::ws::Message, warp::Error>> + Unpin + Send + 'a>(stream: &'a mut R) -> Pin<Box<dyn Future<Output = Result<Self, ReadError>> + Send + 'a>> {
        Box::pin(async move {
            loop {
                let packet = stream.try_next().await?.ok_or(ReadError::EndOfStream)?;
                if packet.is_ping() || packet.is_pong() { continue }
                if packet.is_close() { return Err(ReadError::EndOfStream) }
                break Self::read_sync(&mut packet.as_bytes())
            }
        })
    }

    #[cfg(feature = "tokio-tungstenite")]
    #[cfg_attr(docsrs, doc(cfg(feature = "tokio-tungstenite")))]
    /// Takes ownership of an async websocket stream, reads a value of this type from it, then returns it along with the stream.
    ///
    /// This can be used to get around drop glue issues that might arise with `read_ws`.
    fn read_ws_owned<R: Stream<Item = Result<tungstenite::Message, tungstenite::Error>> + Unpin + Send + 'static>(mut stream: R) -> Pin<Box<dyn Future<Output = Result<(R, Self), ReadError>> + Send>> {
        Box::pin(async move {
            let value = Self::read_ws(&mut stream).await?;
            Ok((stream, value))
        })
    }

    #[cfg(feature = "warp")]
    #[cfg_attr(docsrs, doc(cfg(feature = "warp")))]
    /// Writes a value of this type to a [`warp`] websocket.
    ///
    /// # Cancellation safety
    ///
    /// The default implementation of this method is not cancellation safe.
    fn write_warp<'a, W: Sink<warp::filters::ws::Message, Error = warp::Error> + Unpin + Send + 'a>(&'a self, sink: &'a mut W) -> Pin<Box<dyn Future<Output = Result<(), WriteError>> + Send + 'a>>
    where Self: Sync {
        Box::pin(async move {
            let mut buf = Vec::default();
            self.write(&mut buf).await?;
            sink.send(warp::filters::ws::Message::binary(buf)).await?;
            Ok(())
        })
    }

    #[cfg(feature = "tungstenite")]
    #[cfg_attr(docsrs, doc(cfg(feature = "tungstenite")))]
    /// Reads a value of this type from a [`tungstenite`](tungstenite) websocket.
    fn read_ws_sync(websocket: &mut tungstenite::WebSocket<impl Read + Write>) -> Result<Self, ReadError> {
        let packet = websocket.read_message()?;
        Self::read_sync(&mut &*packet.into_data())
    }

    #[cfg(feature = "tungstenite")]
    #[cfg_attr(docsrs, doc(cfg(feature = "tungstenite")))]
    /// Writes a value of this type to a [`tungstenite`](tungstenite) websocket.
    fn write_ws_sync(&self, websocket: &mut tungstenite::WebSocket<impl Read + Write>) -> Result<(), WriteError> {
        let mut buf = Vec::default();
        self.write_sync(&mut buf)?;
        websocket.write_message(tungstenite::Message::binary(buf))?;
        Ok(())
    }
}
