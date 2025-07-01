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
//! * [`bitvec`](https://docs.rs/bitvec): [`BitVec<u8, Lsb0>`](https://docs.rs/bitvec/latest/bitvec/vec/struct.BitVec.html)
//! * [`bytes`](https://docs.rs/bytes): [`Bytes`](https://docs.rs/bytes/latest/bytes/struct.Bytes.html)
//! * [`chrono`](https://docs.rs/chrono): [`NaiveDate`](https://docs.rs/chrono/latest/chrono/naive/struct.NaiveDate.html), [`DateTime`](https://docs.rs/chrono/latest/chrono/struct.DateTime.html), [`Utc`](https://docs.rs/chrono/latest/chrono/offset/struct.Utc.html), and [`FixedOffset`](https://docs.rs/chrono/latest/chrono/offset/struct.FixedOffset.html)
//! * [`chrono-tz`](https://docs.rs/chrono-tz): [`Tz`](https://docs.rs/chrono-tz/latest/chrono_tz/enum.Tz.html)
//! * [`doubloon`](https://docs.rs/doubloon): [`Money`](https://docs.rs/doubloon/latest/doubloon/struct.Money.html) and all [ISO currencies](https://docs.rs/doubloon/latest/doubloon/iso_currencies/index.html)
//! * [`either`](https://docs.rs/either): [`Either`](https://docs.rs/either/latest/either/enum.Either.html)
//! * [`enumset`](https://docs.rs/enumset): [`EnumSet`](https://docs.rs/enumset/latest/enumset/struct.EnumSet.html)
//! * [`git2`](https://docs.rs/git2): [`Oid`](https://docs.rs/git2/latest/git2/struct.Oid.html)
//! * [`gix-hash`](https://docs.rs/gix-hash): [`ObjectId`](https://docs.rs/gix-hash/latest/gix_hash/enum.ObjectId.html)
//! * [`noisy_float`](https://docs.rs/noisy_float): [`NoisyFloat`](https://docs.rs/noisy_float/latest/noisy_float/struct.NoisyFloat.html)
//! * [`rust_decimal`](https://docs.rs/rust_decimal): [`Decimal`](https://docs.rs/rust_decimal/latest/rust_decimal/struct.Decimal.html)
//! * [`semver`](https://docs.rs/semver): [`Version`](https://docs.rs/semver/latest/semver/struct.Version.html), [`Prerelease`](https://docs.rs/semver/latest/semver/struct.Prerelease.html), and [`BuildMetadata`](https://docs.rs/semver/latest/semver/struct.BuildMetadata.html)
//! * [`serde_json`](https://docs.rs/serde_json): [`Value`](https://docs.rs/serde_json/latest/serde_json/enum.Value.html), [`Map`](https://docs.rs/serde_json/latest/serde_json/struct.Map.html), and [`Number`](https://docs.rs/serde_json/latest/serde_json/struct.Number.html)
//! * [`serenity`](https://docs.rs/serenity): The [ID types](https://docs.rs/serenity/latest/serenity/model/id/index.html), not including [`ShardId`](https://docs.rs/serenity/latest/serenity/model/id/struct.ShardId.html)
//! * [`uuid`](https://docs.rs/uuid): [`Uuid`](https://docs.rs/uuid/latest/uuid/struct.Uuid.html)
//!
//! Additionally, this crate offers optional dependencies on the `tokio-tungstenite` crate to add convenience methods for reading/writing [`Protocol`] types from/to its websockets. The following versions are supported:
//!
//! * The latest release (currently [`tokio-tungstenite` 0.26](https://docs.rs/tokio-tungstenite/0.26), feature flag `tokio-tungstenite026`)
//! * The version used by [the `master` branch of `rocket_ws` on GitHub](https://github.com/rwf2/Rocket/tree/master/contrib/ws) (currently [`tokio-tungstenite` 0.24](https://docs.rs/tokio-tungstenite/0.24), feature flag `tokio-tungstenite024`)
//! * The version used by [the latest `rocket_ws` crates.io release](https://docs.rs/rocket_ws) (currently [`tokio-tungstenite` 0.21](https://docs.rs/tokio-tungstenite/0.21), feature flag `tokio-tungstenite021`)

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
#[cfg(any(feature = "tokio-tungstenite021", feature = "tokio-tungstenite024", feature = "tokio-tungstenite026"))] use {
    std::{
        iter,
        mem,
    },
    fallible_collections::FallibleVec,
    futures::{
        Sink,
        SinkExt as _,
        future::{
            self,
            Either,
        },
        stream::{
            self,
            Stream,
            StreamExt as _,
            TryStreamExt as _,
        },
    },
};
#[cfg(feature = "tokio-tungstenite021")] use tokio_tungstenite021::tungstenite as tungstenite021;
#[cfg(feature = "tokio-tungstenite024")] use tokio_tungstenite024::tungstenite as tungstenite024;
#[cfg(feature = "tokio-tungstenite026")] use tokio_tungstenite026::tungstenite as tungstenite026;
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

