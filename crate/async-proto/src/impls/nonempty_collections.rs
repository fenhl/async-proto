use {
    std::{
        hash::Hash,
        io::prelude::*,
        num::NonZero,
        pin::Pin,
    },
    nonempty_collections::{
        NEMap,
        NESet,
        NEVec,
    },
    tokio::io::{
        AsyncRead,
        AsyncWrite,
    },
    crate::{
        ErrorContext,
        LengthPrefixed,
        Protocol,
        ReadError,
        ReadErrorKind,
        WriteError,
    },
};

/// A vector is prefixed with the length as a [`u64`].
#[cfg_attr(docsrs, doc(cfg(feature = "nonempty-collections")))]
impl<T: Protocol + Send + Sync> Protocol for NEVec<T> {
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

#[cfg_attr(docsrs, doc(cfg(feature = "nonempty-collections")))]
impl<T: Protocol + Send + Sync> LengthPrefixed for NEVec<T> {
    fn read_length_prefixed<'a, R: AsyncRead + Unpin + Send + 'a>(stream: &'a mut R, max_len: u64) -> Pin<Box<dyn Future<Output = Result<Self, ReadError>> + Send + 'a>> {
        Box::pin(async move {
            let len = super::read_len(stream, max_len, || ErrorContext::BuiltIn { for_type: "NEVec" }).await?;
            let len = NonZero::new(len).ok_or_else(|| ReadError {
                context: ErrorContext::BuiltIn { for_type: "NEVec" },
                kind: ReadErrorKind::UnknownVariant64(0),
            })?;
            let mut buf = Self::with_capacity(len, T::read(stream).await?); //TODO use fallible allocation once available
            for _ in 1..len.get() {
                buf.push(T::read(stream).await?);
            }
            Ok(buf)
        })
    }

    fn write_length_prefixed<'a, W: AsyncWrite + Unpin + Send + 'a>(&'a self, sink: &'a mut W, max_len: u64) -> Pin<Box<dyn Future<Output = Result<(), WriteError>> + Send + 'a>> {
        Box::pin(async move {
            super::write_len(sink, self.len().get(), max_len, || ErrorContext::BuiltIn { for_type: "NEVec" }).await?;
            for elt in self {
                elt.write(sink).await?;
            }
            Ok(())
        })
    }

    fn read_length_prefixed_sync(stream: &mut impl Read, max_len: u64) -> Result<Self, ReadError> {
        let len = super::read_len_sync(stream, max_len, || ErrorContext::BuiltIn { for_type: "NEVec" })?;
        let len = NonZero::new(len).ok_or_else(|| ReadError {
            context: ErrorContext::BuiltIn { for_type: "NEVec" },
            kind: ReadErrorKind::UnknownVariant64(0),
        })?;
        let mut buf = Self::with_capacity(len, T::read_sync(stream)?); //TODO use fallible allocation once available
        for _ in 1..len.get() {
            buf.push(T::read_sync(stream)?);
        }
        Ok(buf)
    }

    fn write_length_prefixed_sync(&self, sink: &mut impl Write, max_len: u64) -> Result<(), WriteError> {
        super::write_len_sync(sink, self.len().get(), max_len, || ErrorContext::BuiltIn { for_type: "NEVec" })?;
        for elt in self {
            elt.write_sync(sink)?;
        }
        Ok(())
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "nonempty-collections")))]
impl<T: Protocol + Eq + Hash + Send + Sync> Protocol for NESet<T> {
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

/// A set is prefixed with the length as a [`u64`].
#[cfg_attr(docsrs, doc(cfg(feature = "nonempty-collections")))]
impl<T: Protocol + Eq + Hash + Send + Sync> LengthPrefixed for NESet<T> {
    fn read_length_prefixed<'a, R: AsyncRead + Unpin + Send + 'a>(stream: &'a mut R, max_len: u64) -> Pin<Box<dyn Future<Output = Result<Self, ReadError>> + Send + 'a>> {
        Box::pin(async move {
            let len = super::read_len(stream, max_len, || ErrorContext::BuiltIn { for_type: "NESet" }).await?;
            let len = NonZero::new(len).ok_or_else(|| ReadError {
                context: ErrorContext::BuiltIn { for_type: "NESet" },
                kind: ReadErrorKind::UnknownVariant64(0),
            })?;
            let mut set = Self::with_capacity(len, T::read(stream).await?); //TODO use fallible allocation once available
            for _ in 1..len.get() {
                set.insert(T::read(stream).await?);
            }
            Ok(set)
        })
    }

    fn write_length_prefixed<'a, W: AsyncWrite + Unpin + Send + 'a>(&'a self, sink: &'a mut W, max_len: u64) -> Pin<Box<dyn Future<Output = Result<(), WriteError>> + Send + 'a>> {
        Box::pin(async move {
            super::write_len(sink, self.len().get(), max_len, || ErrorContext::BuiltIn { for_type: "NESet" }).await?;
            for elt in self {
                elt.write(sink).await?;
            }
            Ok(())
        })
    }

