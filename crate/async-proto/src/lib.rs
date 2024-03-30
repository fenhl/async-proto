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
//! * [`enumset`](https://docs.rs/enumset): [`EnumSet`](https://docs.rs/either/latest/either/enum.Either.html)
//! * [`git2`](https://docs.rs/git2): [`Oid`](https://docs.rs/git2/latest/git2/struct.Oid.html)
//! * [`noisy_float`](https://docs.rs/noisy_float): [`NoisyFloat`](https://docs.rs/noisy_float/latest/noisy_float/struct.NoisyFloat.html)
//! * [`semver`](https://docs.rs/semver): [`Version`](https://docs.rs/semver/latest/semver/struct.Version.html), [`Prerelease`](https://docs.rs/semver/latest/semver/struct.Prerelease.html), and [`BuildMetadata`](https://docs.rs/semver/latest/semver/struct.BuildMetadata.html)
//! * [`serde_json`](https://docs.rs/serde_json): [`Value`](https://docs.rs/serde_json/latest/serde_json/enum.Value.html), [`Map`](https://docs.rs/serde_json/latest/serde_json/struct.Map.html), and [`Number`](https://docs.rs/serde_json/latest/serde_json/struct.Number.html)
//! * [`uuid`](https://docs.rs/uuid): [`Uuid`](https://docs.rs/uuid/latest/uuid/struct.Uuid.html)
//!
//! Additionally, the following features can be enabled via Cargo:
//!
//! * `tokio-tungstenite`: Adds a dependency on the [`tokio-tungstenite`](https://docs.rs/tokio-tungstenite) crate and convenience methods for reading/writing [`Protocol`] types from/to its websockets.
//! * `tungstenite`: Adds a dependency on the [`tungstenite`](https://docs.rs/tungstenite) crate and convenience methods for synchronously reading/writing [`Protocol`] types from/to its websockets.
//! * `warp`: Adds a dependency on the [`warp`](https://docs.rs/warp) crate and convenience methods for reading/writing [`Protocol`] types from/to its websockets.

