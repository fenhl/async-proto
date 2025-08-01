//! [`Protocol`] implementations for primitive and [`std`] types.

#![allow(missing_docs)]

use {
    std::{
        collections::{
            BTreeMap,
            BTreeSet,
            HashMap,
            HashSet,
        },
        convert::{
            TryFrom as _,
            TryInto as _,
        },
        future::Future,
        hash::Hash,
        io::prelude::*,
        ops::{
            Range,
            RangeFrom,
            RangeInclusive,
            RangeTo,
            RangeToInclusive,
        },
        pin::Pin,
    },
    byteorder::{
        NetworkEndian,
        ReadBytesExt as _,
        WriteBytesExt as _,
    },
    fallible_collections::{
        FallibleBox,
        FallibleVec,
    },
    tokio::io::{
        AsyncRead,
        AsyncReadExt as _,
        AsyncWrite,
        AsyncWriteExt as _,
    },
    async_proto_derive::impl_protocol_for,
    crate::{
        ErrorContext,
        LengthPrefixed,
        Protocol,
        ReadError,
        ReadErrorKind,
        WriteError,
        WriteErrorKind,
    },
};

#[cfg(feature = "bitvec")] mod bitvec;
#[cfg(feature = "bytes")] mod bytes;
#[cfg(feature = "chrono")] mod chrono;
#[cfg(feature = "chrono-tz")] mod chrono_tz;
#[cfg(feature = "doubloon")] mod doubloon;
#[cfg(feature = "either")] mod either;
#[cfg(feature = "enumset")] mod enumset;
#[cfg(feature = "git2")] mod git2;
#[cfg(feature = "gix-hash")] mod gix_hash;
#[cfg(feature = "noisy_float")] mod noisy_float;
#[cfg(feature = "rust_decimal")] mod rust_decimal;
#[cfg(feature = "semver")] mod semver;
#[cfg(feature = "serde_json")] mod serde_json;
#[cfg(feature = "serenity")] mod serenity;
#[cfg(feature = "uuid")] mod uuid;

async fn read_len<'a, R: AsyncRead + Unpin + Send + 'a>(stream: &'a mut R, max_len: u64, error_ctx: impl Fn() -> ErrorContext) -> Result<usize, ReadError> {
    let len = match max_len {
        0 => 0,
        1..=255 => u8::read(stream).await?.into(),
        256..=65_535 => u16::read(stream).await?.into(),
        65_536..=4_294_967_295 => u32::read(stream).await?.into(),
        _ => u64::read(stream).await?,
    };
    if len > max_len {
        Err(ReadError {
            context: error_ctx(),
            kind: ReadErrorKind::MaxLen { len, max_len },
        })
    } else {
        usize::try_from(len).map_err(|e| ReadError {
            context: error_ctx(),
            kind: e.into(),
        })
    }
}

async fn write_len<'a, W: AsyncWrite + Unpin + Send + 'a>(sink: &'a mut W, len: usize, max_len: u64, error_ctx: impl Fn() -> ErrorContext) -> Result<(), WriteError> {
    let len = u64::try_from(len).map_err(|e| WriteError {
        context: error_ctx(),
        kind: e.into(),
    })?;
    if len > max_len {
        return Err(WriteError {
            context: error_ctx(),
            kind: WriteErrorKind::MaxLen { len, max_len },
        })
    }
    match max_len {
        0 => {}
        1..=255 => (len as u8).write(sink).await?,
        256..=65_535 => (len as u16).write(sink).await?,
        65_536..=4_294_967_295 => (len as u32).write(sink).await?,
        _ => len.write(sink).await?,
    }
    Ok(())
}

fn read_len_sync(stream: &mut impl Read, max_len: u64, error_ctx: impl Fn() -> ErrorContext) -> Result<usize, ReadError> {
    let len = match max_len {
        0 => 0,
        1..=255 => u8::read_sync(stream)?.into(),
        256..=65_535 => u16::read_sync(stream)?.into(),
        65_536..=4_294_967_295 => u32::read_sync(stream)?.into(),
        _ => u64::read_sync(stream)?,
    };
    if len > max_len {
        Err(ReadError {
            context: error_ctx(),
            kind: ReadErrorKind::MaxLen { len, max_len },
        })
    } else {
        usize::try_from(len).map_err(|e| ReadError {
            context: error_ctx(),
            kind: e.into(),
        })
    }
}

fn write_len_sync(sink: &mut impl Write, len: usize, max_len: u64, error_ctx: impl Fn() -> ErrorContext) -> Result<(), WriteError> {
    let len = u64::try_from(len).map_err(|e| WriteError {
        context: error_ctx(),
        kind: e.into(),
    })?;
    if len > max_len {
        return Err(WriteError {
            context: error_ctx(),
            kind: WriteErrorKind::MaxLen { len, max_len },
        })
    }
    match max_len {
        0 => {}
        1..=255 => (len as u8).write_sync(sink)?,
        256..=65_535 => (len as u16).write_sync(sink)?,
        65_536..=4_294_967_295 => (len as u32).write_sync(sink)?,
        _ => len.write_sync(sink)?,
    }
    Ok(())
}

