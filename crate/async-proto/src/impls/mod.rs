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
        ops::{
            Range,
            RangeFrom,
            RangeInclusive,
            RangeTo,
            RangeToInclusive,
        },
        pin::Pin,
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
#[cfg(any(feature = "read-sync", feature = "write-sync"))] use {
    std::io::prelude::*,
    byteorder::NetworkEndian,
};
#[cfg(feature = "read-sync")] use byteorder::ReadBytesExt as _;
#[cfg(feature = "write-sync")] use byteorder::WriteBytesExt as _;

#[cfg(feature = "chrono-tz")] mod chrono_tz;
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

            #[cfg(feature = "read-sync")]
            #[cfg_attr(docsrs, doc(cfg(feature = "read-sync")))]
            fn read_sync(stream: &mut impl Read) -> Result<Self, ReadError> {
                Ok(stream.$read$(::<$endian>)?()?)
            }

            #[cfg(feature = "write-sync")]
            #[cfg_attr(docsrs, doc(cfg(feature = "write-sync")))]
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

/// Primitive number types are encoded in [big-endian](https://en.wikipedia.org/wiki/Big-endian) format.
impl Protocol for f32 {
    fn read<'a, R: AsyncRead + Unpin + Send + 'a>(stream: &'a mut R) -> Pin<Box<dyn Future<Output = Result<Self, ReadError>> + Send + 'a>> {
        Box::pin(async move {
            Ok(Self::from_be_bytes(<[u8; 4]>::read(stream).await?))
        })
    }

    fn write<'a, W: AsyncWrite + Unpin + Send + 'a>(&'a self, sink: &'a mut W) -> Pin<Box<dyn Future<Output = Result<(), WriteError>> + Send + 'a>> {
        Box::pin(async move {
            self.to_be_bytes().write(sink).await
        })
    }

    #[cfg(feature = "read-sync")]
    #[cfg_attr(docsrs, doc(cfg(feature = "read-sync")))]
    fn read_sync(stream: &mut impl Read) -> Result<Self, ReadError> {
        Ok(stream.read_f32::<NetworkEndian>()?)
    }

    #[cfg(feature = "write-sync")]
    #[cfg_attr(docsrs, doc(cfg(feature = "write-sync")))]
    fn write_sync(&self, sink: &mut impl Write) -> Result<(), WriteError> {
        Ok(sink.write_f32::<NetworkEndian>(*self)?)
    }
}

/// Primitive number types are encoded in [big-endian](https://en.wikipedia.org/wiki/Big-endian) format.
impl Protocol for f64 {
    fn read<'a, R: AsyncRead + Unpin + Send + 'a>(stream: &'a mut R) -> Pin<Box<dyn Future<Output = Result<Self, ReadError>> + Send + 'a>> {
        Box::pin(async move {
            Ok(Self::from_be_bytes(<[u8; 8]>::read(stream).await?))
        })
    }

    fn write<'a, W: AsyncWrite + Unpin + Send + 'a>(&'a self, sink: &'a mut W) -> Pin<Box<dyn Future<Output = Result<(), WriteError>> + Send + 'a>> {
        Box::pin(async move {
            self.to_be_bytes().write(sink).await
        })
    }

    #[cfg(feature = "read-sync")]
    #[cfg_attr(docsrs, doc(cfg(feature = "read-sync")))]
    fn read_sync(stream: &mut impl Read) -> Result<Self, ReadError> {
        Ok(stream.read_f64::<NetworkEndian>()?)
    }

    #[cfg(feature = "write-sync")]
    #[cfg_attr(docsrs, doc(cfg(feature = "write-sync")))]
    fn write_sync(&self, sink: &mut impl Write) -> Result<(), WriteError> {
        Ok(sink.write_f64::<NetworkEndian>(*self)?)
    }
}

impl<Idx: Protocol + Send + Sync> Protocol for Range<Idx> { //TODO derive
    fn read<'a, R: AsyncRead + Unpin + Send + 'a>(stream: &'a mut R) -> Pin<Box<dyn Future<Output = Result<Self, ReadError>> + Send + 'a>> {
        Box::pin(async move {
            Ok(Idx::read(stream).await?..Idx::read(stream).await?)
        })
    }

    fn write<'a, W: AsyncWrite + Unpin + Send + 'a>(&'a self, sink: &'a mut W) -> Pin<Box<dyn Future<Output = Result<(), WriteError>> + Send + 'a>> {
        Box::pin(async move {
            self.start.write(sink).await?;
            self.end.write(sink).await?;
            Ok(())
        })
    }

    #[cfg(feature = "read-sync")]
    #[cfg_attr(docsrs, doc(cfg(feature = "read-sync")))]
    fn read_sync(stream: &mut impl Read) -> Result<Self, ReadError> {
        Ok(Idx::read_sync(stream)?..Idx::read_sync(stream)?)
    }