/// The maximum message size that can be sent and received by tokio-tungstenite without errors on the default configuration.
#[cfg(any(feature = "tokio-tungstenite021", feature = "tokio-tungstenite024", feature = "tokio-tungstenite026"))] const WS_MAX_MESSAGE_SIZE: usize = 16777216;

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

    #[cfg(feature = "tokio-tungstenite021")]
    #[cfg_attr(docsrs, doc(cfg(feature = "tokio-tungstenite021")))]
    /// Reads a value of this type from a `tokio-tungstenite` websocket.
    ///
    /// # Cancellation safety
    ///
    /// The default implementation of this method is not cancellation safe.
    fn read_ws021<'a, R: Stream<Item = Result<tungstenite021::Message, tungstenite021::Error>> + Unpin + Send + 'a>(stream: &'a mut R) -> Pin<Box<dyn Future<Output = Result<Self, ReadError>> + Send + 'a>> {
        Box::pin(async move {
            let packet = stream.try_next().await.map_err(|e| ReadError {
                context: ErrorContext::DefaultImpl,
                kind: e.into(),
            })?.ok_or_else(|| ReadError {
                context: ErrorContext::DefaultImpl,
                kind: ReadErrorKind::EndOfStream,
            })?;
            match packet {
                tungstenite021::Message::Text(data) => match data.chars().next() {
                    Some('m') => {
                        let len = data[1..].parse::<usize>().map_err(|e| ReadError {
                            context: ErrorContext::DefaultImpl,
                            kind: e.into(),
                        })?;
                        let mut buf = <Vec<_> as FallibleVec<_>>::try_with_capacity(len).map_err(|e| ReadError {
                            context: ErrorContext::DefaultImpl,
                            kind: e.into(),
                        })?;
                        while buf.len() < len {
                            let packet = stream.try_next().await.map_err(|e| ReadError {
                                context: ErrorContext::DefaultImpl,
                                kind: e.into(),
                            })?.ok_or_else(|| ReadError {
                                context: ErrorContext::DefaultImpl,
                                kind: ReadErrorKind::EndOfStream,
                            })?;
                            if let tungstenite021::Message::Binary(data) = packet {
                                buf.extend_from_slice(&data);
                            } else {
                                return Err(ReadError {
                                    context: ErrorContext::DefaultImpl,
                                    kind: ReadErrorKind::MessageKind021(packet),
                                })
                            }
                        }
                        Self::read_sync(&mut &*buf).map_err(|ReadError { context, kind }| ReadError {
                            context: ErrorContext::WebSocket {
                                source: Box::new(context),
                            },
                            kind,
                        })
                    }
                    _ => Err(ReadError {
                        context: ErrorContext::DefaultImpl,
                        kind: ReadErrorKind::WebSocketTextMessage024(data),
                    }),
                },
                tungstenite021::Message::Binary(data) => Self::read_sync(&mut &*data).map_err(|ReadError { context, kind }| ReadError {
                    context: ErrorContext::WebSocket {
                        source: Box::new(context),
                    },
                    kind,
                }),
                _ => Err(ReadError {
                    context: ErrorContext::DefaultImpl,
                    kind: ReadErrorKind::MessageKind021(packet),
                }),
            }
        })
    }

    #[cfg(feature = "tokio-tungstenite024")]
    #[cfg_attr(docsrs, doc(cfg(feature = "tokio-tungstenite024")))]
    /// Reads a value of this type from a `tokio-tungstenite` websocket.
    ///
    /// # Cancellation safety
    ///
    /// The default implementation of this method is not cancellation safe.
    fn read_ws024<'a, R: Stream<Item = Result<tungstenite024::Message, tungstenite024::Error>> + Unpin + Send + 'a>(stream: &'a mut R) -> Pin<Box<dyn Future<Output = Result<Self, ReadError>> + Send + 'a>> {
        Box::pin(async move {
            let packet = stream.try_next().await.map_err(|e| ReadError {
                context: ErrorContext::DefaultImpl,
                kind: e.into(),
            })?.ok_or_else(|| ReadError {
                context: ErrorContext::DefaultImpl,
                kind: ReadErrorKind::EndOfStream,
            })?;
            match packet {
                tungstenite024::Message::Text(data) => match data.chars().next() {
                    Some('m') => {
                        let len = data[1..].parse::<usize>().map_err(|e| ReadError {
                            context: ErrorContext::DefaultImpl,
                            kind: e.into(),
                        })?;
                        let mut buf = <Vec<_> as FallibleVec<_>>::try_with_capacity(len).map_err(|e| ReadError {
                            context: ErrorContext::DefaultImpl,
                            kind: e.into(),
                        })?;
                        while buf.len() < len {
                            let packet = stream.try_next().await.map_err(|e| ReadError {
                                context: ErrorContext::DefaultImpl,
                                kind: e.into(),
                            })?.ok_or_else(|| ReadError {
                                context: ErrorContext::DefaultImpl,
                                kind: ReadErrorKind::EndOfStream,
                            })?;
                            if let tungstenite024::Message::Binary(data) = packet {
                                buf.extend_from_slice(&data);
                            } else {
                                return Err(ReadError {
                                    context: ErrorContext::DefaultImpl,
                                    kind: ReadErrorKind::MessageKind024(packet),
                                })
                            }
                        }
                        Self::read_sync(&mut &*buf).map_err(|ReadError { context, kind }| ReadError {
                            context: ErrorContext::WebSocket {
                                source: Box::new(context),
                            },
                            kind,
                        })
                    }
                    _ => Err(ReadError {
                        context: ErrorContext::DefaultImpl,
                        kind: ReadErrorKind::WebSocketTextMessage024(data),
                    }),
                },
                tungstenite024::Message::Binary(data) => Self::read_sync(&mut &*data).map_err(|ReadError { context, kind }| ReadError {
                    context: ErrorContext::WebSocket {
                        source: Box::new(context),
                    },
                    kind,
                }),
                _ => Err(ReadError {
                    context: ErrorContext::DefaultImpl,
                    kind: ReadErrorKind::MessageKind024(packet),
                }),
            }
        })
    }

    #[cfg(feature = "tokio-tungstenite026")]
    #[cfg_attr(docsrs, doc(cfg(feature = "tokio-tungstenite026")))]
    /// Reads a value of this type from a `tokio-tungstenite` websocket.
    ///
    /// # Cancellation safety
    ///
    /// The default implementation of this method is not cancellation safe.
    fn read_ws026<'a, R: Stream<Item = Result<tungstenite026::Message, tungstenite026::Error>> + Unpin + Send + 'a>(stream: &'a mut R) -> Pin<Box<dyn Future<Output = Result<Self, ReadError>> + Send + 'a>> {
        Box::pin(async move {
            let packet = stream.try_next().await.map_err(|e| ReadError {
                context: ErrorContext::DefaultImpl,
                kind: e.into(),
            })?.ok_or_else(|| ReadError {
                context: ErrorContext::DefaultImpl,
                kind: ReadErrorKind::EndOfStream,
            })?;
            match packet {
                tungstenite026::Message::Text(data) => match data.chars().next() {
                    Some('m') => {
                        let len = data[1..].parse::<usize>().map_err(|e| ReadError {
                            context: ErrorContext::DefaultImpl,
                            kind: e.into(),
                        })?;
                        let mut buf = <Vec<_> as FallibleVec<_>>::try_with_capacity(len).map_err(|e| ReadError {
                            context: ErrorContext::DefaultImpl,
                            kind: e.into(),
                        })?;
                        while buf.len() < len {
                            let packet = stream.try_next().await.map_err(|e| ReadError {
                                context: ErrorContext::DefaultImpl,
                                kind: e.into(),
                            })?.ok_or_else(|| ReadError {
                                context: ErrorContext::DefaultImpl,
                                kind: ReadErrorKind::EndOfStream,
                            })?;
                            if let tungstenite026::Message::Binary(data) = packet {
                                buf.extend_from_slice(&data);
                            } else {
                                return Err(ReadError {
                                    context: ErrorContext::DefaultImpl,
                                    kind: ReadErrorKind::MessageKind026(packet),
                                })
                            }
                        }
                        Self::read_sync(&mut &*buf).map_err(|ReadError { context, kind }| ReadError {
                            context: ErrorContext::WebSocket {
                                source: Box::new(context),
                            },
                            kind,
                        })
                    }
                    _ => Err(ReadError {
                        context: ErrorContext::DefaultImpl,
                        kind: ReadErrorKind::WebSocketTextMessage026(data),
                    }),
                },
                tungstenite026::Message::Binary(data) => Self::read_sync(&mut &*data).map_err(|ReadError { context, kind }| ReadError {
                    context: ErrorContext::WebSocket {
                        source: Box::new(context),
                    },
                    kind,
                }),
                _ => Err(ReadError {
                    context: ErrorContext::DefaultImpl,
                    kind: ReadErrorKind::MessageKind026(packet),
                }),
            }
        })
    }

    #[cfg(feature = "tokio-tungstenite021")]
    #[cfg_attr(docsrs, doc(cfg(feature = "tokio-tungstenite021")))]
    /// Writes a value of this type to a `tokio-tungstenite` websocket.
    ///
    /// # Cancellation safety
    ///
    /// The default implementation of this method is not cancellation safe.
    fn write_ws021<'a, W: Sink<tungstenite021::Message, Error = tungstenite021::Error> + Unpin + Send + 'a>(&'a self, sink: &'a mut W) -> Pin<Box<dyn Future<Output = Result<(), WriteError>> + Send + 'a>>
    where Self: Sync {
        Box::pin(async move {
            let mut buf = Vec::default();
            self.write_sync(&mut buf).map_err(|WriteError { context, kind }| WriteError {
                context: ErrorContext::WebSocket {
                    source: Box::new(context),
                },
                kind,
            })?;
            if buf.len() <= WS_MAX_MESSAGE_SIZE {
                sink.send(tungstenite021::Message::binary(buf)).await.map_err(|e| WriteError {
                    context: ErrorContext::DefaultImpl,
                    kind: e.into(),
                })?;
            } else {
                sink.send(tungstenite021::Message::text(format!("m{}", buf.len()))).await.map_err(|e| WriteError {
                    context: ErrorContext::DefaultImpl,
                    kind: e.into(),
                })?;
                for chunk in buf.chunks(WS_MAX_MESSAGE_SIZE) {
                    sink.send(tungstenite021::Message::binary(chunk)).await.map_err(|e| WriteError {
                        context: ErrorContext::DefaultImpl,
                        kind: e.into(),
                    })?;
                }
            }
            Ok(())
        })
    }

    #[cfg(feature = "tokio-tungstenite024")]
    #[cfg_attr(docsrs, doc(cfg(feature = "tokio-tungstenite024")))]
    /// Writes a value of this type to a `tokio-tungstenite` websocket.
    ///
    /// # Cancellation safety
    ///
    /// The default implementation of this method is not cancellation safe.
    fn write_ws024<'a, W: Sink<tungstenite024::Message, Error = tungstenite024::Error> + Unpin + Send + 'a>(&'a self, sink: &'a mut W) -> Pin<Box<dyn Future<Output = Result<(), WriteError>> + Send + 'a>>
    where Self: Sync {
        Box::pin(async move {
            let mut buf = Vec::default();
            self.write_sync(&mut buf).map_err(|WriteError { context, kind }| WriteError {
                context: ErrorContext::WebSocket {
                    source: Box::new(context),
                },
                kind,
            })?;
            if buf.len() <= WS_MAX_MESSAGE_SIZE {
                sink.send(tungstenite024::Message::binary(buf)).await.map_err(|e| WriteError {
                    context: ErrorContext::DefaultImpl,
                    kind: e.into(),
                })?;
            } else {
                sink.send(tungstenite024::Message::text(format!("m{}", buf.len()))).await.map_err(|e| WriteError {
                    context: ErrorContext::DefaultImpl,
                    kind: e.into(),
                })?;
                for chunk in buf.chunks(WS_MAX_MESSAGE_SIZE) {
                    sink.send(tungstenite024::Message::binary(chunk)).await.map_err(|e| WriteError {
                        context: ErrorContext::DefaultImpl,
                        kind: e.into(),
                    })?;
                }
            }
            Ok(())
        })
    }

    #[cfg(feature = "tokio-tungstenite026")]
    #[cfg_attr(docsrs, doc(cfg(feature = "tokio-tungstenite026")))]
    /// Writes a value of this type to a `tokio-tungstenite` websocket.
    ///
    /// # Cancellation safety
    ///
    /// The default implementation of this method is not cancellation safe.
    fn write_ws026<'a, W: Sink<tungstenite026::Message, Error = tungstenite026::Error> + Unpin + Send + 'a>(&'a self, sink: &'a mut W) -> Pin<Box<dyn Future<Output = Result<(), WriteError>> + Send + 'a>>
    where Self: Sync {
        Box::pin(async move {
            let mut buf = Vec::default();
            self.write_sync(&mut buf).map_err(|WriteError { context, kind }| WriteError {
                context: ErrorContext::WebSocket {
                    source: Box::new(context),
                },
                kind,
            })?;
            if buf.len() <= WS_MAX_MESSAGE_SIZE {
                sink.send(tungstenite026::Message::binary(buf)).await.map_err(|e| WriteError {
                    context: ErrorContext::DefaultImpl,
                    kind: e.into(),
                })?;
            } else {
                sink.send(tungstenite026::Message::text(format!("m{}", buf.len()))).await.map_err(|e| WriteError {
                    context: ErrorContext::DefaultImpl,
                    kind: e.into(),
                })?;
                for chunk in buf.chunks(WS_MAX_MESSAGE_SIZE) {
                    sink.send(tungstenite026::Message::binary(tungstenite026::Bytes::copy_from_slice(chunk))).await.map_err(|e| WriteError {
                        context: ErrorContext::DefaultImpl,
                        kind: e.into(),
                    })?;
                }
            }
            Ok(())
        })
    }

    #[cfg(feature = "tokio-tungstenite021")]
    #[cfg_attr(docsrs, doc(cfg(feature = "tokio-tungstenite021")))]
    /// Reads a value of this type from a [`tungstenite021`] websocket.
    fn read_ws_sync021(websocket: &mut tungstenite021::WebSocket<impl Read + Write>) -> Result<Self, ReadError> {
        let packet = websocket.read().map_err(|e| ReadError {
            context: ErrorContext::DefaultImpl,
            kind: e.into(),
        })?;
        match packet {
            tungstenite021::Message::Text(data) => match data.chars().next() {
                Some('m') => {
                    let len = data[1..].parse::<usize>().map_err(|e| ReadError {
                        context: ErrorContext::DefaultImpl,
                        kind: e.into(),
                    })?;
                    let mut buf = <Vec<_> as FallibleVec<_>>::try_with_capacity(len).map_err(|e| ReadError {
                        context: ErrorContext::DefaultImpl,
                        kind: e.into(),
                    })?;
                    while buf.len() < len {
                        let packet = websocket.read().map_err(|e| ReadError {
                            context: ErrorContext::DefaultImpl,
                            kind: e.into(),
                        })?;
                        if let tungstenite021::Message::Binary(data) = packet {
                            buf.extend_from_slice(&data);
                        } else {
                            return Err(ReadError {
                                context: ErrorContext::DefaultImpl,
                                kind: ReadErrorKind::MessageKind021(packet),
                            })
                        }
                    }
                    Self::read_sync(&mut &*buf).map_err(|ReadError { context, kind }| ReadError {
                        context: ErrorContext::WebSocket {
                            source: Box::new(context),
                        },
                        kind,
                    })
                }
                _ => return Err(ReadError {
                    context: ErrorContext::DefaultImpl,
                    kind: ReadErrorKind::WebSocketTextMessage024(data),
                }),
            },
            tungstenite021::Message::Binary(data) => Self::read_sync(&mut &*data).map_err(|ReadError { context, kind }| ReadError {
                context: ErrorContext::WebSocket {
                    source: Box::new(context),
                },
                kind,
            }),
            _ => Err(ReadError {
                context: ErrorContext::DefaultImpl,
                kind: ReadErrorKind::MessageKind021(packet),
            }),
        }
    }

    #[cfg(feature = "tokio-tungstenite024")]
    #[cfg_attr(docsrs, doc(cfg(feature = "tokio-tungstenite024")))]
    /// Reads a value of this type from a [`tungstenite024`] websocket.
    fn read_ws_sync024(websocket: &mut tungstenite024::WebSocket<impl Read + Write>) -> Result<Self, ReadError> {
        let packet = websocket.read().map_err(|e| ReadError {
            context: ErrorContext::DefaultImpl,
            kind: e.into(),
        })?;
        match packet {
            tungstenite024::Message::Text(data) => match data.chars().next() {
                Some('m') => {
                    let len = data[1..].parse::<usize>().map_err(|e| ReadError {
                        context: ErrorContext::DefaultImpl,
                        kind: e.into(),
                    })?;
                    let mut buf = <Vec<_> as FallibleVec<_>>::try_with_capacity(len).map_err(|e| ReadError {
                        context: ErrorContext::DefaultImpl,
                        kind: e.into(),
                    })?;
                    while buf.len() < len {
                        let packet = websocket.read().map_err(|e| ReadError {
                            context: ErrorContext::DefaultImpl,
                            kind: e.into(),
                        })?;
                        if let tungstenite024::Message::Binary(data) = packet {
                            buf.extend_from_slice(&data);
                        } else {
                            return Err(ReadError {
                                context: ErrorContext::DefaultImpl,
                                kind: ReadErrorKind::MessageKind024(packet),
                            })
                        }
                    }
                    Self::read_sync(&mut &*buf).map_err(|ReadError { context, kind }| ReadError {
                        context: ErrorContext::WebSocket {
                            source: Box::new(context),
                        },
                        kind,
                    })
                }
                _ => return Err(ReadError {
                    context: ErrorContext::DefaultImpl,
                    kind: ReadErrorKind::WebSocketTextMessage024(data),
                }),
            },
            tungstenite024::Message::Binary(data) => Self::read_sync(&mut &*data).map_err(|ReadError { context, kind }| ReadError {
                context: ErrorContext::WebSocket {
                    source: Box::new(context),
                },
                kind,
            }),
            _ => Err(ReadError {
                context: ErrorContext::DefaultImpl,
                kind: ReadErrorKind::MessageKind024(packet),
            }),
        }
    }

    #[cfg(feature = "tokio-tungstenite026")]
    #[cfg_attr(docsrs, doc(cfg(feature = "tokio-tungstenite026")))]
    /// Reads a value of this type from a [`tungstenite026`] websocket.
    fn read_ws_sync026(websocket: &mut tungstenite026::WebSocket<impl Read + Write>) -> Result<Self, ReadError> {
        let packet = websocket.read().map_err(|e| ReadError {
            context: ErrorContext::DefaultImpl,
            kind: e.into(),
        })?;
        match packet {
            tungstenite026::Message::Text(data) => match data.chars().next() {
                Some('m') => {
                    let len = data[1..].parse::<usize>().map_err(|e| ReadError {
                        context: ErrorContext::DefaultImpl,
                        kind: e.into(),
                    })?;
                    let mut buf = <Vec<_> as FallibleVec<_>>::try_with_capacity(len).map_err(|e| ReadError {
                        context: ErrorContext::DefaultImpl,
                        kind: e.into(),
                    })?;
                    while buf.len() < len {
                        let packet = websocket.read().map_err(|e| ReadError {
                            context: ErrorContext::DefaultImpl,
                            kind: e.into(),
                        })?;
                        if let tungstenite026::Message::Binary(data) = packet {
                            buf.extend_from_slice(&data);
                        } else {
                            return Err(ReadError {
                                context: ErrorContext::DefaultImpl,
                                kind: ReadErrorKind::MessageKind026(packet),
                            })
                        }
                    }
                    Self::read_sync(&mut &*buf).map_err(|ReadError { context, kind }| ReadError {
                        context: ErrorContext::WebSocket {
                            source: Box::new(context),
                        },
                        kind,
                    })
                }
                _ => return Err(ReadError {
                    context: ErrorContext::DefaultImpl,
                    kind: ReadErrorKind::WebSocketTextMessage026(data),
                }),
            },
            tungstenite026::Message::Binary(data) => Self::read_sync(&mut &*data).map_err(|ReadError { context, kind }| ReadError {
                context: ErrorContext::WebSocket {
                    source: Box::new(context),
                },
                kind,
            }),
            _ => Err(ReadError {
                context: ErrorContext::DefaultImpl,
                kind: ReadErrorKind::MessageKind026(packet),
            }),
        }
    }

    #[cfg(feature = "tokio-tungstenite021")]
    #[cfg_attr(docsrs, doc(cfg(feature = "tokio-tungstenite021")))]
    /// Writes a value of this type to a [`tungstenite021`] websocket.
    fn write_ws_sync021(&self, websocket: &mut tungstenite021::WebSocket<impl Read + Write>) -> Result<(), WriteError> {
        let mut buf = Vec::default();
        self.write_sync(&mut buf).map_err(|WriteError { context, kind }| WriteError {
            context: ErrorContext::WebSocket {
                source: Box::new(context),
            },
            kind,
        })?;
        if buf.len() <= WS_MAX_MESSAGE_SIZE {
            websocket.send(tungstenite021::Message::binary(buf)).map_err(|e| WriteError {
                context: ErrorContext::DefaultImpl,
                kind: e.into(),
            })?;
        } else {
            websocket.send(tungstenite021::Message::text(format!("m{}", buf.len()))).map_err(|e| WriteError {
                context: ErrorContext::DefaultImpl,
                kind: e.into(),
            })?;
            for chunk in buf.chunks(WS_MAX_MESSAGE_SIZE) {
                websocket.send(tungstenite021::Message::binary(chunk)).map_err(|e| WriteError {
                    context: ErrorContext::DefaultImpl,
                    kind: e.into(),
                })?;
            }
        }
        websocket.flush().map_err(|e| WriteError {
            context: ErrorContext::DefaultImpl,
            kind: e.into(),
        })?;
        Ok(())
    }

    #[cfg(feature = "tokio-tungstenite024")]
    #[cfg_attr(docsrs, doc(cfg(feature = "tokio-tungstenite024")))]
    /// Writes a value of this type to a [`tungstenite024`] websocket.
    fn write_ws_sync024(&self, websocket: &mut tungstenite024::WebSocket<impl Read + Write>) -> Result<(), WriteError> {
        let mut buf = Vec::default();
        self.write_sync(&mut buf).map_err(|WriteError { context, kind }| WriteError {
            context: ErrorContext::WebSocket {
                source: Box::new(context),
            },
            kind,
        })?;
        if buf.len() <= WS_MAX_MESSAGE_SIZE {
            websocket.send(tungstenite024::Message::binary(buf)).map_err(|e| WriteError {
                context: ErrorContext::DefaultImpl,
                kind: e.into(),
            })?;
        } else {
            websocket.send(tungstenite024::Message::text(format!("m{}", buf.len()))).map_err(|e| WriteError {
                context: ErrorContext::DefaultImpl,
                kind: e.into(),
            })?;
            for chunk in buf.chunks(WS_MAX_MESSAGE_SIZE) {
                websocket.send(tungstenite024::Message::binary(chunk)).map_err(|e| WriteError {
                    context: ErrorContext::DefaultImpl,
                    kind: e.into(),
                })?;
            }
        }
        websocket.flush().map_err(|e| WriteError {
            context: ErrorContext::DefaultImpl,
            kind: e.into(),
        })?;
        Ok(())
    }

    #[cfg(feature = "tokio-tungstenite026")]
    #[cfg_attr(docsrs, doc(cfg(feature = "tokio-tungstenite026")))]
    /// Writes a value of this type to a [`tungstenite026`] websocket.
    fn write_ws_sync026(&self, websocket: &mut tungstenite026::WebSocket<impl Read + Write>) -> Result<(), WriteError> {
        let mut buf = Vec::default();
        self.write_sync(&mut buf).map_err(|WriteError { context, kind }| WriteError {
            context: ErrorContext::WebSocket {
                source: Box::new(context),
            },
            kind,
        })?;
        if buf.len() <= WS_MAX_MESSAGE_SIZE {
            websocket.send(tungstenite026::Message::binary(buf)).map_err(|e| WriteError {
                context: ErrorContext::DefaultImpl,
                kind: e.into(),
            })?;
        } else {
            websocket.send(tungstenite026::Message::text(format!("m{}", buf.len()))).map_err(|e| WriteError {
                context: ErrorContext::DefaultImpl,
                kind: e.into(),
            })?;
            for chunk in buf.chunks(WS_MAX_MESSAGE_SIZE) {
                websocket.send(tungstenite026::Message::binary(tungstenite026::Bytes::copy_from_slice(chunk))).map_err(|e| WriteError {
                    context: ErrorContext::DefaultImpl,
                    kind: e.into(),
                })?;
            }
        }
        websocket.flush().map_err(|e| WriteError {
            context: ErrorContext::DefaultImpl,
            kind: e.into(),
        })?;
        Ok(())
    }

    #[cfg(feature = "tokio-tungstenite021")]
    #[cfg_attr(docsrs, doc(cfg(feature = "tokio-tungstenite021")))]
    /// Takes ownership of an async websocket stream, reads a value of this type from it, then returns it along with the stream.
    ///
    /// This can be used to get around drop glue issues that might arise with `read_ws`.
    fn read_ws_owned021<R: Stream<Item = Result<tungstenite021::Message, tungstenite021::Error>> + Unpin + Send + 'static>(mut stream: R) -> Pin<Box<dyn Future<Output = Result<(R, Self), ReadError>> + Send>> {
        Box::pin(async move {
            let value = Self::read_ws021(&mut stream).await?;
            Ok((stream, value))
        })
    }

    #[cfg(feature = "tokio-tungstenite024")]
    #[cfg_attr(docsrs, doc(cfg(feature = "tokio-tungstenite024")))]
    /// Takes ownership of an async websocket stream, reads a value of this type from it, then returns it along with the stream.
    ///
    /// This can be used to get around drop glue issues that might arise with `read_ws`.
    fn read_ws_owned024<R: Stream<Item = Result<tungstenite024::Message, tungstenite024::Error>> + Unpin + Send + 'static>(mut stream: R) -> Pin<Box<dyn Future<Output = Result<(R, Self), ReadError>> + Send>> {
        Box::pin(async move {
            let value = Self::read_ws024(&mut stream).await?;
            Ok((stream, value))
        })
    }

    #[cfg(feature = "tokio-tungstenite026")]
    #[cfg_attr(docsrs, doc(cfg(feature = "tokio-tungstenite026")))]
    /// Takes ownership of an async websocket stream, reads a value of this type from it, then returns it along with the stream.
    ///
    /// This can be used to get around drop glue issues that might arise with `read_ws`.
    fn read_ws_owned026<R: Stream<Item = Result<tungstenite026::Message, tungstenite026::Error>> + Unpin + Send + 'static>(mut stream: R) -> Pin<Box<dyn Future<Output = Result<(R, Self), ReadError>> + Send>> {
        Box::pin(async move {
            let value = Self::read_ws026(&mut stream).await?;
            Ok((stream, value))
        })
    }
}

