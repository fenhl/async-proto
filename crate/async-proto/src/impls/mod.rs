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

#[cfg(feature = "serde_json")] mod serde_json;

macro_rules! impl_protocol_primitive {
    ($ty:ty, $read:ident, $write:ident$(, $endian:ty)?) => {
        impl Protocol for $ty {
            fn read<'a, R: AsyncRead + Unpin + Send + 'a>(stream: &'a mut R) -> Pin<Box<dyn Future<Output = Result<$ty, ReadError>> + Send + 'a>> {
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
            fn read_sync(stream: &mut impl Read) -> Result<$ty, ReadError> {
                Ok(stream.$read$(::<$endian>)?()?)
            }

            #[cfg(feature = "write-sync")]
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

impl Protocol for f32 {
    fn read<'a, R: AsyncRead + Unpin + Send + 'a>(stream: &'a mut R) -> Pin<Box<dyn Future<Output = Result<f32, ReadError>> + Send + 'a>> {
        Box::pin(async move {
            Ok(f32::from_be_bytes(<[u8; 4]>::read(stream).await?))
        })
    }

    fn write<'a, W: AsyncWrite + Unpin + Send + 'a>(&'a self, sink: &'a mut W) -> Pin<Box<dyn Future<Output = Result<(), WriteError>> + Send + 'a>> {
        Box::pin(async move {
            self.to_be_bytes().write(sink).await
        })
    }

    #[cfg(feature = "read-sync")]
    fn read_sync(stream: &mut impl Read) -> Result<f32, ReadError> {
        Ok(stream.read_f32::<NetworkEndian>()?)
    }

    #[cfg(feature = "write-sync")]
    fn write_sync(&self, sink: &mut impl Write) -> Result<(), WriteError> {
        Ok(sink.write_f32::<NetworkEndian>(*self)?)
    }
}

impl Protocol for f64 {
    fn read<'a, R: AsyncRead + Unpin + Send + 'a>(stream: &'a mut R) -> Pin<Box<dyn Future<Output = Result<f64, ReadError>> + Send + 'a>> {
        Box::pin(async move {
            Ok(f64::from_be_bytes(<[u8; 8]>::read(stream).await?))
        })
    }

    fn write<'a, W: AsyncWrite + Unpin + Send + 'a>(&'a self, sink: &'a mut W) -> Pin<Box<dyn Future<Output = Result<(), WriteError>> + Send + 'a>> {
        Box::pin(async move {
            self.to_be_bytes().write(sink).await
        })
    }

    #[cfg(feature = "read-sync")]
    fn read_sync(stream: &mut impl Read) -> Result<f64, ReadError> {
        Ok(stream.read_f64::<NetworkEndian>()?)
    }

    #[cfg(feature = "write-sync")]
    fn write_sync(&self, sink: &mut impl Write) -> Result<(), WriteError> {
        Ok(sink.write_f64::<NetworkEndian>(*self)?)
    }
}

macro_rules! impl_protocol_tuple {
    ($($ty:ident),*) => {
        #[allow(unused)]
        impl<$($ty: Protocol + Send + Sync),*> Protocol for ($($ty,)*) {
            fn read<'a, R: AsyncRead + Unpin + Send + 'a>(stream: &'a mut R) -> Pin<Box<dyn Future<Output = Result<($($ty,)*), ReadError>> + Send + 'a>> {
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
            fn read_sync(stream: &mut impl Read) -> Result<($($ty,)*), ReadError> {
                Ok((
                    $($ty::read_sync(stream)?,)*
                ))
            }

            #[cfg(feature = "write-sync")]
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

macro_rules! impl_protocol_array {
    ($n:literal $(, $ty:ident)+) => {
        #[allow(unused)]
        impl<T: Protocol + Send + Sync> Protocol for [T; $n] {
            fn read<'a, R: AsyncRead + Unpin + Send + 'a>(stream: &'a mut R) -> Pin<Box<dyn Future<Output = Result<[T; $n], ReadError>> + Send + 'a>> {
                Box::pin(async move {
                    Ok([
                        $($ty::read(stream).await?,)+
                    ])
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
            fn read_sync(stream: &mut impl Read) -> Result<[T; $n], ReadError> {
                Ok([
                    $($ty::read_sync(stream)?,)+
                ])
            }

            #[cfg(feature = "write-sync")]
            fn write_sync(&self, sink: &mut impl Write) -> Result<(), WriteError> {
                for elt in self {
                    elt.write_sync(sink)?;
                }
                Ok(())
            }
        }
    };
}

impl<T> Protocol for [T; 0] {
    fn read<'a, R: AsyncRead + Unpin + Send + 'a>(_: &'a mut R) -> Pin<Box<dyn Future<Output = Result<[T; 0], ReadError>> + Send + 'a>> {
        Box::pin(async move {
            Ok([])
        })
    }

    fn write<'a, W: AsyncWrite + Unpin + Send + 'a>(&'a self, _: &'a mut W) -> Pin<Box<dyn Future<Output = Result<(), WriteError>> + Send + 'a>> {
        Box::pin(async move {
            Ok(())
        })
    }

    #[cfg(feature = "read-sync")]
    fn read_sync(_: &mut impl Read) -> Result<[T; 0], ReadError> {
        Ok([])
    }

    #[cfg(feature = "write-sync")]
    fn write_sync(&self, _: &mut impl Write) -> Result<(), WriteError> {
        Ok(())
    }
}

impl_protocol_array!(1, T);
impl_protocol_array!(2, T, T);
impl_protocol_array!(3, T, T, T);
impl_protocol_array!(4, T, T, T, T);
impl_protocol_array!(5, T, T, T, T, T);
impl_protocol_array!(6, T, T, T, T, T, T);
impl_protocol_array!(7, T, T, T, T, T, T, T);
impl_protocol_array!(8, T, T, T, T, T, T, T, T);
impl_protocol_array!(9, T, T, T, T, T, T, T, T, T);
impl_protocol_array!(10, T, T, T, T, T, T, T, T, T, T);
impl_protocol_array!(11, T, T, T, T, T, T, T, T, T, T, T);
impl_protocol_array!(12, T, T, T, T, T, T, T, T, T, T, T, T);
impl_protocol_array!(13, T, T, T, T, T, T, T, T, T, T, T, T, T);
impl_protocol_array!(14, T, T, T, T, T, T, T, T, T, T, T, T, T, T);
impl_protocol_array!(15, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T);
impl_protocol_array!(16, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T);
impl_protocol_array!(17, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T);
impl_protocol_array!(18, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T);
impl_protocol_array!(19, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T);
impl_protocol_array!(20, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T);
impl_protocol_array!(21, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T);
impl_protocol_array!(22, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T);
impl_protocol_array!(23, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T);
impl_protocol_array!(24, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T);
impl_protocol_array!(25, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T);
impl_protocol_array!(26, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T);
impl_protocol_array!(27, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T);
impl_protocol_array!(28, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T);
impl_protocol_array!(29, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T);
impl_protocol_array!(30, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T);
impl_protocol_array!(31, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T);
impl_protocol_array!(32, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T);

impl Protocol for bool {
    fn read<'a, R: AsyncRead + Unpin + Send + 'a>(stream: &'a mut R) -> Pin<Box<dyn Future<Output = Result<bool, ReadError>> + Send + 'a>> {
        Box::pin(async move {
            Ok(match u8::read(stream).await? {
                0 => false,
                1 => true,
                n => return Err(ReadError::UnknownVariant(n)),
            })
        })
    }

    fn write<'a, W: AsyncWrite + Unpin + Send + 'a>(&'a self, sink: &'a mut W) -> Pin<Box<dyn Future<Output = Result<(), WriteError>> + Send + 'a>> {
        Box::pin(async move {
            if *self { 1u8 } else { 0 }.write(sink).await
        })
    }

    #[cfg(feature = "read-sync")]
    fn read_sync(stream: &mut impl Read) -> Result<bool, ReadError> {
        Ok(match u8::read_sync(stream)? {
            0 => false,
            1 => true,
            n => return Err(ReadError::UnknownVariant(n)),
        })
    }

    #[cfg(feature = "write-sync")]
    fn write_sync(&self, sink: &mut impl Write) -> Result<(), WriteError> {
        if *self { 1u8 } else { 0 }.write_sync(sink)
    }
}

impl<T: Protocol + Sync> Protocol for Option<T> { //TODO add support for generics to impl_protocol_for, then replace this impl
    fn read<'a, R: AsyncRead + Unpin + Send + 'a>(stream: &'a mut R) -> Pin<Box<dyn Future<Output = Result<Option<T>, ReadError>> + Send + 'a>> {
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
    fn read_sync(stream: &mut impl Read) -> Result<Option<T>, ReadError> {
        Ok(if bool::read_sync(stream)? {
            Some(T::read_sync(stream)?)
        } else {
            None
        })
    }

    #[cfg(feature = "write-sync")]
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

impl<T: Protocol + Send + Sync> Protocol for Vec<T> {
    fn read<'a, R: AsyncRead + Unpin + Send + 'a>(stream: &'a mut R) -> Pin<Box<dyn Future<Output = Result<Vec<T>, ReadError>> + Send + 'a>> {
        Box::pin(async move {
            let len = u64::read(stream).await?;
            let mut buf = Vec::with_capacity(len.try_into()?);
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
    fn read_sync(stream: &mut impl Read) -> Result<Vec<T>, ReadError> {
        let len = u64::read_sync(stream)?;
        let mut buf = Vec::with_capacity(len.try_into()?);
        for _ in 0..len {
            buf.push(T::read_sync(stream)?);
        }
        Ok(buf)
    }

    #[cfg(feature = "write-sync")]
    fn write_sync(&self, sink: &mut impl Write) -> Result<(), WriteError> {
        u64::try_from(self.len())?.write_sync(sink)?;
        for elt in self {
            elt.write_sync(sink)?;
        }
        Ok(())
    }
}

impl<T: Protocol + Ord + Send + Sync + 'static> Protocol for BTreeSet<T> {
    fn read<'a, R: AsyncRead + Unpin + Send + 'a>(stream: &'a mut R) -> Pin<Box<dyn Future<Output = Result<BTreeSet<T>, ReadError>> + Send + 'a>> {
        Box::pin(async move {
            let len = u64::read(stream).await?;
            usize::try_from(len)?; // error here rather than panicking in the insert loop
            let mut set = BTreeSet::default();
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
    fn read_sync(stream: &mut impl Read) -> Result<BTreeSet<T>, ReadError> {
        let len = u64::read_sync(stream)?;
        usize::try_from(len)?; // error here rather than panicking in the insert loop
        let mut set = BTreeSet::default();
        for _ in 0..len {
            set.insert(T::read_sync(stream)?);
        }
        Ok(set)
    }

    #[cfg(feature = "write-sync")]
    fn write_sync(&self, sink: &mut impl Write) -> Result<(), WriteError> {
        u64::try_from(self.len())?.write_sync(sink)?;
        for elt in self {
            elt.write_sync(sink)?;
        }
        Ok(())
    }
}

impl<T: Protocol + Eq + Hash + Send + Sync> Protocol for HashSet<T> {
    fn read<'a, R: AsyncRead + Unpin + Send + 'a>(stream: &'a mut R) -> Pin<Box<dyn Future<Output = Result<HashSet<T>, ReadError>> + Send + 'a>> {
        Box::pin(async move {
            let len = u64::read(stream).await?;
            let mut set = HashSet::with_capacity(len.try_into()?);
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
    fn read_sync(stream: &mut impl Read) -> Result<HashSet<T>, ReadError> {
        let len = u64::read_sync(stream)?;
        let mut set = HashSet::with_capacity(len.try_into()?);
        for _ in 0..len {
            set.insert(T::read_sync(stream)?);
        }
        Ok(set)
    }

    #[cfg(feature = "write-sync")]
    fn write_sync(&self, sink: &mut impl Write) -> Result<(), WriteError> {
        u64::try_from(self.len())?.write_sync(sink)?;
        for elt in self {
            elt.write_sync(sink)?;
        }
        Ok(())
    }
}

impl Protocol for String {
    fn read<'a, R: AsyncRead + Unpin + Send + 'a>(stream: &'a mut R) -> Pin<Box<dyn Future<Output = Result<String, ReadError>> + Send + 'a>> {
        Box::pin(async move {
            let buf = Vec::read(stream).await?;
            Ok(String::from_utf8(buf)?)
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
    fn read_sync(stream: &mut impl Read) -> Result<String, ReadError> {
        let buf = Vec::read_sync(stream)?;
        Ok(String::from_utf8(buf)?)
    }

    #[cfg(feature = "write-sync")]
    fn write_sync(&self, sink: &mut impl Write) -> Result<(), WriteError> {
        u64::try_from(self.len())?.write_sync(sink)?;
        sink.write(self.as_bytes())?;
        Ok(())
    }
}

impl<K: Protocol + Ord + Send + Sync + 'static, V: Protocol + Send + Sync + 'static> Protocol for BTreeMap<K, V> {
    fn read<'a, R: AsyncRead + Unpin + Send + 'a>(stream: &'a mut R) -> Pin<Box<dyn Future<Output = Result<BTreeMap<K, V>, ReadError>> + Send + 'a>> {
        Box::pin(async move {
            let len = u64::read(stream).await?;
            usize::try_from(len)?; // error here rather than panicking in the insert loop
            let mut map = BTreeMap::default();
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
    fn read_sync(stream: &mut impl Read) -> Result<BTreeMap<K, V>, ReadError> {
        let len = u64::read_sync(stream)?;
        usize::try_from(len)?; // error here rather than panicking in the insert loop
        let mut map = BTreeMap::default();
        for _ in 0..len {
            map.insert(K::read_sync(stream)?, V::read_sync(stream)?);
        }
        Ok(map)
    }

    #[cfg(feature = "write-sync")]
    fn write_sync(&self, sink: &mut impl Write) -> Result<(), WriteError> {
        u64::try_from(self.len())?.write_sync(sink)?;
        for (k, v) in self {
            k.write_sync(sink)?;
            v.write_sync(sink)?;
        }
        Ok(())
    }
}

impl<K: Protocol + Eq + Hash + Send + Sync, V: Protocol + Send + Sync> Protocol for HashMap<K, V> {
    fn read<'a, R: AsyncRead + Unpin + Send + 'a>(stream: &'a mut R) -> Pin<Box<dyn Future<Output = Result<HashMap<K, V>, ReadError>> + Send + 'a>> {
        Box::pin(async move {
            let len = u64::read(stream).await?;
            let mut map = HashMap::with_capacity(len.try_into()?);
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
    fn read_sync(stream: &mut impl Read) -> Result<HashMap<K, V>, ReadError> {
        let len = u64::read_sync(stream)?;
        let mut map = HashMap::with_capacity(len.try_into()?);
        for _ in 0..len {
            map.insert(K::read_sync(stream)?, V::read_sync(stream)?);
        }
        Ok(map)
    }

    #[cfg(feature = "write-sync")]
    fn write_sync(&self, sink: &mut impl Write) -> Result<(), WriteError> {
        u64::try_from(self.len())?.write_sync(sink)?;
        for (k, v) in self {
            k.write_sync(sink)?;
            v.write_sync(sink)?;
        }
        Ok(())
    }
}

impl Protocol for std::time::Duration {
    fn read<'a, R: AsyncRead + Unpin + Send + 'a>(stream: &'a mut R) -> Pin<Box<dyn Future<Output = Result<std::time::Duration, ReadError>> + Send + 'a>> {
        Box::pin(async move {
            Ok(std::time::Duration::new(u64::read(stream).await?, u32::read(stream).await?))
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
    fn read_sync(stream: &mut impl Read) -> Result<std::time::Duration, ReadError> {
        Ok(std::time::Duration::new(u64::read_sync(stream)?, u32::read_sync(stream)?))
    }

    #[cfg(feature = "write-sync")]
    fn write_sync(&self, sink: &mut impl Write) -> Result<(), WriteError> {
        self.as_secs().write_sync(sink)?;
        self.subsec_nanos().write_sync(sink)?;
        Ok(())
    }
}

impl_protocol_for! {
    enum std::convert::Infallible {}
}