    #[cfg(feature = "write-sync")]
    #[cfg_attr(docsrs, doc(cfg(feature = "write-sync")))]
    fn write_sync(&self, sink: &mut impl Write) -> Result<(), WriteError> {
        self.start.write_sync(sink)?;
        self.end.write_sync(sink)?;
        Ok(())
    }
}

impl<Idx: Protocol + Send + Sync> Protocol for RangeFrom<Idx> { //TODO derive
    fn read<'a, R: AsyncRead + Unpin + Send + 'a>(stream: &'a mut R) -> Pin<Box<dyn Future<Output = Result<Self, ReadError>> + Send + 'a>> {
        Box::pin(async move {
            Ok(Idx::read(stream).await?..)
        })
    }

    fn write<'a, W: AsyncWrite + Unpin + Send + 'a>(&'a self, sink: &'a mut W) -> Pin<Box<dyn Future<Output = Result<(), WriteError>> + Send + 'a>> {
        Box::pin(async move {
            self.start.write(sink).await?;
            Ok(())
        })
    }

    #[cfg(feature = "read-sync")]
    #[cfg_attr(docsrs, doc(cfg(feature = "read-sync")))]
    fn read_sync(stream: &mut impl Read) -> Result<Self, ReadError> {
        Ok(Idx::read_sync(stream)?..)
    }

    #[cfg(feature = "write-sync")]
    #[cfg_attr(docsrs, doc(cfg(feature = "write-sync")))]
    fn write_sync(&self, sink: &mut impl Write) -> Result<(), WriteError> {
        self.start.write_sync(sink)?;
        Ok(())
    }
}

impl<Idx: Protocol + Send + Sync> Protocol for RangeInclusive<Idx> {
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

    #[cfg(feature = "read-sync")]
    #[cfg_attr(docsrs, doc(cfg(feature = "read-sync")))]
    fn read_sync(stream: &mut impl Read) -> Result<Self, ReadError> {
        Ok(Idx::read_sync(stream)?..=Idx::read_sync(stream)?)
    }

    #[cfg(feature = "write-sync")]
    #[cfg_attr(docsrs, doc(cfg(feature = "write-sync")))]
    fn write_sync(&self, sink: &mut impl Write) -> Result<(), WriteError> {
        self.start().write_sync(sink)?;
        self.end().write_sync(sink)?;
        Ok(())
    }
}

impl<Idx: Protocol + Send + Sync> Protocol for RangeTo<Idx> { //TODO derive
    fn read<'a, R: AsyncRead + Unpin + Send + 'a>(stream: &'a mut R) -> Pin<Box<dyn Future<Output = Result<Self, ReadError>> + Send + 'a>> {
        Box::pin(async move {
            Ok(..Idx::read(stream).await?)
        })
    }

    fn write<'a, W: AsyncWrite + Unpin + Send + 'a>(&'a self, sink: &'a mut W) -> Pin<Box<dyn Future<Output = Result<(), WriteError>> + Send + 'a>> {
        Box::pin(async move {
            self.end.write(sink).await?;
            Ok(())
        })
    }

    #[cfg(feature = "read-sync")]
    #[cfg_attr(docsrs, doc(cfg(feature = "read-sync")))]
    fn read_sync(stream: &mut impl Read) -> Result<Self, ReadError> {
        Ok(..Idx::read_sync(stream)?)
    }

    #[cfg(feature = "write-sync")]
    #[cfg_attr(docsrs, doc(cfg(feature = "write-sync")))]
    fn write_sync(&self, sink: &mut impl Write) -> Result<(), WriteError> {
        self.end.write_sync(sink)?;
        Ok(())
    }
}

impl<Idx: Protocol + Send + Sync> Protocol for RangeToInclusive<Idx> { //TODO derive
    fn read<'a, R: AsyncRead + Unpin + Send + 'a>(stream: &'a mut R) -> Pin<Box<dyn Future<Output = Result<Self, ReadError>> + Send + 'a>> {
        Box::pin(async move {
            Ok(..=Idx::read(stream).await?)
        })
    }

    fn write<'a, W: AsyncWrite + Unpin + Send + 'a>(&'a self, sink: &'a mut W) -> Pin<Box<dyn Future<Output = Result<(), WriteError>> + Send + 'a>> {
        Box::pin(async move {
            self.end.write(sink).await?;
            Ok(())
        })
    }

    #[cfg(feature = "read-sync")]
    #[cfg_attr(docsrs, doc(cfg(feature = "read-sync")))]
    fn read_sync(stream: &mut impl Read) -> Result<Self, ReadError> {
        Ok(..=Idx::read_sync(stream)?)
    }