/// Establishes a WebSocket connection to the given URL and returns a typed sink/stream pair.
///
/// Useful for WebSocket connections where the message type per direction is always the same.
#[cfg(feature = "tokio-tungstenite021")]
#[cfg_attr(docsrs, doc(cfg(feature = "tokio-tungstenite021")))]
pub async fn websocket021<R: Protocol, W: Protocol>(request: impl tungstenite021::client::IntoClientRequest + Unpin) -> tungstenite021::Result<(impl Sink<W, Error = WriteError>, impl Stream<Item = Result<R, ReadError>>)> {
    let (sock, _) = tokio_tungstenite021::connect_async(request).await?;
    let (sink, stream) = sock.split();
    Ok((
        sink.sink_map_err(|e| WriteError {
            context: ErrorContext::WebSocketSink,
            kind: e.into(),
        }).with_flat_map::<W, _, _>(|msg| {
            let mut buf = Vec::default();
            match msg.write_sync(&mut buf) {
                Ok(()) => Either::Left(if buf.len() <= WS_MAX_MESSAGE_SIZE {
                    Either::Left(stream::once(future::ready(tungstenite021::Message::binary(buf))))
                } else {
                    Either::Right(stream::iter(
                        iter::once(tungstenite021::Message::text(format!("m{}", buf.len())))
                        .chain(buf.chunks(WS_MAX_MESSAGE_SIZE).map(tungstenite021::Message::binary))
                        .collect::<Vec<_>>()
                    ))
                }.map(Ok)),
                Err(WriteError { context, kind }) => Either::Right(stream::once(future::err(WriteError {
                    context: ErrorContext::WebSocket {
                        source: Box::new(context),
                    },
                    kind,
                }))),
            }
        }),
        stream.scan(None, |state, res| {
            fn scanner<R: Protocol>(state: &mut Option<(usize, Vec<u8>)>, res: tungstenite021::Result<tungstenite021::Message>) -> Result<impl Stream<Item = Result<R, ReadError>> + use<R>, ReadError> {
                let packet = res.map_err(|e| ReadError {
                    context: ErrorContext::WebSocketStream,
                    kind: e.into(),
                })?;
                Ok(if let Some((len, buf)) = state {
                    if let tungstenite021::Message::Binary(data) = packet {
                        buf.extend_from_slice(&data);
                    } else {
                        return Err(ReadError {
                            context: ErrorContext::DefaultImpl,
                            kind: ReadErrorKind::MessageKind021(packet),
                        })
                    }
                    if buf.len() >= *len {
                        let buf = mem::take(buf);
                        *state = None;
                        Either::Right(stream::once(future::ok(R::read_sync(&mut &*buf).map_err(|ReadError { context, kind }| ReadError {
                            context: ErrorContext::WebSocket {
                                source: Box::new(context),
                            },
                            kind,
                        })?)))
                    } else {
                        Either::Left(stream::empty())
                    }
                } else {
                    match packet {
                        tungstenite021::Message::Text(data) => match data.chars().next() {
                            Some('m') => {
                                let len = data[1..].parse::<usize>().map_err(|e| ReadError {
                                    context: ErrorContext::DefaultImpl,
                                    kind: e.into(),
                                })?;
                                let buf = FallibleVec::try_with_capacity(len).map_err(|e| ReadError {
                                    context: ErrorContext::DefaultImpl,
                                    kind: e.into(),
                                })?;
                                *state = Some((len, buf));
                                Either::Left(stream::empty())
                            }
                            _ => return Err(ReadError {
                                context: ErrorContext::DefaultImpl,
                                kind: ReadErrorKind::WebSocketTextMessage024(data),
                            }),
                        },
                        tungstenite021::Message::Binary(data) => Either::Right(stream::once(future::ok(R::read_sync(&mut &*data).map_err(|ReadError { context, kind }| ReadError {
                            context: ErrorContext::WebSocket {
                                source: Box::new(context),
                            },
                            kind,
                        })?))),
                        _ => return Err(ReadError {
                            context: ErrorContext::DefaultImpl,
                            kind: ReadErrorKind::MessageKind021(packet),
                        }),
                    }
                })
            }

            future::ready(Some(scanner(state, res)))
        }).try_flatten(),
    ))
}

