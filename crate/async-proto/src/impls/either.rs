use async_proto_derive::impl_protocol_for;

impl_protocol_for! {
    #[async_proto(attr(cfg_attr(docsrs, doc(cfg(feature = "either")))))]
    enum either::Either<T, U> {
        Left(T),
        Right(U),
    }
}
