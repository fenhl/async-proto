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
    tokio::io::{
        AsyncRead,
        AsyncReadExt as _,
        AsyncWrite,
        AsyncWriteExt as _,
    },
    async_proto_derive::impl_protocol_for,
    crate::{
        Protocol,
        ReadError,
        WriteError,
    },
};

#[cfg(feature = "bytes")] mod bytes;
#[cfg(feature = "chrono")] mod chrono;
#[cfg(feature = "chrono-tz")] mod chrono_tz;
#[cfg(feature = "noisy_float")] mod noisy_float;
#[cfg(feature = "serde_json")] mod serde_json;

macro_rules! impl_protocol_primitive {
    ($ty:ty, $read:ident, $write:ident$(, $endian:ty)?) => {
        /// Primitive number types are encoded in [big-endian](https://en.wikipedia.org/wiki/Big-endian) format.
        impl Protocol for $ty {
            fn read<'a, R: AsyncRead + Unpin + Send + 'a>(stream: &'a mut R) -> Pin<Box<dyn Future<Output = Result<Self, ReadError>> + Send + 'a>> {
                Box::pin(async move {
                    Ok(stream.$read().await?)
                })
            }

            fn write<'a, W: AsyncWrite + Unpin + Send + 'a>(&'a self, sink: &'a mut W) -> Pin<Box<dyn Future<Output = Result<(), WriteError>> + Send + 'a>> {
                Box::pin(async move {
                    Ok(sink.$write(*self).await?)
                })
            }

            fn read_sync(stream: &mut impl Read) -> Result<Self, ReadError> {
                Ok(stream.$read$(::<$endian>)?()?)
            }

            fn write_sync(&self, sink: &mut impl Write) -> Result<(), WriteError> {
                Ok(sink.$write$(::<$endian>)?(*self)?)
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
            let mut vec = Vec::with_capacity(N);
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
        let mut vec = Vec::with_capacity(N);
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
                n => return Err(ReadError::UnknownVariant8(n)),
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
            n => return Err(ReadError::UnknownVariant8(n)),
        })
    }

    fn write_sync(&self, sink: &mut impl Write) -> Result<(), WriteError> {
        if *self { 1u8 } else { 0 }.write_sync(sink)
    }
}

impl<T: Protocol> Protocol for Box<T> {
    fn read<'a, R: AsyncRead + Unpin + Send + 'a>(stream: &'a mut R) -> Pin<Box<dyn Future<Output = Result<Self, ReadError>> + Send + 'a>> {
        Box::pin(async move {
            Ok(T::read(stream).await?.into()) //TODO use try_new once stabilized
        })
    }

    fn write<'a, W: AsyncWrite + Unpin + Send + 'a>(&'a self, sink: &'a mut W) -> Pin<Box<dyn Future<Output = Result<(), WriteError>> + Send + 'a>> {
        (**self).write(sink)
    }

    fn read_sync(stream: &mut impl Read) -> Result<Self, ReadError> {
        Ok(T::read_sync(stream)?.into()) //TODO use try_new once stabilized
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
        Box::pin(async move {
            let len = u64::read(stream).await?;
            let mut buf = Self::with_capacity(len.try_into()?);
            for _ in 0..len {
                buf.push(T::read(stream).await?);
            }
            Ok(buf)
        })
    }

    fn write<'a, W: AsyncWrite + Unpin + Send + 'a>(&'a self, sink: &'a mut W) -> Pin<Box<dyn Future<Output = Result<(), WriteError>> + Send + 'a>> {
        Box::pin(async move {
            u64::try_from(self.len())?.write(sink).await?;
            for elt in self {
                elt.write(sink).await?;
            }
            Ok(())
        })
    }

    fn read_sync(stream: &mut impl Read) -> Result<Self, ReadError> {
        let len = u64::read_sync(stream)?;
        let mut buf = Self::with_capacity(len.try_into()?);
        for _ in 0..len {
            buf.push(T::read_sync(stream)?);
        }
        Ok(buf)
    }

    fn write_sync(&self, sink: &mut impl Write) -> Result<(), WriteError> {
        u64::try_from(self.len())?.write_sync(sink)?;
        for elt in self {
            elt.write_sync(sink)?;
        }
        Ok(())
    }
}