/// Establishes a WebSocket connection to the given URL and returns a typed sink/stream pair.
///
/// Useful for WebSocket connections where the message type per direction is always the same.
#[cfg(feature = "tokio-tungstenite024")]
#[cfg_attr(docsrs, doc(cfg(feature = "tokio-tungstenite024")))]
pub async fn websocket024<R: Protocol, W: Protocol>(request: impl tungstenite024::client::IntoClientRequest + Unpin) -> tungstenite024::Result<(impl Sink<W, Error = WriteError>, impl Stream<Item = Result<R, ReadError>>)> {
    let (sock, _) = tokio_tungstenite024::connect_async(request).await?;
    let (sink, stream) = sock.split();
    Ok((
        sink.sink_map_err(|e| WriteError {
            context: ErrorContext::WebSocketSink,
            kind: e.into(),
        }).with_flat_map::<W, _, _>(|msg| {
            let mut buf = Vec::default();
            match msg.write_sync(&mut buf) {
                Ok(()) => Either::Left(if buf.len() <= WS_MAX_MESSAGE_SIZE {
                    Either::Left(stream::once(future::ready(tungstenite024::Message::binary(buf))))
                } else {
                    Either::Right(stream::iter(
                        iter::once(tungstenite024::Message::text(format!("m{}", buf.len())))
                        .chain(buf.chunks(WS_MAX_MESSAGE_SIZE).map(tungstenite024::Message::binary))
                        .collect::<Vec<_>>()
                    ))
                }.map(Ok)),
                Err(WriteError { context, kind }) => Either::Right(stream::once(future::err(WriteError {
                    context: ErrorContext::WebSocket {
                        source: Box::new(context),
                    },
                    kind,
                }))),
            }
        }),
        stream.scan(None, |state, res| {
            fn scanner<R: Protocol>(state: &mut Option<(usize, Vec<u8>)>, res: tungstenite024::Result<tungstenite024::Message>) -> Result<impl Stream<Item = Result<R, ReadError>> + use<R>, ReadError> {
                let packet = res.map_err(|e| ReadError {
                    context: ErrorContext::WebSocketStream,
                    kind: e.into(),
                })?;
                Ok(if let Some((len, buf)) = state {
                    if let tungstenite024::Message::Binary(data) = packet {
                        buf.extend_from_slice(&data);
                    } else {
                        return Err(ReadError {
                            context: ErrorContext::DefaultImpl,
                            kind: ReadErrorKind::MessageKind024(packet),
                        })
                    }
                    if buf.len() >= *len {
                        let buf = mem::take(buf);
                        *state = None;
                        Either::Right(stream::once(future::ok(R::read_sync(&mut &*buf).map_err(|ReadError { context, kind }| ReadError {
                            context: ErrorContext::WebSocket {
                                source: Box::new(context),
                            },
                            kind,
                        })?)))
                    } else {
                        Either::Left(stream::empty())
                    }
                } else {
                    match packet {
                        tungstenite024::Message::Text(data) => match data.chars().next() {
                            Some('m') => {
                                let len = data[1..].parse::<usize>().map_err(|e| ReadError {
                                    context: ErrorContext::DefaultImpl,
                                    kind: e.into(),
                                })?;
                                let buf = FallibleVec::try_with_capacity(len).map_err(|e| ReadError {
                                    context: ErrorContext::DefaultImpl,
                                    kind: e.into(),
                                })?;
                                *state = Some((len, buf));
                                Either::Left(stream::empty())
                            }
                            _ => return Err(ReadError {
                                context: ErrorContext::DefaultImpl,
                                kind: ReadErrorKind::WebSocketTextMessage024(data),
                            }),
                        },
                        tungstenite024::Message::Binary(data) => Either::Right(stream::once(future::ok(R::read_sync(&mut &*data).map_err(|ReadError { context, kind }| ReadError {
                            context: ErrorContext::WebSocket {
                                source: Box::new(context),
                            },
                            kind,
                        })?))),
                        _ => return Err(ReadError {
                            context: ErrorContext::DefaultImpl,
                            kind: ReadErrorKind::MessageKind024(packet),
                        }),
                    }
                })
            }

            future::ready(Some(scanner(state, res)))
        }).try_flatten(),
    ))
}

