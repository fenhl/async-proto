use {
    async_proto_derive::impl_protocol_for,
    crate::{
        Protocol,
        ReadError,
    },
};

#[derive(Protocol)]
#[async_proto(internal)]
struct IdentifierProxy(String);

impl TryFrom<IdentifierProxy> for semver::Prerelease {
    type Error = ReadError;

    fn try_from(IdentifierProxy(s): IdentifierProxy) -> Result<Self, ReadError> {
        s.parse::<Self>().map_err(|e| ReadError::Custom(e.to_string()))
    }
}

impl<'a> From<&'a semver::Prerelease> for IdentifierProxy {
    fn from(prerelease: &semver::Prerelease) -> Self {
        Self(prerelease.to_string())
    }
}

impl TryFrom<IdentifierProxy> for semver::BuildMetadata {
    type Error = ReadError;

    fn try_from(IdentifierProxy(s): IdentifierProxy) -> Result<Self, ReadError> {
        s.parse::<Self>().map_err(|e| ReadError::Custom(e.to_string()))
    }
}

impl<'a> From<&'a semver::BuildMetadata> for IdentifierProxy {
    fn from(build: &semver::BuildMetadata) -> Self {
        Self(build.to_string())
    }
}

impl_protocol_for! {
    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "semver")))))]
    struct semver::Version {
        major: u64,
        minor: u64,
        patch: u64,
        pre: semver::Prerelease,
        build: semver::BuildMetadata,
    }

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "semver")))))]
    #[async_proto(via = IdentifierProxy)]
    type semver::Prerelease;

    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "semver")))))]
    #[async_proto(via = IdentifierProxy)]
    type semver::BuildMetadata;
}