/// A set is prefixed with the length as a [`u64`].
impl<T: Protocol + Ord + Send + Sync + 'static> Protocol for BTreeSet<T> {
    fn read<'a, R: AsyncRead + Unpin + Send + 'a>(stream: &'a mut R) -> Pin<Box<dyn Future<Output = Result<Self, ReadError>> + Send + 'a>> {
        Box::pin(async move {
            let len = u64::read(stream).await?;
            usize::try_from(len)?; // error here rather than panicking in the insert loop
            let mut set = Self::default();
            for _ in 0..len {
                set.insert(T::read(stream).await?);
            }
            Ok(set)
        })
    }

    fn write<'a, W: AsyncWrite + Unpin + Send + 'a>(&'a self, sink: &'a mut W) -> Pin<Box<dyn Future<Output = Result<(), WriteError>> + Send + 'a>> {
        Box::pin(async move {
            u64::try_from(self.len())?.write(sink).await?;
            for elt in self {
                elt.write(sink).await?;
            }
            Ok(())
        })
    }

    fn read_sync(stream: &mut impl Read) -> Result<Self, ReadError> {
        let len = u64::read_sync(stream)?;
        usize::try_from(len)?; // error here rather than panicking in the insert loop
        let mut set = Self::default();
        for _ in 0..len {
            set.insert(T::read_sync(stream)?);
        }
        Ok(set)
    }

    fn write_sync(&self, sink: &mut impl Write) -> Result<(), WriteError> {
        u64::try_from(self.len())?.write_sync(sink)?;
        for elt in self {
            elt.write_sync(sink)?;
        }
        Ok(())
    }
}

/// A set is prefixed with the length as a [`u64`].
impl<T: Protocol + Eq + Hash + Send + Sync> Protocol for HashSet<T> {
    fn read<'a, R: AsyncRead + Unpin + Send + 'a>(stream: &'a mut R) -> Pin<Box<dyn Future<Output = Result<Self, ReadError>> + Send + 'a>> {
        Box::pin(async move {
            let len = u64::read(stream).await?;
            let mut set = Self::with_capacity(len.try_into()?);
            for _ in 0..len {
                set.insert(T::read(stream).await?);
            }
            Ok(set)
        })
    }

    fn write<'a, W: AsyncWrite + Unpin + Send + 'a>(&'a self, sink: &'a mut W) -> Pin<Box<dyn Future<Output = Result<(), WriteError>> + Send + 'a>> {
        Box::pin(async move {
            u64::try_from(self.len())?.write(sink).await?;
            for elt in self {
                elt.write(sink).await?;
            }
            Ok(())
        })
    }

    fn read_sync(stream: &mut impl Read) -> Result<Self, ReadError> {
        let len = u64::read_sync(stream)?;
        let mut set = Self::with_capacity(len.try_into()?);
        for _ in 0..len {
            set.insert(T::read_sync(stream)?);
        }
        Ok(set)
    }

    fn write_sync(&self, sink: &mut impl Write) -> Result<(), WriteError> {
        u64::try_from(self.len())?.write_sync(sink)?;
        for elt in self {
            elt.write_sync(sink)?;
        }
        Ok(())
    }
}

/// A string is encoded in UTF-8 and prefixed with the length in bytes as a [`u64`].
impl Protocol for String {
    fn read<'a, R: AsyncRead + Unpin + Send + 'a>(stream: &'a mut R) -> Pin<Box<dyn Future<Output = Result<Self, ReadError>> + Send + 'a>> {
        Box::pin(async move {
            let buf = Vec::read(stream).await?;
            Ok(Self::from_utf8(buf)?)
        })
    }

    fn write<'a, W: AsyncWrite + Unpin + Send + 'a>(&'a self, sink: &'a mut W) -> Pin<Box<dyn Future<Output = Result<(), WriteError>> + Send + 'a>> {
        Box::pin(async move {
            u64::try_from(self.len())?.write(sink).await?;
            sink.write(self.as_bytes()).await?;
            Ok(())
        })
    }

    fn read_sync(stream: &mut impl Read) -> Result<Self, ReadError> {
        let buf = Vec::read_sync(stream)?;
        Ok(Self::from_utf8(buf)?)
    }

    fn write_sync(&self, sink: &mut impl Write) -> Result<(), WriteError> {
        u64::try_from(self.len())?.write_sync(sink)?;
        sink.write(self.as_bytes())?;
        Ok(())
    }
}

