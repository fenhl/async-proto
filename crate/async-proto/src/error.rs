use std::{
    borrow::Cow,
    convert::Infallible,
    io,
};
#[cfg(feature = "tokio-tungstenite021")] use tokio_tungstenite021::tungstenite as tungstenite021;
#[cfg(feature = "tokio-tungstenite024")] use tokio_tungstenite024::tungstenite as tungstenite024;
#[cfg(feature = "tokio-tungstenite027")] use tokio_tungstenite027::tungstenite as tungstenite027;

/// Specifies what went wrong while reading (receiving) a value.
#[derive(Debug, thiserror::Error)]
#[allow(missing_docs)]
pub enum ReadErrorKind {
    /// Received a buffer with more than [`usize::MAX`] elements
    #[error("received a buffer with more than usize::MAX elements: {0}")]
    BufSize(#[from] std::num::TryFromIntError),
    /// An error variant you can use when manually implementing [`Protocol`](crate::Protocol)
    #[error("{0}")]
    Custom(String),
    /// The end of the stream was encountered before a complete value was read.
    ///
    /// Note that this error condition may also be represented as a [`ReadErrorKind::Io`] with [`kind`](io::Error::kind) [`UnexpectedEof`](io::ErrorKind::UnexpectedEof).
    #[error("reached end of stream")]
    EndOfStream,
    #[error("received an infinite or NaN number")]
    FloatNotFinite,
    #[error("received length ({len}) exceeds specified maximum length ({max_len})")]
    MaxLen {
        len: u64,
        max_len: u64,
    },
    #[cfg(feature = "tokio-tungstenite021")]
    #[cfg_attr(docsrs, doc(cfg(feature = "tokio-tungstenite021")))]
    /// Received a non-`Binary` WebSocket message (e.g. `Text` or `Ping`).
    #[error("unexpected type of WebSocket message")]
    MessageKind021(tungstenite021::Message),
    #[cfg(feature = "tokio-tungstenite024")]
    #[cfg_attr(docsrs, doc(cfg(feature = "tokio-tungstenite024")))]
    /// Received a non-`Binary` WebSocket message (e.g. `Text` or `Ping`).
    #[error("unexpected type of WebSocket message")]
    MessageKind024(tungstenite024::Message),
    #[cfg(feature = "tokio-tungstenite027")]
    #[cfg_attr(docsrs, doc(cfg(feature = "tokio-tungstenite027")))]
    /// Received a non-`Binary` WebSocket message (e.g. `Text` or `Ping`).
    #[error("unexpected type of WebSocket message")]
    MessageKind027(tungstenite027::Message),
    /// Attempted to read an empty type
    #[error("attempted to read an empty type")]
    ReadNever,
    #[error("{0:?}")] // fallible_collections::TryReserveError does not implement Error, see https://github.com/vcombey/fallible_collections/pull/44
    TryReserve(fallible_collections::TryReserveError),
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
    #[cfg(any(any(feature = "tokio-tungstenite021", feature = "tokio-tungstenite024")))]
    #[cfg_attr(docsrs, doc(cfg(any(any(feature = "tokio-tungstenite021", feature = "tokio-tungstenite024")))))]
    #[error("unexpected text message received from WebSocket: {0}")]
    WebSocketTextMessage024(String),
    #[cfg(any(feature = "tokio-tungstenite027"))]
    #[cfg_attr(docsrs, doc(cfg(any(feature = "tokio-tungstenite027"))))]
    #[error("unexpected text message received from WebSocket: {0}")]
    WebSocketTextMessage027(tungstenite027::Utf8Bytes),
    #[error(transparent)] Io(#[from] io::Error),
    #[cfg(any(any(feature = "tokio-tungstenite021", feature = "tokio-tungstenite024", feature = "tokio-tungstenite027")))]
    #[cfg_attr(docsrs, doc(cfg(any(any(feature = "tokio-tungstenite021", feature = "tokio-tungstenite024", feature = "tokio-tungstenite027")))))]
    #[error(transparent)] ParseInt(#[from] std::num::ParseIntError),
    #[cfg(any(feature = "tokio-tungstenite021"))]
    #[cfg_attr(docsrs, doc(cfg(any(feature = "tokio-tungstenite021"))))]
    #[error(transparent)] Tungstenite021(#[from] tungstenite021::Error),
    #[cfg(any(feature = "tokio-tungstenite024"))]
    #[cfg_attr(docsrs, doc(cfg(any(feature = "tokio-tungstenite024"))))]
    #[error(transparent)] Tungstenite024(#[from] tungstenite024::Error),
    #[cfg(any(feature = "tokio-tungstenite027"))]
    #[cfg_attr(docsrs, doc(cfg(any(feature = "tokio-tungstenite027"))))]
    #[error(transparent)] Tungstenite027(#[from] tungstenite027::Error),
    #[error(transparent)] Utf8(#[from] std::string::FromUtf8Error),
}

impl From<Infallible> for ReadErrorKind {
    fn from(never: Infallible) -> Self {
        match never {}
    }
}

impl From<String> for ReadErrorKind {
    fn from(s: String) -> Self {
        Self::Custom(s)
    }
}

impl<'a> From<&'a str> for ReadErrorKind {
    fn from(s: &str) -> Self {
        Self::Custom(s.to_owned())
    }
}

impl<'a> From<Cow<'a, str>> for ReadErrorKind {
    fn from(s: Cow<'a, str>) -> Self {
        Self::Custom(s.into_owned())
    }
}

impl From<fallible_collections::TryReserveError> for ReadErrorKind {
    fn from(e: fallible_collections::TryReserveError) -> Self {
        Self::TryReserve(e)
    }
}

impl From<ReadErrorKind> for io::Error {
    fn from(e: ReadErrorKind) -> Self {
        match e {
            ReadErrorKind::BufSize(e) => io::Error::new(io::ErrorKind::InvalidData, e),
            ReadErrorKind::Io(e) => e,
            #[cfg(feature = "tokio-tungstenite021")] ReadErrorKind::Tungstenite021(e) => io::Error::new(io::ErrorKind::Other, e),
            #[cfg(feature = "tokio-tungstenite024")] ReadErrorKind::Tungstenite024(e) => io::Error::new(io::ErrorKind::Other, e),
            #[cfg(feature = "tokio-tungstenite027")] ReadErrorKind::Tungstenite027(e) => io::Error::new(io::ErrorKind::Other, e),
            ReadErrorKind::Utf8(e) => io::Error::new(io::ErrorKind::InvalidData, e),
            ReadErrorKind::EndOfStream => io::Error::new(io::ErrorKind::UnexpectedEof, e),
            #[cfg(any(feature = "tokio-tungstenite021", feature = "tokio-tungstenite024"))] ReadErrorKind::WebSocketTextMessage024(ref msg) => io::Error::new(if msg.is_empty() { io::ErrorKind::UnexpectedEof } else { io::ErrorKind::InvalidData }, e),
            #[cfg(feature = "tokio-tungstenite027")] ReadErrorKind::WebSocketTextMessage027(ref msg) => io::Error::new(if msg.is_empty() { io::ErrorKind::UnexpectedEof } else { io::ErrorKind::InvalidData }, e),
            ReadErrorKind::FloatNotFinite |
            ReadErrorKind::MaxLen { .. } |
            ReadErrorKind::UnknownVariant8(_) |
            ReadErrorKind::UnknownVariant16(_) |
            ReadErrorKind::UnknownVariant32(_) |
            ReadErrorKind::UnknownVariant64(_) |
            ReadErrorKind::UnknownVariant128(_) => io::Error::new(io::ErrorKind::InvalidData, e),
            #[cfg(feature = "tokio-tungstenite021")] ReadErrorKind::MessageKind021(_) => io::Error::new(io::ErrorKind::InvalidData, e),
            #[cfg(feature = "tokio-tungstenite024")] ReadErrorKind::MessageKind024(_) => io::Error::new(io::ErrorKind::InvalidData, e),
            #[cfg(feature = "tokio-tungstenite027")] ReadErrorKind::MessageKind027(_) => io::Error::new(io::ErrorKind::InvalidData, e),
            #[cfg(any(feature = "tokio-tungstenite021", feature = "tokio-tungstenite024", feature = "tokio-tungstenite027"))] ReadErrorKind::ParseInt(_) => io::Error::new(io::ErrorKind::InvalidData, e),
            ReadErrorKind::ReadNever => io::Error::new(io::ErrorKind::InvalidInput, e),
            ReadErrorKind::TryReserve(_) => io::Error::new(io::ErrorKind::OutOfMemory, e),
            ReadErrorKind::Custom(_) => io::Error::new(io::ErrorKind::Other, e),
        }
    }
}

impl From<ReadError> for io::Error {
    fn from(ReadError { kind, .. }: ReadError) -> Self {
        kind.into()
    }
}

/// The error returned from the [`read`](crate::Protocol::read) and [`read_sync`](crate::Protocol::read_sync) methods.
#[derive(Debug, thiserror::Error)]
#[error("{kind}")]
pub struct ReadError {
    /// Where it went wrong.
    pub context: ErrorContext,
    /// What went wrong.
    pub kind: ReadErrorKind,
}

/// Specifies what went wrong while writing (sending) a value.
#[derive(Debug, thiserror::Error)]
#[allow(missing_docs)]
pub enum WriteErrorKind {
    /// Tried to send a buffer with more than [`u64::MAX`] elements
    #[error("tried to send a buffer with more than u64::MAX elements: {0}")]
    BufSize(#[from] std::num::TryFromIntError),
    /// An error variant you can use when manually implementing [`Protocol`](crate::Protocol)
    #[error("{0}")]
    Custom(String),
    #[error(transparent)] Io(#[from] io::Error),
    #[error("attempted to write length {len} exceeding specified maximum length ({max_len})")]
    MaxLen {
        len: u64,
        max_len: u64,
    },
    #[cfg(feature = "tokio-tungstenite021")]
    #[cfg_attr(docsrs, doc(cfg(feature = "tokio-tungstenite021")))]
    #[error(transparent)] Tungstenite021(#[from] tungstenite021::Error),
    #[cfg(feature = "tokio-tungstenite024")]
    #[cfg_attr(docsrs, doc(cfg(feature = "tokio-tungstenite024")))]
    #[error(transparent)] Tungstenite024(#[from] tungstenite024::Error),
    #[cfg(feature = "tokio-tungstenite027")]
    #[cfg_attr(docsrs, doc(cfg(feature = "tokio-tungstenite027")))]
    #[error(transparent)] Tungstenite027(#[from] tungstenite027::Error),
}

impl From<Infallible> for WriteErrorKind {
    fn from(never: Infallible) -> Self {
        match never {}
    }
}

impl From<String> for WriteErrorKind {
    fn from(s: String) -> Self {
        Self::Custom(s)
    }
}

impl<'a> From<&'a str> for WriteErrorKind {
    fn from(s: &str) -> Self {
        Self::Custom(s.to_owned())
    }
}

impl<'a> From<Cow<'a, str>> for WriteErrorKind {
    fn from(s: Cow<'a, str>) -> Self {
        Self::Custom(s.into_owned())
    }
}

impl From<WriteErrorKind> for io::Error {
    fn from(e: WriteErrorKind) -> Self {
        match e {
            WriteErrorKind::BufSize(e) => io::Error::new(io::ErrorKind::InvalidData, e),
            WriteErrorKind::Io(e) => e,
            WriteErrorKind::MaxLen { .. } => io::Error::new(io::ErrorKind::InvalidData, e),
            #[cfg(feature = "tokio-tungstenite021")] WriteErrorKind::Tungstenite021(e) => io::Error::new(io::ErrorKind::Other, e),
            #[cfg(feature = "tokio-tungstenite024")] WriteErrorKind::Tungstenite024(e) => io::Error::new(io::ErrorKind::Other, e),
            #[cfg(feature = "tokio-tungstenite027")] WriteErrorKind::Tungstenite027(e) => io::Error::new(io::ErrorKind::Other, e),
            WriteErrorKind::Custom(_) => io::Error::new(io::ErrorKind::Other, e),
        }
    }
}

impl From<WriteError> for io::Error {
    fn from(WriteError { kind, .. }: WriteError) -> Self {
        kind.into()
    }
}

/// The error returned from the [`write`](crate::Protocol::write) and [`write_sync`](crate::Protocol::write_sync) methods.
#[derive(Debug, thiserror::Error)]
#[error("{kind}")]
pub struct WriteError {
    /// Where it went wrong.
    pub context: ErrorContext,
    /// What went wrong.
    pub kind: WriteErrorKind,
}

/// Provides additional information about the origin of an error.
#[derive(Debug)]
pub enum ErrorContext {
    /// An error context you can use when manually implementing `Protocol`.
    Custom(String),
    /// The error was produced by a `Protocol` implementation defined in the `async-proto` crate.
    BuiltIn {
        /// The name of the type whose `Protocol` implementation produced the error.
        ///
        /// Typically does not include type parameters.
        for_type: &'static str,
    },
    /// The error occurred while reading/writing a WebSocket message.
    WebSocket {
        /// The context of the error returned from the message's `Protocol` implementation.
        source: Box<Self>,
    },
    /// The error was produced by a sink returned from [`async_proto::websocket021`](crate::websocket021) and [`async_proto::websocket024`](crate::websocket024).
    WebSocketSink,
    /// The error was produced by a stream returned from [`async_proto::websocket021`](crate::websocket021) and [`async_proto::websocket024`](crate::websocket024).
    WebSocketStream,
    /// The error was produced by the default implementation of a `Protocol` trait method.
    DefaultImpl,
    /// The error was produced by an automatically derived `Protocol` implementation.
    Derived {
        /// The name of the type whose `Protocol` implementation produced the error.
        for_type: &'static str,
    },
    /// The error occurred while reading/writing the discriminant of an enum.
    EnumDiscrim {
        /// The context of the error returned from the discriminant type's `Protocol` implementation.
        source: Box<Self>,
    },
    /// The error occurred while reading/writing a field of a tuple, tuple struct, or tuple enum variant.
    UnnamedField {
        /// The position of the field, starting at 0.
        idx: usize,
        /// The context of the error returned from the field's `Protocol` implementation.
        source: Box<Self>,
    },
    /// The error occurred while reading/writing a field of a struct or struct enum variant.
    NamedField {
        /// The name of the field.
        name: &'static str,
        /// The context of the error returned from the field's `Protocol` implementation.
        source: Box<Self>,
    },
    /// The error occurred in the `FromStr` implementation of a type whose `Protocol` implementation was derived with `#[async_proto(as_string)]`.
    FromStr,
    /// The error occurred while reading/writing a string representing a type whose `Protocol` implementation was derived with `#[async_proto(as_string)]`.
    AsString {
        /// The context of the error returned from `String`'s `Protocol` implementation.
        source: Box<Self>,
    },
    /// The error occurred in the `TryInto` implementation for a type whose `Protocol` implementation was derived with `#[async_proto(via = ...)]`.
    TryInto,
    /// The error occurred while reading/writing a proxy type representing a type whose `Protocol` implementation was derived with `#[async_proto(via ...)]`.
    Via {
        /// The context of the error returned from the proxy type's `Protocol` implementation.
        source: Box<Self>,
    },
    /// The error was produced by the `async_proto::bitflags` macro.
    Bitflags {
        /// The context of the error returned from the bits type's `Protocol` implementation.
        source: Box<Self>,
    },
}