use {
    std::{
        future::Future,
        io::{
            self,
            prelude::*,
        },
        pin::Pin,
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
#[cfg(feature = "tokio-tungstenite")] use futures::stream::StreamExt as _;
pub use {
    async_proto_derive::{
        Protocol,
        bitflags,
    },
    crate::error::*,
};
#[doc(hidden)] pub use tokio; // used in proc macro

mod error;
mod impls;

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
    ///     std::{
    ///         io,
    ///         net::TcpStream,
    ///     },
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
    ///     fn try_read<T: Protocol>(&mut self) -> io::Result<Option<T>> {
    ///         self.tcp_stream.set_nonblocking(true)?;
    ///         Ok(T::try_read(&mut self.tcp_stream, &mut self.buf)?)
    ///     }
    ///
    ///     fn write<T: Protocol>(&mut self, msg: &T) -> io::Result<()> {
    ///         self.tcp_stream.set_nonblocking(false)?;
    ///         msg.write_sync(&mut self.tcp_stream)?;
    ///         Ok(())
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
                    buf.drain(..buf.len() - value_len);
                    return Ok(Some(value))
                }
                Err(ReadError { kind: ReadErrorKind::Io(e), .. }) if e.kind() == io::ErrorKind::UnexpectedEof => {}
                Err(e) => return Err(e),
            }
            match stream.read(&mut temp_buf) {
                Ok(0) => return Err(ReadError {
                    context: ErrorContext::DefaultImpl,
                    kind: ReadErrorKind::EndOfStream,
                }),
                Ok(n) => buf.extend_from_slice(&temp_buf[..n]),
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => return Ok(None),
                Err(e) => return Err(ReadError {
                    context: ErrorContext::DefaultImpl,
                    kind: e.into(),
                }),
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
            let packet = stream.try_next().await.map_err(|e| ReadError {
                context: ErrorContext::DefaultImpl,
                kind: e.into(),
            })?.ok_or_else(|| ReadError {
                context: ErrorContext::DefaultImpl,
                kind: ReadErrorKind::EndOfStream,
            })?;
            if !packet.is_binary() {
                return Err(ReadError {
                    context: ErrorContext::DefaultImpl,
                    kind: ReadErrorKind::MessageKind(packet),
                })
            }
            Self::read_sync(&mut &*packet.into_data()).map_err(|ReadError { context, kind }| ReadError {
                context: ErrorContext::WebSocket {
                    source: Box::new(context),
                },
                kind,
            })
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
            self.write_sync(&mut buf).map_err(|WriteError { context, kind }| WriteError {
                context: ErrorContext::WebSocket {
                    source: Box::new(context),
                },
                kind,
            })?;
            sink.send(tungstenite::Message::binary(buf)).await.map_err(|e| WriteError {
                context: ErrorContext::DefaultImpl,
                kind: e.into(),
            })?;
            Ok(())
        })
    }

    #[cfg(feature = "tungstenite")]
    #[cfg_attr(docsrs, doc(cfg(feature = "tungstenite")))]
    /// Reads a value of this type from a [`tungstenite`] websocket.
    fn read_ws_sync(websocket: &mut tungstenite::WebSocket<impl Read + Write>) -> Result<Self, ReadError> {
        let packet = websocket.read().map_err(|e| ReadError {
            context: ErrorContext::DefaultImpl,
            kind: e.into(),
        })?;
        if !packet.is_binary() {
            return Err(ReadError {
                context: ErrorContext::DefaultImpl,
                kind: ReadErrorKind::MessageKind(packet),
            })
        }
        Self::read_sync(&mut &*packet.into_data()).map_err(|ReadError { context, kind }| ReadError {
            context: ErrorContext::WebSocket {
                source: Box::new(context),
            },
            kind,
        })
    }

    #[cfg(feature = "tungstenite")]
    #[cfg_attr(docsrs, doc(cfg(feature = "tungstenite")))]
    /// Writes a value of this type to a [`tungstenite`] websocket.
    fn write_ws_sync(&self, websocket: &mut tungstenite::WebSocket<impl Read + Write>) -> Result<(), WriteError> {
        let mut buf = Vec::default();
        self.write_sync(&mut buf).map_err(|WriteError { context, kind }| WriteError {
            context: ErrorContext::WebSocket {
                source: Box::new(context),
            },
            kind,
        })?;
        websocket.send(tungstenite::Message::binary(buf)).map_err(|e| WriteError {
            context: ErrorContext::DefaultImpl,
            kind: e.into(),
        })?;
        websocket.flush().map_err(|e| WriteError {
            context: ErrorContext::DefaultImpl,
            kind: e.into(),
        })?;
        Ok(())
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
    /// Reads a value of this type from a [`warp`] websocket.
    ///
    /// # Cancellation safety
    ///
    /// The default implementation of this method is cancellation safe.
    fn read_warp<'a, R: Stream<Item = Result<warp::filters::ws::Message, warp::Error>> + Unpin + Send + 'a>(stream: &'a mut R) -> Pin<Box<dyn Future<Output = Result<Self, ReadError>> + Send + 'a>> {
        Box::pin(async move {
            loop {
                let packet = stream.try_next().await.map_err(|e| ReadError {
                    context: ErrorContext::DefaultImpl,
                    kind: e.into(),
                })?.ok_or_else(|| ReadError {
                    context: ErrorContext::DefaultImpl,
                    kind: ReadErrorKind::EndOfStream,
                })?;
                if packet.is_ping() || packet.is_pong() { continue }
                if packet.is_close() {
                    return Err(ReadError {
                        context: ErrorContext::DefaultImpl,
                        kind: ReadErrorKind::EndOfStream,
                    })
                }
                break Self::read_sync(&mut packet.as_bytes())
            }
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
            self.write_sync(&mut buf).map_err(|WriteError { context, kind }| WriteError {
                context: ErrorContext::WebSocket {
                    source: Box::new(context),
                },
                kind,
            })?;
            sink.send(warp::filters::ws::Message::binary(buf)).await.map_err(|e| WriteError {
                context: ErrorContext::DefaultImpl,
                kind: e.into(),
            })?;
            Ok(())
        })
    }
}

/// Establishes a WebSocket connection to the given URL and returns a typed sink/stream pair.
///
/// Useful for WebSocket connections where the message type per direction is always the same.
#[cfg(feature = "tokio-tungstenite")]
#[cfg_attr(docsrs, doc(cfg(feature = "tokio-tungstenite")))]
pub async fn websocket<R: Protocol, W: Protocol>(request: impl tungstenite::client::IntoClientRequest + Unpin) -> tungstenite::Result<(impl Sink<W, Error = WriteError>, impl Stream<Item = Result<R, ReadError>>)> {
    let (sock, _) = tokio_tungstenite::connect_async(request).await?;
    let (sink, stream) = sock.split();
    Ok((
        sink.sink_map_err(|e| WriteError {
            context: ErrorContext::WebSocketSink,
            kind: e.into(),
        }).with::<W, _, _, WriteError>(|msg| async move {
            let mut buf = Vec::default();
            msg.write_sync(&mut buf).map_err(|WriteError { context, kind }| WriteError {
                context: ErrorContext::WebSocket {
                    source: Box::new(context),
                },
                kind,
            })?;
            Ok(tungstenite::Message::binary(buf))
        }),
        stream.map_err(|e| ReadError {
            context: ErrorContext::WebSocketStream,
            kind: e.into(),
        }).and_then(|packet| async move {
            if !packet.is_binary() {
                return Err(ReadError {
                    context: ErrorContext::WebSocketStream,
                    kind: ReadErrorKind::MessageKind(packet),
                })
            }
            R::read_sync(&mut &*packet.into_data()).map_err(|ReadError { context, kind }| ReadError {
                context: ErrorContext::WebSocket {
                    source: Box::new(context),
                },
                kind,
            })
        }),
    ))
}