macro_rules! impl_protocol_primitive {
    ($ty:ty, $read:ident, $write:ident$(, $endian:ty)?) => {
        /// Primitive number types are encoded in [big-endian](https://en.wikipedia.org/wiki/Big-endian) format.
        impl Protocol for $ty {
            fn read<'a, R: AsyncRead + Unpin + Send + 'a>(stream: &'a mut R) -> Pin<Box<dyn Future<Output = Result<Self, ReadError>> + Send + 'a>> {
                Box::pin(async move {
                    Ok(stream.$read().await.map_err(|e| ReadError {
                        context: ErrorContext::BuiltIn { for_type: stringify!($ty) },
                        kind: e.into(),
                    })?)
                })
            }

            fn write<'a, W: AsyncWrite + Unpin + Send + 'a>(&'a self, sink: &'a mut W) -> Pin<Box<dyn Future<Output = Result<(), WriteError>> + Send + 'a>> {
                Box::pin(async move {
                    Ok(sink.$write(*self).await.map_err(|e| WriteError {
                        context: ErrorContext::BuiltIn { for_type: stringify!($ty) },
                        kind: e.into(),
                    })?)
                })
            }

            fn read_sync(stream: &mut impl Read) -> Result<Self, ReadError> {
                Ok(stream.$read$(::<$endian>)?().map_err(|e| ReadError {
                    context: ErrorContext::BuiltIn { for_type: stringify!($ty) },
                    kind: e.into(),
                })?)
            }

            fn write_sync(&self, sink: &mut impl Write) -> Result<(), WriteError> {
                Ok(sink.$write$(::<$endian>)?(*self).map_err(|e| WriteError {
                    context: ErrorContext::BuiltIn { for_type: stringify!($ty) },
                    kind: e.into(),
                })?)
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

impl<Idx: Protocol + Send + Sync> Protocol for RangeInclusive<Idx> { //TODO derive
    fn read<'a, R: AsyncRead + Unpin + Send + 'a>(stream: &'a mut R) -> Pin<Box<dyn Future<Output = Result<Self, ReadError>> + Send + 'a>> {
        Box::pin(async move {
            Ok(Idx::read(stream).await?..=Idx::read(stream).await?)
        })
    }

    fn write<'a, W: AsyncWrite + Unpin + Send + 'a>(&'a self, sink: &'a mut W) -> Pin<Box<dyn Future<Output = Result<(), WriteError>> + Send + 'a>> {
        Box::pin(async move {
            self.start().write(sink).await?;
            self.end().write(sink).await?;
            Ok(())
        })
    }

    fn read_sync(stream: &mut impl Read) -> Result<Self, ReadError> {
        Ok(Idx::read_sync(stream)?..=Idx::read_sync(stream)?)
    }

    fn write_sync(&self, sink: &mut impl Write) -> Result<(), WriteError> {
        self.start().write_sync(sink)?;
        self.end().write_sync(sink)?;
        Ok(())
    }
}

macro_rules! impl_protocol_tuple {
    ($($ty:ident),*) => {
        #[allow(unused)]
        impl<$($ty: Protocol + Send + Sync),*> Protocol for ($($ty,)*) {
            fn read<'a, R: AsyncRead + Unpin + Send + 'a>(stream: &'a mut R) -> Pin<Box<dyn Future<Output = Result<Self, ReadError>> + Send + 'a>> {
                Box::pin(async move {
                    Ok((
                        $($ty::read(stream).await?,)*
                    ))
                })
            }

            #[allow(non_snake_case)]
            fn write<'a, W: AsyncWrite + Unpin + Send + 'a>(&'a self, sink: &'a mut W) -> Pin<Box<dyn Future<Output = Result<(), WriteError>> + Send + 'a>> {
                Box::pin(async move {
                    let ($($ty,)*) = self;
                    $(
                        $ty.write(sink).await?;
                    )*
                    Ok(())
                })
            }

            fn read_sync(stream: &mut impl Read) -> Result<Self, ReadError> {
                Ok((
                    $($ty::read_sync(stream)?,)*
                ))
            }

            #[allow(non_snake_case)]
            fn write_sync(&self, sink: &mut impl Write) -> Result<(), WriteError> {
                let ($($ty,)*) = self;
                $(
                    $ty.write_sync(sink)?;
                )*
                Ok(())
            }
        }
    };
}

impl_protocol_tuple!();
impl_protocol_tuple!(A);
impl_protocol_tuple!(A, B);
impl_protocol_tuple!(A, B, C);
impl_protocol_tuple!(A, B, C, D);
impl_protocol_tuple!(A, B, C, D, E);
impl_protocol_tuple!(A, B, C, D, E, F);
impl_protocol_tuple!(A, B, C, D, E, F, G);
impl_protocol_tuple!(A, B, C, D, E, F, G, H);
impl_protocol_tuple!(A, B, C, D, E, F, G, H, I);
impl_protocol_tuple!(A, B, C, D, E, F, G, H, I, J);
impl_protocol_tuple!(A, B, C, D, E, F, G, H, I, J, K);
impl_protocol_tuple!(A, B, C, D, E, F, G, H, I, J, K, L);

impl<T: Protocol + Send + Sync, const N: usize> Protocol for [T; N] {
    fn read<'a, R: AsyncRead + Unpin + Send + 'a>(stream: &'a mut R) -> Pin<Box<dyn Future<Output = Result<Self, ReadError>> + Send + 'a>> {
        Box::pin(async move {
            let mut vec = <Vec<_> as FallibleVec<_>>::try_with_capacity(N).map_err(|e| ReadError {
                context: ErrorContext::BuiltIn { for_type: "[T; N]" },
                kind: e.into(),
            })?;
            for _ in 0..N {
                vec.push(T::read(stream).await?);
            }
            Ok(match vec.try_into() {
                Ok(array) => array,
                Err(_) => panic!("wrong array length"),
            })
        })
    }

    fn write<'a, W: AsyncWrite + Unpin + Send + 'a>(&'a self, sink: &'a mut W) -> Pin<Box<dyn Future<Output = Result<(), WriteError>> + Send + 'a>> {
        Box::pin(async move {
            for elt in self {
                elt.write(sink).await?;
            }
            Ok(())
        })
    }

    fn read_sync(stream: &mut impl Read) -> Result<Self, ReadError> {
        let mut vec = <Vec<_> as FallibleVec<_>>::try_with_capacity(N).map_err(|e| ReadError {
            context: ErrorContext::BuiltIn { for_type: "[T; N]" },
            kind: e.into(),
        })?;
        for _ in 0..N {
            vec.push(T::read_sync(stream)?);
        }
        Ok(match vec.try_into() {
            Ok(array) => array,
            Err(_) => panic!("wrong array length"),
        })
    }

    fn write_sync(&self, sink: &mut impl Write) -> Result<(), WriteError> {
        for elt in self {
            elt.write_sync(sink)?;
        }
        Ok(())
    }
}

/// Represented as one byte, with `0` for `false` and `1` for `true`.
impl Protocol for bool {
    fn read<'a, R: AsyncRead + Unpin + Send + 'a>(stream: &'a mut R) -> Pin<Box<dyn Future<Output = Result<Self, ReadError>> + Send + 'a>> {
        Box::pin(async move {
            Ok(match u8::read(stream).await? {
                0 => false,
                1 => true,
                n => return Err(ReadError {
                    context: ErrorContext::BuiltIn { for_type: "bool" },
                    kind: ReadErrorKind::UnknownVariant8(n),
                }),
            })
        })
    }

    fn write<'a, W: AsyncWrite + Unpin + Send + 'a>(&'a self, sink: &'a mut W) -> Pin<Box<dyn Future<Output = Result<(), WriteError>> + Send + 'a>> {
        Box::pin(async move {
            if *self { 1u8 } else { 0 }.write(sink).await
        })
    }

    fn read_sync(stream: &mut impl Read) -> Result<Self, ReadError> {
        Ok(match u8::read_sync(stream)? {
            0 => false,
            1 => true,
            n => return Err(ReadError {
                context: ErrorContext::BuiltIn { for_type: "bool" },
                kind: ReadErrorKind::UnknownVariant8(n),
            }),
        })
    }

    fn write_sync(&self, sink: &mut impl Write) -> Result<(), WriteError> {
        if *self { 1u8 } else { 0 }.write_sync(sink)
    }
}

impl<T: Protocol> Protocol for Box<T> {
    fn read<'a, R: AsyncRead + Unpin + Send + 'a>(stream: &'a mut R) -> Pin<Box<dyn Future<Output = Result<Self, ReadError>> + Send + 'a>> {
        Box::pin(async move {
            Ok(<Box<_> as FallibleBox<_>>::try_new(T::read(stream).await?).map_err(|e| ReadError {
                context: ErrorContext::BuiltIn { for_type: "Box" },
                kind: e.into(),
            })?)
        })
    }

    fn write<'a, W: AsyncWrite + Unpin + Send + 'a>(&'a self, sink: &'a mut W) -> Pin<Box<dyn Future<Output = Result<(), WriteError>> + Send + 'a>> {
        (**self).write(sink)
    }

    fn read_sync(stream: &mut impl Read) -> Result<Self, ReadError> {
        Ok(<Box<_> as FallibleBox<_>>::try_new(T::read_sync(stream)?).map_err(|e| ReadError {
            context: ErrorContext::BuiltIn { for_type: "Box" },
            kind: e.into(),
        })?)
    }

    fn write_sync(&self, sink: &mut impl Write) -> Result<(), WriteError> {
        (**self).write_sync(sink)
    }
}

/// A vector is prefixed with the length as a [`u64`].
///
/// Note that due to Rust's lack of [specialization](https://github.com/rust-lang/rust/issues/31844), this implementation is inefficient for `Vec<u8>`.
/// Prefer [`Bytes`](https://docs.rs/bytes/latest/bytes/struct.Bytes.html) if possible.
impl<T: Protocol + Send + Sync> Protocol for Vec<T> {
        fn read<'a, R: AsyncRead + Unpin + Send + 'a>(stream: &'a mut R) -> Pin<Box<dyn Future<Output = Result<Self, ReadError>> + Send + 'a>> {
        Self::read_length_prefixed(stream, u64::MAX)
    }

    fn write<'a, W: AsyncWrite + Unpin + Send + 'a>(&'a self, sink: &'a mut W) -> Pin<Box<dyn Future<Output = Result<(), WriteError>> + Send + 'a>> {
        self.write_length_prefixed(sink, u64::MAX)
    }

    fn read_sync(stream: &mut impl Read) -> Result<Self, ReadError> {
        Self::read_length_prefixed_sync(stream, u64::MAX)
    }

    fn write_sync(&self, sink: &mut impl Write) -> Result<(), WriteError> {
        self.write_length_prefixed_sync(sink, u64::MAX)
    }
}

/// Note that due to Rust's lack of [specialization](https://github.com/rust-lang/rust/issues/31844), this implementation is inefficient for `Vec<u8>`.
/// Prefer [`Bytes`](https://docs.rs/bytes/latest/bytes/struct.Bytes.html) if possible.
impl<T: Protocol + Send + Sync> LengthPrefixed for Vec<T> {
    fn read_length_prefixed<'a, R: AsyncRead + Unpin + Send + 'a>(stream: &'a mut R, max_len: u64) -> Pin<Box<dyn Future<Output = Result<Self, ReadError>> + Send + 'a>> {
        Box::pin(async move {
            let len = read_len(stream, max_len, || ErrorContext::BuiltIn { for_type: "Vec" }).await?;
            let mut buf = <Self as FallibleVec<_>>::try_with_capacity(len).map_err(|e| ReadError {
                context: ErrorContext::BuiltIn { for_type: "Vec" },
                kind: e.into(),
            })?;
            for _ in 0..len {
                buf.push(T::read(stream).await?);
            }
            Ok(buf)
        })
    }

    fn write_length_prefixed<'a, W: AsyncWrite + Unpin + Send + 'a>(&'a self, sink: &'a mut W, max_len: u64) -> Pin<Box<dyn Future<Output = Result<(), WriteError>> + Send + 'a>> {
        Box::pin(async move {
            write_len(sink, self.len(), max_len, || ErrorContext::BuiltIn { for_type: "Vec" }).await?;
            for elt in self {
                elt.write(sink).await?;
            }
            Ok(())
        })
    }

    fn read_length_prefixed_sync(stream: &mut impl Read, max_len: u64) -> Result<Self, ReadError> {
        let len = read_len_sync(stream, max_len, || ErrorContext::BuiltIn { for_type: "Vec" })?;
        let mut buf = <Self as FallibleVec<_>>::try_with_capacity(len).map_err(|e| ReadError {
            context: ErrorContext::BuiltIn { for_type: "Vec" },
            kind: e.into(),
        })?;
        for _ in 0..len {
            buf.push(T::read_sync(stream)?);
        }
        Ok(buf)
    }

    fn write_length_prefixed_sync(&self, sink: &mut impl Write, max_len: u64) -> Result<(), WriteError> {
        write_len_sync(sink, self.len(), max_len, || ErrorContext::BuiltIn { for_type: "Vec" })?;
        for elt in self {
            elt.write_sync(sink)?;
        }
        Ok(())
    }
}

/// A set is prefixed with the length as a [`u64`].
impl<T: Protocol + Ord + Send + Sync + 'static> Protocol for BTreeSet<T> {
        fn read<'a, R: AsyncRead + Unpin + Send + 'a>(stream: &'a mut R) -> Pin<Box<dyn Future<Output = Result<Self, ReadError>> + Send + 'a>> {
        Self::read_length_prefixed(stream, u64::MAX)
    }

    fn write<'a, W: AsyncWrite + Unpin + Send + 'a>(&'a self, sink: &'a mut W) -> Pin<Box<dyn Future<Output = Result<(), WriteError>> + Send + 'a>> {
        self.write_length_prefixed(sink, u64::MAX)
    }

    fn read_sync(stream: &mut impl Read) -> Result<Self, ReadError> {
        Self::read_length_prefixed_sync(stream, u64::MAX)
    }

    fn write_sync(&self, sink: &mut impl Write) -> Result<(), WriteError> {
        self.write_length_prefixed_sync(sink, u64::MAX)
    }
}

impl<T: Protocol + Ord + Send + Sync + 'static> LengthPrefixed for BTreeSet<T> {
    fn read_length_prefixed<'a, R: AsyncRead + Unpin + Send + 'a>(stream: &'a mut R, max_len: u64) -> Pin<Box<dyn Future<Output = Result<Self, ReadError>> + Send + 'a>> {
        Box::pin(async move {
            let len = read_len(stream, max_len, || ErrorContext::BuiltIn { for_type: "BTreeSet" }).await?;
            let mut set = Self::default();
            for _ in 0..len {
                set.insert(T::read(stream).await?); //TODO use fallible allocation once available
            }
            Ok(set)
        })
    }

    fn write_length_prefixed<'a, W: AsyncWrite + Unpin + Send + 'a>(&'a self, sink: &'a mut W, max_len: u64) -> Pin<Box<dyn Future<Output = Result<(), WriteError>> + Send + 'a>> {
        Box::pin(async move {
            write_len(sink, self.len(), max_len, || ErrorContext::BuiltIn { for_type: "BTreeSet" }).await?;
            for elt in self {
                elt.write(sink).await?;
            }
            Ok(())
        })
    }

    fn read_length_prefixed_sync(stream: &mut impl Read, max_len: u64) -> Result<Self, ReadError> {
        let len = read_len_sync(stream, max_len, || ErrorContext::BuiltIn { for_type: "BTreeSet" })?;        let mut set = Self::default();
        for _ in 0..len {
            set.insert(T::read_sync(stream)?); //TODO use fallible allocation once available
        }
        Ok(set)
    }

    fn write_length_prefixed_sync(&self, sink: &mut impl Write, max_len: u64) -> Result<(), WriteError> {
        write_len_sync(sink, self.len(), max_len, || ErrorContext::BuiltIn { for_type: "BTreeSet" })?;
        for elt in self {
            elt.write_sync(sink)?;
        }
        Ok(())
    }
}

impl<T: Protocol + Eq + Hash + Send + Sync> Protocol for HashSet<T> {
    fn read<'a, R: AsyncRead + Unpin + Send + 'a>(stream: &'a mut R) -> Pin<Box<dyn Future<Output = Result<Self, ReadError>> + Send + 'a>> {
        Self::read_length_prefixed(stream, u64::MAX)
    }

    fn write<'a, W: AsyncWrite + Unpin + Send + 'a>(&'a self, sink: &'a mut W) -> Pin<Box<dyn Future<Output = Result<(), WriteError>> + Send + 'a>> {
        self.write_length_prefixed(sink, u64::MAX)
    }

    fn read_sync(stream: &mut impl Read) -> Result<Self, ReadError> {
        Self::read_length_prefixed_sync(stream, u64::MAX)
    }

    fn write_sync(&self, sink: &mut impl Write) -> Result<(), WriteError> {
        self.write_length_prefixed_sync(sink, u64::MAX)
    }
}

impl<T: Protocol + Eq + Hash + Send + Sync> LengthPrefixed for HashSet<T> {
    fn read_length_prefixed<'a, R: AsyncRead + Unpin + Send + 'a>(stream: &'a mut R, max_len: u64) -> Pin<Box<dyn Future<Output = Result<Self, ReadError>> + Send + 'a>> {
        Box::pin(async move {
            let len = read_len(stream, max_len, || ErrorContext::BuiltIn { for_type: "HashSet" }).await?;
            let mut set = Self::with_capacity(len); //TODO use fallible allocation once available
            for _ in 0..len {
                set.insert(T::read(stream).await?);
            }
            Ok(set)
        })
    }

    fn write_length_prefixed<'a, W: AsyncWrite + Unpin + Send + 'a>(&'a self, sink: &'a mut W, max_len: u64) -> Pin<Box<dyn Future<Output = Result<(), WriteError>> + Send + 'a>> {
        Box::pin(async move {
            write_len(sink, self.len(), max_len, || ErrorContext::BuiltIn { for_type: "HashSet" }).await?;
            for elt in self {
                elt.write(sink).await?;
            }
            Ok(())
        })
    }

    fn read_length_prefixed_sync(stream: &mut impl Read, max_len: u64) -> Result<Self, ReadError> {
        let len = read_len_sync(stream, max_len, || ErrorContext::BuiltIn { for_type: "HashSet" })?;
        let mut set = Self::with_capacity(len); //TODO use fallible allocation once available
        for _ in 0..len {
            set.insert(T::read_sync(stream)?);
        }
        Ok(set)
    }

    fn write_length_prefixed_sync(&self, sink: &mut impl Write, max_len: u64) -> Result<(), WriteError> {
        write_len_sync(sink, self.len(), max_len, || ErrorContext::BuiltIn { for_type: "HashSet" })?;
        for elt in self {
            elt.write_sync(sink)?;
        }
        Ok(())
    }
}

/// A string is encoded in UTF-8 and prefixed with the length in bytes as a [`u64`].
impl Protocol for String {
    fn read<'a, R: AsyncRead + Unpin + Send + 'a>(stream: &'a mut R) -> Pin<Box<dyn Future<Output = Result<Self, ReadError>> + Send + 'a>> {
        Self::read_length_prefixed(stream, u64::MAX)
    }

    fn write<'a, W: AsyncWrite + Unpin + Send + 'a>(&'a self, sink: &'a mut W) -> Pin<Box<dyn Future<Output = Result<(), WriteError>> + Send + 'a>> {
        self.write_length_prefixed(sink, u64::MAX)
    }

    fn read_sync(stream: &mut impl Read) -> Result<Self, ReadError> {
        Self::read_length_prefixed_sync(stream, u64::MAX)
    }

    fn write_sync(&self, sink: &mut impl Write) -> Result<(), WriteError> {
        self.write_length_prefixed_sync(sink, u64::MAX)
    }
}

/// A string is encoded in UTF-8 and prefixed with the length in bytes.
impl LengthPrefixed for String {
    fn read_length_prefixed<'a, R: AsyncRead + Unpin + Send + 'a>(stream: &'a mut R, max_len: u64) -> Pin<Box<dyn Future<Output = Result<Self, ReadError>> + Send + 'a>> {
        Box::pin(async move {
            let len = read_len(stream, max_len, || ErrorContext::BuiltIn { for_type: "String" }).await?;
            let mut buf = Vec::default();
            buf.try_resize(len, 0).map_err(|e| ReadError {
                context: ErrorContext::BuiltIn { for_type: "String" },
                kind: e.into(),
            })?;
            stream.read_exact(&mut buf).await.map_err(|e| ReadError {
                context: ErrorContext::BuiltIn { for_type: "String" },
                kind: e.into(),
            })?;
            Ok(Self::from_utf8(buf).map_err(|e| ReadError {
                context: ErrorContext::BuiltIn { for_type: "String" },
                kind: e.into(),
            })?)
        })
    }

    fn write_length_prefixed<'a, W: AsyncWrite + Unpin + Send + 'a>(&'a self, sink: &'a mut W, max_len: u64) -> Pin<Box<dyn Future<Output = Result<(), WriteError>> + Send + 'a>> {
        Box::pin(async move {
            write_len(sink, self.len(), max_len, || ErrorContext::BuiltIn { for_type: "String" }).await?;
            sink.write(self.as_bytes()).await.map_err(|e| WriteError {
                context: ErrorContext::BuiltIn { for_type: "String" },
                kind: e.into(),
            })?;
            Ok(())
        })
    }

    fn read_length_prefixed_sync(stream: &mut impl Read, max_len: u64) -> Result<Self, ReadError> {
        let len = read_len_sync(stream, max_len, || ErrorContext::BuiltIn { for_type: "String" })?;
        let mut buf = Vec::default();
        buf.try_resize(len, 0).map_err(|e| ReadError {
            context: ErrorContext::BuiltIn { for_type: "String" },
            kind: e.into(),
        })?;
        stream.read_exact(&mut buf).map_err(|e| ReadError {
            context: ErrorContext::BuiltIn { for_type: "String" },
            kind: e.into(),
        })?;
        Ok(Self::from_utf8(buf).map_err(|e| ReadError {
            context: ErrorContext::BuiltIn { for_type: "String" },
            kind: e.into(),
        })?)
    }

    fn write_length_prefixed_sync(&self, sink: &mut impl Write, max_len: u64) -> Result<(), WriteError> {
        write_len_sync(sink, self.len(), max_len, || ErrorContext::BuiltIn { for_type: "String" })?;
        sink.write(self.as_bytes()).map_err(|e| WriteError {
            context: ErrorContext::BuiltIn { for_type: "String" },
            kind: e.into(),
        })?;
        Ok(())
    }
}

impl<K: Protocol + Ord + Send + Sync + 'static, V: Protocol + Send + Sync + 'static> Protocol for BTreeMap<K, V> {
    fn read<'a, R: AsyncRead + Unpin + Send + 'a>(stream: &'a mut R) -> Pin<Box<dyn Future<Output = Result<Self, ReadError>> + Send + 'a>> {
        Self::read_length_prefixed(stream, u64::MAX)
    }

    fn write<'a, W: AsyncWrite + Unpin + Send + 'a>(&'a self, sink: &'a mut W) -> Pin<Box<dyn Future<Output = Result<(), WriteError>> + Send + 'a>> {
        self.write_length_prefixed(sink, u64::MAX)
    }

    fn read_sync(stream: &mut impl Read) -> Result<Self, ReadError> {
        Self::read_length_prefixed_sync(stream, u64::MAX)
    }

    fn write_sync(&self, sink: &mut impl Write) -> Result<(), WriteError> {
        self.write_length_prefixed_sync(sink, u64::MAX)
    }
}

impl<K: Protocol + Ord + Send + Sync + 'static, V: Protocol + Send + Sync + 'static> LengthPrefixed for BTreeMap<K, V> {
    fn read_length_prefixed<'a, R: AsyncRead + Unpin + Send + 'a>(stream: &'a mut R, max_len: u64) -> Pin<Box<dyn Future<Output = Result<Self, ReadError>> + Send + 'a>> {
        Box::pin(async move {
            let len = read_len(stream, max_len, || ErrorContext::BuiltIn { for_type: "BTreeMap" }).await?;
            let mut map = Self::default();
            for _ in 0..len {
                map.insert(K::read(stream).await?, V::read(stream).await?); //TODO use fallible allocation once available
            }
            Ok(map)
        })
    }

    fn write_length_prefixed<'a, W: AsyncWrite + Unpin + Send + 'a>(&'a self, sink: &'a mut W, max_len: u64) -> Pin<Box<dyn Future<Output = Result<(), WriteError>> + Send + 'a>> {
        Box::pin(async move {
            write_len(sink, self.len(), max_len, || ErrorContext::BuiltIn { for_type: "BTreeMap" }).await?;
            for (k, v) in self {
                k.write(sink).await?;
                v.write(sink).await?;
            }
            Ok(())
        })
    }

    fn read_length_prefixed_sync(stream: &mut impl Read, max_len: u64) -> Result<Self, ReadError> {
        let len = read_len_sync(stream, max_len, || ErrorContext::BuiltIn { for_type: "BTreeMap" })?;
        let mut map = Self::default();
        for _ in 0..len {
            map.insert(K::read_sync(stream)?, V::read_sync(stream)?); //TODO use fallible allocation once available
        }
        Ok(map)
    }

    fn write_length_prefixed_sync(&self, sink: &mut impl Write, max_len: u64) -> Result<(), WriteError> {
        write_len_sync(sink, self.len(), max_len, || ErrorContext::BuiltIn { for_type: "BTreeMap" })?;
        for (k, v) in self {
            k.write_sync(sink)?;
            v.write_sync(sink)?;
        }
        Ok(())
    }
}

/// A map is prefixed with the length as a [`u64`].
impl<K: Protocol + Eq + Hash + Send + Sync, V: Protocol + Send + Sync> Protocol for HashMap<K, V> {
    fn read<'a, R: AsyncRead + Unpin + Send + 'a>(stream: &'a mut R) -> Pin<Box<dyn Future<Output = Result<Self, ReadError>> + Send + 'a>> {
        Self::read_length_prefixed(stream, u64::MAX)
    }

    fn write<'a, W: AsyncWrite + Unpin + Send + 'a>(&'a self, sink: &'a mut W) -> Pin<Box<dyn Future<Output = Result<(), WriteError>> + Send + 'a>> {
        self.write_length_prefixed(sink, u64::MAX)
    }

    fn read_sync(stream: &mut impl Read) -> Result<Self, ReadError> {
        Self::read_length_prefixed_sync(stream, u64::MAX)
    }

    fn write_sync(&self, sink: &mut impl Write) -> Result<(), WriteError> {
        self.write_length_prefixed_sync(sink, u64::MAX)
    }
}

impl<K: Protocol + Eq + Hash + Send + Sync, V: Protocol + Send + Sync> LengthPrefixed for HashMap<K, V> {
    fn read_length_prefixed<'a, R: AsyncRead + Unpin + Send + 'a>(stream: &'a mut R, max_len: u64) -> Pin<Box<dyn Future<Output = Result<Self, ReadError>> + Send + 'a>> {
        Box::pin(async move {
            let len = read_len(stream, max_len, || ErrorContext::BuiltIn { for_type: "HashMap" }).await?;
            let mut map = Self::with_capacity(len); //TODO use fallible allocation once available
            for _ in 0..len {
                map.insert(K::read(stream).await?, V::read(stream).await?);
            }
            Ok(map)
        })
    }

    fn write_length_prefixed<'a, W: AsyncWrite + Unpin + Send + 'a>(&'a self, sink: &'a mut W, max_len: u64) -> Pin<Box<dyn Future<Output = Result<(), WriteError>> + Send + 'a>> {
        Box::pin(async move {
            write_len(sink, self.len(), max_len, || ErrorContext::BuiltIn { for_type: "HashMap" }).await?;
            for (k, v) in self {
                k.write(sink).await?;
                v.write(sink).await?;
            }
            Ok(())
        })
    }

    fn read_length_prefixed_sync(stream: &mut impl Read, max_len: u64) -> Result<Self, ReadError> {
        let len = read_len_sync(stream, max_len, || ErrorContext::BuiltIn { for_type: "HashMap" })?;
        let mut map = Self::with_capacity(len); //TODO use fallible allocation once available
        for _ in 0..len {
            map.insert(K::read_sync(stream)?, V::read_sync(stream)?);
        }
        Ok(map)
    }

    fn write_length_prefixed_sync(&self, sink: &mut impl Write, max_len: u64) -> Result<(), WriteError> {
        write_len_sync(sink, self.len(), max_len, || ErrorContext::BuiltIn { for_type: "HashMap" })?;
        for (k, v) in self {
            k.write_sync(sink)?;
            v.write_sync(sink)?;
        }
        Ok(())
    }
}

/// A cow is represented like its owned variant.
///
/// Note that due to a restriction in the type system, writing a borrowed cow requires cloning it.
impl<'cow, B: ToOwned + Sync + ?Sized> Protocol for std::borrow::Cow<'cow, B>
where B::Owned: Protocol + Send + Sync {
    fn read<'a, R: AsyncRead + Unpin + Send + 'a>(stream: &'a mut R) -> Pin<Box<dyn Future<Output = Result<Self, ReadError>> + Send + 'a>> {
        Box::pin(async move {
            Ok(Self::Owned(B::Owned::read(stream).await?))
        })
    }

    fn write<'a, W: AsyncWrite + Unpin + Send + 'a>(&'a self, sink: &'a mut W) -> Pin<Box<dyn Future<Output = Result<(), WriteError>> + Send + 'a>> {
        Box::pin(async move {
            match self {
                Self::Borrowed(borrowed) => (*borrowed).to_owned().write(sink).await?,
                Self::Owned(owned) => owned.write(sink).await?,
            }
            Ok(())
        })
    }

    fn read_sync(stream: &mut impl Read) -> Result<Self, ReadError> {
        Ok(Self::Owned(B::Owned::read_sync(stream)?))
    }

    fn write_sync(&self, sink: &mut impl Write) -> Result<(), WriteError> {
        match self {
            Self::Borrowed(borrowed) => (*borrowed).to_owned().write_sync(sink)?,
            Self::Owned(owned) => owned.write_sync(sink)?,
        }
        Ok(())
    }
}

/// A cow is represented like its owned variant.
///
/// Note that due to a restriction in the type system, writing a borrowed cow requires cloning it.
impl<'cow, B: ToOwned + Sync + ?Sized> LengthPrefixed for std::borrow::Cow<'cow, B>
where B::Owned: LengthPrefixed + Send + Sync {
    fn read_length_prefixed<'a, R: AsyncRead + Unpin + Send + 'a>(stream: &'a mut R, max_len: u64) -> Pin<Box<dyn Future<Output = Result<Self, ReadError>> + Send + 'a>> {
        Box::pin(async move {
            Ok(Self::Owned(B::Owned::read_length_prefixed(stream, max_len).await?))
        })
    }

    fn write_length_prefixed<'a, W: AsyncWrite + Unpin + Send + 'a>(&'a self, sink: &'a mut W, max_len: u64) -> Pin<Box<dyn Future<Output = Result<(), WriteError>> + Send + 'a>> {
        Box::pin(async move {
            match self {
                Self::Borrowed(borrowed) => (*borrowed).to_owned().write_length_prefixed(sink, max_len).await?,
                Self::Owned(owned) => owned.write_length_prefixed(sink, max_len).await?,
            }
            Ok(())
        })
    }

    fn read_length_prefixed_sync(stream: &mut impl Read, max_len: u64) -> Result<Self, ReadError> {
        Ok(Self::Owned(B::Owned::read_length_prefixed_sync(stream, max_len)?))
    }

    fn write_length_prefixed_sync(&self, sink: &mut impl Write, max_len: u64) -> Result<(), WriteError> {
        match self {
            Self::Borrowed(borrowed) => (*borrowed).to_owned().write_length_prefixed_sync(sink, max_len)?,
            Self::Owned(owned) => owned.write_length_prefixed_sync(sink, max_len)?,
        }
        Ok(())
    }
}

#[derive(Protocol)]
#[async_proto(internal)]
struct F32Proxy([u8; 4]);

impl From<F32Proxy> for f32 {
    fn from(F32Proxy(bytes): F32Proxy) -> Self {
        Self::from_be_bytes(bytes)
    }
}

impl<'a> From<&'a f32> for F32Proxy {
    fn from(val: &f32) -> Self {
        Self(val.to_be_bytes())
    }
}

#[derive(Protocol)]
#[async_proto(internal)]
struct F64Proxy([u8; 8]);

impl From<F64Proxy> for f64 {
    fn from(F64Proxy(bytes): F64Proxy) -> Self {
        Self::from_be_bytes(bytes)
    }
}

impl<'a> From<&'a f64> for F64Proxy {
    fn from(val: &f64) -> Self {
        Self(val.to_be_bytes())
    }
}

#[derive(Protocol)]
#[async_proto(internal)]
struct DurationProxy {
    secs: u64,
    subsec_nanos: u32,
}

impl From<DurationProxy> for std::time::Duration {
    fn from(DurationProxy { secs, subsec_nanos }: DurationProxy) -> Self {
        Self::new(secs, subsec_nanos)
    }
}

impl<'a> From<&'a std::time::Duration> for DurationProxy {
    fn from(duration: &std::time::Duration) -> Self {
        Self {
            secs: duration.as_secs(),
            subsec_nanos: duration.subsec_nanos(),
        }
    }
}

impl_protocol_for! {
    #[async_proto(attr(doc = "A nonzero integer is represented like its value."))]
    #[async_proto(via = u8, clone, map_err = |_| ReadErrorKind::UnknownVariant8(0))]
    type std::num::NonZeroU8;

    #[async_proto(attr(doc = "A nonzero integer is represented like its value."))]
    #[async_proto(via = i8, clone, map_err = |_| ReadErrorKind::UnknownVariant8(0))]
    type std::num::NonZeroI8;

    #[async_proto(attr(doc = "A nonzero integer is represented like its value."))]
    #[async_proto(via = u16, clone, map_err = |_| ReadErrorKind::UnknownVariant16(0))]
    type std::num::NonZeroU16;

    #[async_proto(attr(doc = "A nonzero integer is represented like its value."))]
    #[async_proto(via = i16, clone, map_err = |_| ReadErrorKind::UnknownVariant16(0))]
    type std::num::NonZeroI16;

    #[async_proto(attr(doc = "A nonzero integer is represented like its value."))]
    #[async_proto(via = u32, clone, map_err = |_| ReadErrorKind::UnknownVariant32(0))]
    type std::num::NonZeroU32;

    #[async_proto(attr(doc = "A nonzero integer is represented like its value."))]
    #[async_proto(via = i32, clone, map_err = |_| ReadErrorKind::UnknownVariant32(0))]
    type std::num::NonZeroI32;

    #[async_proto(attr(doc = "A nonzero integer is represented like its value."))]
    #[async_proto(via = u64, clone, map_err = |_| ReadErrorKind::UnknownVariant64(0))]
    type std::num::NonZeroU64;

    #[async_proto(attr(doc = "A nonzero integer is represented like its value."))]
    #[async_proto(via = i64, clone, map_err = |_| ReadErrorKind::UnknownVariant64(0))]
    type std::num::NonZeroI64;

    #[async_proto(attr(doc = "A nonzero integer is represented like its value."))]
    #[async_proto(via = u128, clone, map_err = |_| ReadErrorKind::UnknownVariant128(0))]
    type std::num::NonZeroU128;

    #[async_proto(attr(doc = "A nonzero integer is represented like its value."))]
    #[async_proto(via = i128, clone, map_err = |_| ReadErrorKind::UnknownVariant128(0))]
    type std::num::NonZeroI128;

    #[async_proto(attr(doc = "Primitive number types are encoded in [big-endian](https://en.wikipedia.org/wiki/Big-endian) format."))]
    #[async_proto(via = F32Proxy)]
    type f32;

    #[async_proto(attr(doc = "Primitive number types are encoded in [big-endian](https://en.wikipedia.org/wiki/Big-endian) format."))]
    #[async_proto(via = F64Proxy)]
    type f64;

    #[async_proto(where(Idx: Protocol + Send + Sync))]
    struct Range<Idx> {
        start: Idx,
        end: Idx,
    }

    #[async_proto(where(Idx: Protocol + Sync))]
    struct RangeFrom<Idx> {
        start: Idx,
    }

    #[async_proto(where(Idx: Protocol + Sync))]
    struct RangeTo<Idx> {
        end: Idx,
    }

    #[async_proto(where(Idx: Protocol + Sync))]
    struct RangeToInclusive<Idx> {
        end: Idx,
    }

    #[async_proto(where(T: Protocol + Sync))]
    enum Option<T> {
        None,
        Some(T),
    }

    #[async_proto(where(T: Protocol + Sync, E: Protocol + Sync))]
    enum Result<T, E> {
        Ok(T),
        Err(E),
    }

    enum std::convert::Infallible {}

    #[async_proto(where(T: Sync))]
    struct std::marker::PhantomData<T>;

    struct std::ops::RangeFull;

    #[async_proto(attr(doc = "A duration is represented as the number of whole seconds as a [`u64`] followed by the number of subsecond nanoseconds as a [`u32`]."))]
    #[async_proto(via = DurationProxy)]
    type std::time::Duration;
}