/// A map is prefixed with the length as a [`u64`].
impl<K: Protocol + Ord + Send + Sync + 'static, V: Protocol + Send + Sync + 'static> Protocol for BTreeMap<K, V> {
    fn read<'a, R: AsyncRead + Unpin + Send + 'a>(stream: &'a mut R) -> Pin<Box<dyn Future<Output = Result<Self, ReadError>> + Send + 'a>> {
        Box::pin(async move {
            let len = u64::read(stream).await?;
            usize::try_from(len)?; // error here rather than panicking in the insert loop
            let mut map = Self::default();
            for _ in 0..len {
                map.insert(K::read(stream).await?, V::read(stream).await?);
            }
            Ok(map)
        })
    }

    fn write<'a, W: AsyncWrite + Unpin + Send + 'a>(&'a self, sink: &'a mut W) -> Pin<Box<dyn Future<Output = Result<(), WriteError>> + Send + 'a>> {
        Box::pin(async move {
            u64::try_from(self.len())?.write(sink).await?;
            for (k, v) in self {
                k.write(sink).await?;
                v.write(sink).await?;
            }
            Ok(())
        })
    }

    fn read_sync(stream: &mut impl Read) -> Result<Self, ReadError> {
        let len = u64::read_sync(stream)?;
        usize::try_from(len)?; // error here rather than panicking in the insert loop
        let mut map = Self::default();
        for _ in 0..len {
            map.insert(K::read_sync(stream)?, V::read_sync(stream)?);
        }
        Ok(map)
    }

    fn write_sync(&self, sink: &mut impl Write) -> Result<(), WriteError> {
        u64::try_from(self.len())?.write_sync(sink)?;
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
        Box::pin(async move {
            let len = u64::read(stream).await?;
            let mut map = Self::with_capacity(len.try_into()?);
            for _ in 0..len {
                map.insert(K::read(stream).await?, V::read(stream).await?);
            }
            Ok(map)
        })
    }

    fn write<'a, W: AsyncWrite + Unpin + Send + 'a>(&'a self, sink: &'a mut W) -> Pin<Box<dyn Future<Output = Result<(), WriteError>> + Send + 'a>> {
        Box::pin(async move {
            u64::try_from(self.len())?.write(sink).await?;
            for (k, v) in self {
                k.write(sink).await?;
                v.write(sink).await?;
            }
            Ok(())
        })
    }

    fn read_sync(stream: &mut impl Read) -> Result<Self, ReadError> {
        let len = u64::read_sync(stream)?;
        let mut map = Self::with_capacity(len.try_into()?);
        for _ in 0..len {
            map.insert(K::read_sync(stream)?, V::read_sync(stream)?);
        }
        Ok(map)
    }

    fn write_sync(&self, sink: &mut impl Write) -> Result<(), WriteError> {
        u64::try_from(self.len())?.write_sync(sink)?;
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

macro_rules! impl_protocol_nonzero { //TODO add #[async_proto(map_err = ...)], then replace this with derives
    ($ty:ty, $primitive:ty) => {
        /// A nonzero integer is represented like its value.
        impl Protocol for $ty {
            fn read<'a, R: AsyncRead + Unpin + Send + 'a>(stream: &'a mut R) -> Pin<Box<dyn Future<Output = Result<Self, ReadError>> + Send + 'a>> {
                Box::pin(async move {
                    Ok(Self::new(<$primitive>::read(stream).await?).ok_or(ReadError::UnknownVariant8(0))?)
                })
            }

            fn write<'a, W: AsyncWrite + Unpin + Send + 'a>(&'a self, sink: &'a mut W) -> Pin<Box<dyn Future<Output = Result<(), WriteError>> + Send + 'a>> {
                Box::pin(async move {
                    self.get().write(sink).await?;
                    Ok(())
                })
            }

            fn read_sync(stream: &mut impl Read) -> Result<Self, ReadError> {
                Ok(Self::new(<$primitive>::read_sync(stream)?).ok_or(ReadError::UnknownVariant8(0))?)
            }

            fn write_sync(&self, sink: &mut impl Write) -> Result<(), WriteError> {
                self.get().write_sync(sink)?;
                Ok(())
            }
        }
    };
}

impl_protocol_nonzero!(std::num::NonZeroU8, u8);
impl_protocol_nonzero!(std::num::NonZeroI8, i8);
impl_protocol_nonzero!(std::num::NonZeroU16, u16);
impl_protocol_nonzero!(std::num::NonZeroI16, i16);
impl_protocol_nonzero!(std::num::NonZeroU32, u32);
impl_protocol_nonzero!(std::num::NonZeroI32, i32);
impl_protocol_nonzero!(std::num::NonZeroU64, u64);
impl_protocol_nonzero!(std::num::NonZeroI64, i64);
impl_protocol_nonzero!(std::num::NonZeroU128, u128);
impl_protocol_nonzero!(std::num::NonZeroI128, i128);

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