    fn read_length_prefixed_sync(stream: &mut impl Read, max_len: u64) -> Result<Self, ReadError> {
        let len = super::read_len_sync(stream, max_len, || ErrorContext::BuiltIn { for_type: "NESet" })?;
        let len = NonZero::new(len).ok_or_else(|| ReadError {
            context: ErrorContext::BuiltIn { for_type: "NESet" },
            kind: ReadErrorKind::UnknownVariant64(0),
        })?;
        let mut set = Self::with_capacity(len, T::read_sync(stream)?); //TODO use fallible allocation once available
        for _ in 1..len.get() {
            set.insert(T::read_sync(stream)?);
        }
        Ok(set)
    }

    fn write_length_prefixed_sync(&self, sink: &mut impl Write, max_len: u64) -> Result<(), WriteError> {
        super::write_len_sync(sink, self.len().get(), max_len, || ErrorContext::BuiltIn { for_type: "NESet" })?;
        for elt in self {
            elt.write_sync(sink)?;
        }
        Ok(())
    }
}

/// A map is prefixed with the length as a [`u64`].
#[cfg_attr(docsrs, doc(cfg(feature = "nonempty-collections")))]
impl<K: Protocol + Eq + Hash + Send + Sync, V: Protocol + Send + Sync> Protocol for NEMap<K, V> {
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

#[cfg_attr(docsrs, doc(cfg(feature = "nonempty-collections")))]
impl<K: Protocol + Eq + Hash + Send + Sync, V: Protocol + Send + Sync> LengthPrefixed for NEMap<K, V> {
    fn read_length_prefixed<'a, R: AsyncRead + Unpin + Send + 'a>(stream: &'a mut R, max_len: u64) -> Pin<Box<dyn Future<Output = Result<Self, ReadError>> + Send + 'a>> {
        Box::pin(async move {
            let len = super::read_len(stream, max_len, || ErrorContext::BuiltIn { for_type: "NEMap" }).await?;
            let len = NonZero::new(len).ok_or_else(|| ReadError {
                context: ErrorContext::BuiltIn { for_type: "NEMap" },
                kind: ReadErrorKind::UnknownVariant64(0),
            })?;
            let mut map = Self::with_capacity(len, K::read(stream).await?, V::read(stream).await?); //TODO use fallible allocation once available
            for _ in 1..len.get() {
                map.insert(K::read(stream).await?, V::read(stream).await?);
            }
            Ok(map)
        })
    }

    fn write_length_prefixed<'a, W: AsyncWrite + Unpin + Send + 'a>(&'a self, sink: &'a mut W, max_len: u64) -> Pin<Box<dyn Future<Output = Result<(), WriteError>> + Send + 'a>> {
        Box::pin(async move {
            super::write_len(sink, self.len().get(), max_len, || ErrorContext::BuiltIn { for_type: "NEMap" }).await?;
            for (k, v) in self {
                k.write(sink).await?;
                v.write(sink).await?;
            }
            Ok(())
        })
    }

    fn read_length_prefixed_sync(stream: &mut impl Read, max_len: u64) -> Result<Self, ReadError> {
        let len = super::read_len_sync(stream, max_len, || ErrorContext::BuiltIn { for_type: "NEMap" })?;
        let len = NonZero::new(len).ok_or_else(|| ReadError {
            context: ErrorContext::BuiltIn { for_type: "NEMap" },
            kind: ReadErrorKind::UnknownVariant64(0),
        })?;
        let mut map = Self::with_capacity(len, K::read_sync(stream)?, V::read_sync(stream)?); //TODO use fallible allocation once available
        for _ in 1..len.get() {
            map.insert(K::read_sync(stream)?, V::read_sync(stream)?);
        }
        Ok(map)
    }

    fn write_length_prefixed_sync(&self, sink: &mut impl Write, max_len: u64) -> Result<(), WriteError> {
        super::write_len_sync(sink, self.len().get(), max_len, || ErrorContext::BuiltIn { for_type: "NEMap" })?;
        for (k, v) in self {
            k.write_sync(sink)?;
            v.write_sync(sink)?;
        }
        Ok(())
    }
}