    #[cfg(feature = "write-sync")]
    #[cfg_attr(docsrs, doc(cfg(feature = "write-sync")))]
    fn write_sync(&self, sink: &mut impl Write) -> Result<(), WriteError> {
        self.end.write_sync(sink)?;
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

            #[cfg(feature = "read-sync")]
            #[cfg_attr(docsrs, doc(cfg(feature = "read-sync")))]
            fn read_sync(stream: &mut impl Read) -> Result<Self, ReadError> {
                Ok((
                    $($ty::read_sync(stream)?,)*
                ))
            }

            #[cfg(feature = "write-sync")]
            #[cfg_attr(docsrs, doc(cfg(feature = "write-sync")))]
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

    #[cfg(feature = "read-sync")]
    #[cfg_attr(docsrs, doc(cfg(feature = "read-sync")))]
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

    #[cfg(feature = "write-sync")]
    #[cfg_attr(docsrs, doc(cfg(feature = "write-sync")))]
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

    #[cfg(feature = "read-sync")]
    #[cfg_attr(docsrs, doc(cfg(feature = "read-sync")))]
    fn read_sync(stream: &mut impl Read) -> Result<Self, ReadError> {
        Ok(match u8::read_sync(stream)? {
            0 => false,
            1 => true,
            n => return Err(ReadError::UnknownVariant8(n)),
        })
    }

    #[cfg(feature = "write-sync")]
    #[cfg_attr(docsrs, doc(cfg(feature = "write-sync")))]
    fn write_sync(&self, sink: &mut impl Write) -> Result<(), WriteError> {
        if *self { 1u8 } else { 0 }.write_sync(sink)
    }
}

impl<T: Protocol + Sync> Protocol for Option<T> { //TODO add support for generics to impl_protocol_for, then replace this impl
    fn read<'a, R: AsyncRead + Unpin + Send + 'a>(stream: &'a mut R) -> Pin<Box<dyn Future<Output = Result<Self, ReadError>> + Send + 'a>> {
        Box::pin(async move {
            Ok(if bool::read(stream).await? {
                Some(T::read(stream).await?)
            } else {
                None
            })
        })
    }

    fn write<'a, W: AsyncWrite + Unpin + Send + 'a>(&'a self, sink: &'a mut W) -> Pin<Box<dyn Future<Output = Result<(), WriteError>> + Send + 'a>> {
        Box::pin(async move {
            if let Some(value) = self {
                true.write(sink).await?;
                value.write(sink).await?;
            } else {
                false.write(sink).await?;
            }
            Ok(())
        })
    }

    #[cfg(feature = "read-sync")]
    #[cfg_attr(docsrs, doc(cfg(feature = "read-sync")))]
    fn read_sync(stream: &mut impl Read) -> Result<Self, ReadError> {
        Ok(if bool::read_sync(stream)? {
            Some(T::read_sync(stream)?)
        } else {
            None
        })
    }

    #[cfg(feature = "write-sync")]
    #[cfg_attr(docsrs, doc(cfg(feature = "write-sync")))]
    fn write_sync(&self, sink: &mut impl Write) -> Result<(), WriteError> {
        if let Some(value) = self {
            true.write_sync(sink)?;
            value.write_sync(sink)?;
        } else {
            false.write_sync(sink)?;
        }
        Ok(())
    }
}

/// A vector is prefixed with the length as a [`u64`].
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

    #[cfg(feature = "read-sync")]
    #[cfg_attr(docsrs, doc(cfg(feature = "read-sync")))]
    fn read_sync(stream: &mut impl Read) -> Result<Self, ReadError> {
        let len = u64::read_sync(stream)?;
        let mut buf = Self::with_capacity(len.try_into()?);
        for _ in 0..len {
            buf.push(T::read_sync(stream)?);
        }
        Ok(buf)
    }

    #[cfg(feature = "write-sync")]
    #[cfg_attr(docsrs, doc(cfg(feature = "write-sync")))]
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

    #[cfg(feature = "read-sync")]
    #[cfg_attr(docsrs, doc(cfg(feature = "read-sync")))]
    fn read_sync(stream: &mut impl Read) -> Result<Self, ReadError> {
        let len = u64::read_sync(stream)?;
        usize::try_from(len)?; // error here rather than panicking in the insert loop
        let mut set = Self::default();
        for _ in 0..len {
            set.insert(T::read_sync(stream)?);
        }
        Ok(set)
    }

    #[cfg(feature = "write-sync")]
    #[cfg_attr(docsrs, doc(cfg(feature = "write-sync")))]
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

    #[cfg(feature = "read-sync")]
    #[cfg_attr(docsrs, doc(cfg(feature = "read-sync")))]
    fn read_sync(stream: &mut impl Read) -> Result<Self, ReadError> {
        let len = u64::read_sync(stream)?;
        let mut set = Self::with_capacity(len.try_into()?);
        for _ in 0..len {
            set.insert(T::read_sync(stream)?);
        }
        Ok(set)
    }