/// Establishes a WebSocket connection to the given URL and returns a typed sink/stream pair.
///
/// Useful for WebSocket connections where the message type per direction is always the same.
#[cfg(feature = "tokio-tungstenite026")]
#[cfg_attr(docsrs, doc(cfg(feature = "tokio-tungstenite026")))]
pub async fn websocket026<R: Protocol, W: Protocol>(request: impl tungstenite026::client::IntoClientRequest + Unpin) -> tungstenite026::Result<(impl Sink<W, Error = WriteError>, impl Stream<Item = Result<R, ReadError>>)> {
    let (sock, _) = tokio_tungstenite026::connect_async(request).await?;
    let (sink, stream) = sock.split();
    Ok((
        sink.sink_map_err(|e| WriteError {
            context: ErrorContext::WebSocketSink,
            kind: e.into(),
        }).with_flat_map::<W, _, _>(|msg| {
            let mut buf = Vec::default();
            match msg.write_sync(&mut buf) {
                Ok(()) => Either::Left(if buf.len() <= WS_MAX_MESSAGE_SIZE {
                    Either::Left(stream::once(future::ready(tungstenite026::Message::binary(buf))))
                } else {
                    Either::Right(stream::iter(
                        iter::once(tungstenite026::Message::text(format!("m{}", buf.len())))
                        .chain(buf.chunks(WS_MAX_MESSAGE_SIZE).map(|chunk| tungstenite026::Message::binary(tungstenite026::Bytes::copy_from_slice(chunk))))
                        .collect::<Vec<_>>()
                    ))
                }.map(Ok)),
                Err(WriteError { context, kind }) => Either::Right(stream::once(future::err(WriteError {
                    context: ErrorContext::WebSocket {
                        source: Box::new(context),
                    },
                    kind,
                }))),
            }
        }),
        stream.scan(None, |state, res| {
            fn scanner<R: Protocol>(state: &mut Option<(usize, Vec<u8>)>, res: tungstenite026::Result<tungstenite026::Message>) -> Result<impl Stream<Item = Result<R, ReadError>> + use<R>, ReadError> {
                let packet = res.map_err(|e| ReadError {
                    context: ErrorContext::WebSocketStream,
                    kind: e.into(),
                })?;
                Ok(if let Some((len, buf)) = state {
                    if let tungstenite026::Message::Binary(data) = packet {
                        buf.extend_from_slice(&data);
                    } else {
                        return Err(ReadError {
                            context: ErrorContext::DefaultImpl,
                            kind: ReadErrorKind::MessageKind026(packet),
                        })
                    }
                    if buf.len() >= *len {
                        let buf = mem::take(buf);
                        *state = None;
                        Either::Right(stream::once(future::ok(R::read_sync(&mut &*buf).map_err(|ReadError { context, kind }| ReadError {
                            context: ErrorContext::WebSocket {
                                source: Box::new(context),
                            },
                            kind,
                        })?)))
                    } else {
                        Either::Left(stream::empty())
                    }
                } else {
                    match packet {
                        tungstenite026::Message::Text(data) => match data.chars().next() {
                            Some('m') => {
                                let len = data[1..].parse::<usize>().map_err(|e| ReadError {
                                    context: ErrorContext::DefaultImpl,
                                    kind: e.into(),
                                })?;
                                let buf = FallibleVec::try_with_capacity(len).map_err(|e| ReadError {
                                    context: ErrorContext::DefaultImpl,
                                    kind: e.into(),
                                })?;
                                *state = Some((len, buf));
                                Either::Left(stream::empty())
                            }
                            _ => return Err(ReadError {
                                context: ErrorContext::DefaultImpl,
                                kind: ReadErrorKind::WebSocketTextMessage026(data),
                            }),
                        },
                        tungstenite026::Message::Binary(data) => Either::Right(stream::once(future::ok(R::read_sync(&mut &*data).map_err(|ReadError { context, kind }| ReadError {
                            context: ErrorContext::WebSocket {
                                source: Box::new(context),
                            },
                            kind,
                        })?))),
                        _ => return Err(ReadError {
                            context: ErrorContext::DefaultImpl,
                            kind: ReadErrorKind::MessageKind026(packet),
                        }),
                    }
                })
            }

            future::ready(Some(scanner(state, res)))
        }).try_flatten(),
    ))
}