    #[cfg(feature = "write-sync")]
    #[cfg_attr(docsrs, doc(cfg(feature = "write-sync")))]
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

    #[cfg(feature = "read-sync")]
    #[cfg_attr(docsrs, doc(cfg(feature = "read-sync")))]
    fn read_sync(stream: &mut impl Read) -> Result<Self, ReadError> {
        let buf = Vec::read_sync(stream)?;
        Ok(Self::from_utf8(buf)?)
    }

    #[cfg(feature = "write-sync")]
    #[cfg_attr(docsrs, doc(cfg(feature = "write-sync")))]
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

    #[cfg(feature = "read-sync")]
    #[cfg_attr(docsrs, doc(cfg(feature = "read-sync")))]
    fn read_sync(stream: &mut impl Read) -> Result<Self, ReadError> {
        let len = u64::read_sync(stream)?;
        usize::try_from(len)?; // error here rather than panicking in the insert loop
        let mut map = Self::default();
        for _ in 0..len {
            map.insert(K::read_sync(stream)?, V::read_sync(stream)?);
        }
        Ok(map)
    }

    #[cfg(feature = "write-sync")]
    #[cfg_attr(docsrs, doc(cfg(feature = "write-sync")))]
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

    #[cfg(feature = "read-sync")]
    #[cfg_attr(docsrs, doc(cfg(feature = "read-sync")))]
    fn read_sync(stream: &mut impl Read) -> Result<Self, ReadError> {
        let len = u64::read_sync(stream)?;
        let mut map = Self::with_capacity(len.try_into()?);
        for _ in 0..len {
            map.insert(K::read_sync(stream)?, V::read_sync(stream)?);
        }
        Ok(map)
    }

    #[cfg(feature = "write-sync")]
    #[cfg_attr(docsrs, doc(cfg(feature = "write-sync")))]
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

    #[cfg(feature = "read-sync")]
    #[cfg_attr(docsrs, doc(cfg(feature = "read-sync")))]
    fn read_sync(stream: &mut impl Read) -> Result<Self, ReadError> {
        Ok(Self::Owned(B::Owned::read_sync(stream)?))
    }

    #[cfg(feature = "write-sync")]
    #[cfg_attr(docsrs, doc(cfg(feature = "write-sync")))]
    fn write_sync(&self, sink: &mut impl Write) -> Result<(), WriteError> {
        match self {
            Self::Borrowed(borrowed) => (*borrowed).to_owned().write_sync(sink)?,
            Self::Owned(owned) => owned.write_sync(sink)?,
        }
        Ok(())
    }
}

macro_rules! impl_protocol_nonzero {
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

            #[cfg(feature = "read-sync")]
            #[cfg_attr(docsrs, doc(cfg(feature = "read-sync")))]
            fn read_sync(stream: &mut impl Read) -> Result<Self, ReadError> {
                Ok(Self::new(<$primitive>::read_sync(stream)?).ok_or(ReadError::UnknownVariant8(0))?)
            }

            #[cfg(feature = "write-sync")]
            #[cfg_attr(docsrs, doc(cfg(feature = "write-sync")))]
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

/// A duration is represented as the number of whole seconds as a [`u64`] followed by the number of subsecond nanoseconds as a [`u32`].
impl Protocol for std::time::Duration {
    fn read<'a, R: AsyncRead + Unpin + Send + 'a>(stream: &'a mut R) -> Pin<Box<dyn Future<Output = Result<Self, ReadError>> + Send + 'a>> {
        Box::pin(async move {
            Ok(Self::new(u64::read(stream).await?, u32::read(stream).await?))
        })
    }

    fn write<'a, W: AsyncWrite + Unpin + Send + 'a>(&'a self, sink: &'a mut W) -> Pin<Box<dyn Future<Output = Result<(), WriteError>> + Send + 'a>> {
        Box::pin(async move {
            self.as_secs().write(sink).await?;
            self.subsec_nanos().write(sink).await?;
            Ok(())
        })
    }

    #[cfg(feature = "read-sync")]
    #[cfg_attr(docsrs, doc(cfg(feature = "read-sync")))]
    fn read_sync(stream: &mut impl Read) -> Result<Self, ReadError> {
        Ok(Self::new(u64::read_sync(stream)?, u32::read_sync(stream)?))
    }

    #[cfg(feature = "write-sync")]
    #[cfg_attr(docsrs, doc(cfg(feature = "write-sync")))]
    fn write_sync(&self, sink: &mut impl Write) -> Result<(), WriteError> {
        self.as_secs().write_sync(sink)?;
        self.subsec_nanos().write_sync(sink)?;
        Ok(())
    }
}

impl_protocol_for! {
    enum std::convert::Infallible {}
    struct std::ops::RangeFull;
}
