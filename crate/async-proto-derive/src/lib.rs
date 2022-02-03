#![deny(missing_docs, rust_2018_idioms, unused, unused_crate_dependencies, unused_import_braces, unused_lifetimes, unused_qualifications, warnings)]
#![forbid(unsafe_code)]

//! Procedural macros for the `async-proto` crate.

use {
    std::convert::TryFrom as _,
    itertools::Itertools as _,
    proc_macro::TokenStream,
    proc_macro2::Span,
    quote::{
        quote,
        quote_spanned,
    },
    syn::{
        *,
        parse::{
            Parse,
            ParseStream,
        },
        punctuated::Punctuated,
        spanned::Spanned as _,
        token::{
            Brace,
            Paren,
        },
    },
};

fn read_fields(internal: bool, sync: bool, fields: &Fields) -> proc_macro2::TokenStream {
    let async_proto_crate = if internal { quote!(crate) } else { quote!(::async_proto) };
    let read = if sync { quote!(::read_sync(stream)) } else { quote!(::read(stream).await) };
    match fields {
        Fields::Unit => quote!(),
        Fields::Unnamed(FieldsUnnamed { unnamed, .. }) => {
            let read_fields = unnamed.iter()
                .map(|Field { ty, .. }| quote_spanned! {ty.span()=>
                    <#ty as #async_proto_crate::Protocol>#read?
                })
                .collect_vec();
            quote!((#(#read_fields,)*))
        }
        Fields::Named(FieldsNamed { named, .. }) => {
            let read_fields = named.iter()
                .map(|Field { ident, ty, .. }| quote_spanned! {ty.span()=>
                    #ident: <#ty as #async_proto_crate::Protocol>#read?
                })
                .collect_vec();
            quote!({ #(#read_fields,)* })
        }
    }
}

fn fields_pat(fields: &Fields) -> proc_macro2::TokenStream {
    match fields {
        Fields::Unit => quote!(),
        Fields::Unnamed(FieldsUnnamed { unnamed, .. }) => {
            let field_idents = unnamed.iter()
                .enumerate()
                .map(|(idx, _)| Ident::new(&format!("__field{}", idx), Span::call_site()))
                .collect_vec();
            quote!((#(#field_idents,)*))
        }
        Fields::Named(FieldsNamed { named, .. }) => {
            let field_idents = named.iter()
                .map(|Field { ident, .. }| ident)
                .collect_vec();
            quote!({ #(#field_idents,)* })
        }
    }
}

fn write_fields(sync: bool, fields: &Fields) -> proc_macro2::TokenStream {
    let write = if sync { quote!(.write_sync(sink)?) } else { quote!(.write(sink).await?) };
    match fields {
        Fields::Unit => quote!(),
        Fields::Unnamed(FieldsUnnamed { unnamed, .. }) => {
            let field_idents = unnamed.iter()
                .enumerate()
                .map(|(idx, _)| Ident::new(&format!("__field{}", idx), Span::call_site()))
                .collect_vec();
            let write_fields = field_idents.iter()
                .map(|ident| quote!(#ident#write;));
            quote!(#(#write_fields)*)
        }
        Fields::Named(FieldsNamed { named, .. }) => {
            let field_idents = named.iter()
                .map(|Field { ident, .. }| ident)
                .collect_vec();
            let write_fields = field_idents.iter()
                .map(|ident| quote!(#ident#write;));
            quote!(#(#write_fields)*)
        }
    }
}

enum AsyncProtoAttr {
    Attr(Punctuated<Meta, Token![,]>),
    Internal,
    MapErr(Expr),
    Via(Type),
    Where(Punctuated<WherePredicate, Token![,]>),
}

impl Parse for AsyncProtoAttr {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        Ok(if input.peek(Token![where]) {
            let _ = input.parse::<Token![where]>()?;
            let content;
            parenthesized!(content in input);
            Self::Where(content.parse_terminated(WherePredicate::parse)?)
        } else {
            let ident = input.parse::<Ident>()?;
            match &*ident.to_string() {
                "attr" => {
                    let content;
                    parenthesized!(content in input);
                    Self::Attr(content.parse_terminated(Meta::parse)?)
                }
                "internal" => Self::Internal,
                "map_err" => {
                    let _ = input.parse::<Token![=]>()?;
                    Self::MapErr(input.parse()?)
                }
                "via" => {
                    let _ = input.parse::<Token![=]>()?;
                    Self::Via(input.parse()?)
                }
                _ => return Err(Error::new(ident.span(), "unknown async_proto attribute")),
            }
        })
    }
}

fn impl_protocol_inner(mut internal: bool, attrs: Vec<Attribute>, qual_ty: Path, generics: Generics, data: Option<Data>) -> proc_macro2::TokenStream {
    let mut via = None;
    let mut map_err = None;
    let mut where_predicates = None;
    let mut impl_attrs = Vec::default();
    for attr in attrs.into_iter().filter(|attr| attr.path.is_ident("async_proto")) {
        match attr.parse_args_with(Punctuated::<AsyncProtoAttr, Token![,]>::parse_terminated) {
            Ok(attrs) => for attr in attrs {
                match attr {
                    AsyncProtoAttr::Attr(attr) => impl_attrs.extend(attr),
                    AsyncProtoAttr::Internal => internal = true,
                    AsyncProtoAttr::MapErr(expr) => if map_err.replace(expr).is_some() {
                        return quote!(compile_error!("#[async_proto(map_err = ...)] specified multiple times");).into()
                    },
                    AsyncProtoAttr::Via(ty) => if via.replace(ty).is_some() {
                        return quote!(compile_error!("#[async_proto(via = ...)] specified multiple times");).into()
                    },
                    AsyncProtoAttr::Where(predicates) => if where_predicates.replace(predicates).is_some() {
                        return quote!(compile_error!("#[async_proto(where(...))] specified multiple times");).into()
                    },
                }
            },
            Err(e) => return e.to_compile_error().into(),
        }
    }
    let async_proto_crate = if internal { quote!(crate) } else { quote!(::async_proto) };
    let mut impl_generics = generics.clone();
    if let Some(predicates) = where_predicates {
        impl_generics.make_where_clause().predicates.extend(predicates);
    } else {
        for param in impl_generics.type_params_mut() {
            param.colon_token.get_or_insert_with(<Token![:]>::default);
            param.bounds.push(parse_quote!(#async_proto_crate::Protocol));
            param.bounds.push(parse_quote!(::core::marker::Send));
            param.bounds.push(parse_quote!(::core::marker::Sync));
            param.bounds.push(parse_quote!('static));
        }
    };
    let (impl_read, impl_write, impl_read_sync, impl_write_sync) = if let Some(proxy_ty) = via {
        if internal && data.is_some() { return quote!(compile_error!("redundant type layout specification with #[async_proto(via = ...)]");).into() }
        let map_err = map_err.unwrap_or(parse_quote!(::core::convert::Into::<#async_proto_crate::ReadError>::into));
        (
            quote!(<#proxy_ty as ::core::convert::TryInto<Self>>::try_into(<#proxy_ty as #async_proto_crate::Protocol>::read(stream).await?).map_err(#map_err)),
            quote!(<#proxy_ty as #async_proto_crate::Protocol>::write(&<&'a Self as ::core::convert::Into<#proxy_ty>>::into(self), sink).await),
            quote!(<Self as ::core::convert::TryFrom<#proxy_ty>>::try_from(<#proxy_ty as #async_proto_crate::Protocol>::read_sync(stream)?).map_err(#map_err)),
            quote!(<#proxy_ty as #async_proto_crate::Protocol>::write_sync(&<&Self as ::core::convert::Into<#proxy_ty>>::into(self), sink)),
        )
    } else {
        if map_err.is_some() { return quote!(compile_error!("#[async_proto(map_err = ...)] does nothing without #[async_proto(via = ...)]");).into() }
        match data {
            Some(Data::Struct(DataStruct { fields, .. })) => {
                let fields_pat = fields_pat(&fields);
                let read_fields_async = read_fields(internal, false, &fields);
                let write_fields_async = write_fields(false, &fields);
                let read_fields_sync = read_fields(internal, true, &fields);
                let write_fields_sync = write_fields(true, &fields);
                (
                    quote!(::core::result::Result::Ok(Self #read_fields_async)),
                    quote! {
                        let Self #fields_pat = self;
                        #write_fields_async
                        ::core::result::Result::Ok(())
                    },
                    quote!(::core::result::Result::Ok(Self #read_fields_sync)),
                    quote! {
                        let Self #fields_pat = self;
                        #write_fields_sync
                        ::core::result::Result::Ok(())
                    },
                )
            }
            Some(Data::Enum(DataEnum { variants, .. })) => {
                if variants.is_empty() {
                    (
                        quote!(::core::result::Result::Err(#async_proto_crate::ReadError::ReadNever)),
                        quote!(match *self {}),
                        quote!(::core::result::Result::Err(#async_proto_crate::ReadError::ReadNever)),
                        quote!(match *self {}),
                    )
                } else {
                    let (discrim_ty, unknown_variant_variant, get_discrim) = match variants.len() {
                        0 => unreachable!(), // empty enum handled above
                        1..=256 => (quote!(u8), quote!(UnknownVariant8), (&|idx| {
                            let idx = u8::try_from(idx).expect("variant index unexpectedly high");
                            quote!(#idx)
                        }) as &dyn Fn(usize) -> proc_macro2::TokenStream),
                        257..=65_536 => (quote!(u16), quote!(UnknownVariant16), (&|idx| {
                            let idx = u16::try_from(idx).expect("variant index unexpectedly high");
                            quote!(#idx)
                        }) as &dyn Fn(usize) -> proc_macro2::TokenStream),
                        #[cfg(target_pointer_width = "32")]
                        _ => (quote!(u32), quote!(UnknownVariant32), (&|idx| {
                            let idx = u32::try_from(idx).expect("variant index unexpectedly high");
                            quote!(#idx)
                        }) as &dyn Fn(usize) -> proc_macro2::TokenStream),
                        #[cfg(target_pointer_width = "64")]
                        65_537..=4_294_967_296 => (quote!(u32), quote!(UnknownVariant32), (&|idx| {
                            let idx = u32::try_from(idx).expect("variant index unexpectedly high");
                            quote!(#idx)
                        }) as &dyn Fn(usize) -> proc_macro2::TokenStream),
                        #[cfg(target_pointer_width = "64")]
                        _ => (quote!(u64), quote!(UnknownVariant64), (&|idx| {
                            let idx = u64::try_from(idx).expect("variant index unexpectedly high");
                            quote!(#idx)
                        }) as &dyn Fn(usize) -> proc_macro2::TokenStream),
                    };
                    let read_arms = variants.iter()
                        .enumerate()
                        .map(|(idx, Variant { ident: var, fields, .. })| {
                            let idx = get_discrim(idx);
                            let read_fields = read_fields(internal, false, fields);
                            quote!(#idx => ::core::result::Result::Ok(Self::#var #read_fields))
                        })
                        .collect_vec();
                    let write_arms = variants.iter()
                        .enumerate()
                        .map(|(idx, Variant { ident: var, fields, .. })| {
                            let idx = get_discrim(idx);
                            let fields_pat = fields_pat(&fields);
                            let write_fields = write_fields(false, fields);
                            quote! {
                                Self::#var #fields_pat => {
                                    #idx.write(sink).await?;
                                    #write_fields
                                }
                            }
                        })
                        .collect_vec();
                    let read_sync_arms = variants.iter()
                        .enumerate()
                        .map(|(idx, Variant { ident: var, fields, .. })| {
                            let idx = get_discrim(idx);
                            let read_fields = read_fields(internal, true, fields);
                            quote!(#idx => ::core::result::Result::Ok(Self::#var #read_fields))
                        })
                        .collect_vec();
                    let write_sync_arms = variants.iter()
                        .enumerate()
                        .map(|(idx, Variant { ident: var, fields, .. })| {
                            let idx = get_discrim(idx);
                            let fields_pat = fields_pat(&fields);
                            let write_fields = write_fields(true, fields);
                            quote! {
                                Self::#var #fields_pat => {
                                    #idx.write_sync(sink)?;
                                    #write_fields
                                }
                            }
                        })
                        .collect_vec();
                    (
                        quote! {
                            match <#discrim_ty as #async_proto_crate::Protocol>::read(stream).await? {
                                #(#read_arms,)*
                                n => ::core::result::Result::Err(#async_proto_crate::ReadError::#unknown_variant_variant(n)),
                            }
                        },
                        quote! {
                            match self {
                                #(#write_arms,)*
                            }
                            ::core::result::Result::Ok(())
                        },
                        quote! {
                            match <#discrim_ty as #async_proto_crate::Protocol>::read_sync(stream)? {
                                #(#read_sync_arms,)*
                                n => ::core::result::Result::Err(#async_proto_crate::ReadError::#unknown_variant_variant(n)),
                            }
                        },
                        quote! {
                            match self {
                                #(#write_sync_arms,)*
                            }
                            ::core::result::Result::Ok(())
                        },
                    )
                }
            }
            Some(Data::Union(_)) => return quote!(compile_error!("unions not supported in derive(Protocol)");).into(),
            None => return quote!(compile_error!("missing type layout specification or #[async_proto(via = ...)]");).into(),
        }
    };
    let read_sync = if cfg!(feature = "read-sync") {
        quote! {
            fn read_sync(mut stream: &mut impl ::std::io::Read) -> ::core::result::Result<Self, #async_proto_crate::ReadError> { #impl_read_sync }
        }
    } else {
        quote!()
    };
    let write_sync = if cfg!(feature = "write-sync") {
        quote! {
            fn write_sync(&self, mut sink: &mut impl ::std::io::Write) -> ::core::result::Result<(), #async_proto_crate::WriteError> { #impl_write_sync }
        }
    } else {
        quote!()
    };
    let (impl_generics, ty_generics, where_clause) = impl_generics.split_for_impl();
    quote! {
        #(#[#impl_attrs])*
        impl #impl_generics #async_proto_crate::Protocol for #qual_ty #ty_generics #where_clause {
            fn read<'a, R: #async_proto_crate::tokio::io::AsyncRead + ::core::marker::Unpin + ::core::marker::Send + 'a>(stream: &'a mut R) -> ::std::pin::Pin<::std::boxed::Box<dyn ::std::future::Future<Output = ::core::result::Result<Self, #async_proto_crate::ReadError>> + ::core::marker::Send + 'a>> {
                ::std::boxed::Box::pin(async move { #impl_read })
            }

            fn write<'a, W: #async_proto_crate::tokio::io::AsyncWrite + ::core::marker::Unpin + ::core::marker::Send + 'a>(&'a self, sink: &'a mut W) -> ::std::pin::Pin<::std::boxed::Box<dyn ::std::future::Future<Output = ::core::result::Result<(), #async_proto_crate::WriteError>> + ::core::marker::Send + 'a>> {
                ::std::boxed::Box::pin(async move { #impl_write })
            }

            #read_sync
            #write_sync
        }
    }
}

/// Implements the `Protocol` trait for this type.
///
/// The network representation is very simple:
///
/// * Attempting to read an `enum` with no variants errors immediately, without waiting for data to appear on the stream.
/// * For non-empty `enum`s, the representation starts with the discriminant (a number representing the variant), starting with `0` for the first variant declared and so on.
///     * For `enum`s with up to 256 variants, the discriminant is represented as a [`u8`]. For `enums` with 257 to 65536 variants, as a [`u16`], and so on.
/// * Then follow the `Protocol` representations of any fields of the `struct` or variant, in the order declared.
///
/// This representation can waste bandwidth for some types, e.g. `struct`s with multiple [`bool`] fields. For those, you may want to implement `Protocol` manually.
///
/// # Attributes
///
/// This macro's behavior can be modified using attributes. Multiple attributes can be specified as `#[async_proto(attr1, attr2, ...)]` or `#[async_proto(attr1)] #[async_proto(attr2)] ...`. The following attributes are available:
///
/// * `#[async_proto(attr(...))]`: Adds the given attribute(s) to the `Protocol` implementation. For example, the implementation can be documented using `#[async_proto(attr(doc = "..."))]`. May be specified multiple times.
/// * `#[async_proto(via = Proxy)]`: Implements `Protocol` for this type (let's call it `T`) in terms of another type (`Proxy` in this case) instead of using the variant- and field-based representation described above. `&'a T` must implement `Into<Proxy>` for all `'a`, and `Proxy` must implement `Protocol` and `TryInto<T>` with an `Error` type that implements `Into<ReadError>`.
///     * `#[async_proto(map_err = ...)]`: Removes the requirement for `<Proxy as TryInto<T>>::Error` to implement `Into<ReadError>` and instead uses the given expression (which should be an `FnOnce(<Proxy as TryInto<T>>::Error) -> ReadError`) to convert the error.
/// * `#[async_proto(where(...))]`: Overrides the bounds for the generated `Protocol` implementation. The default is to require `Protocol + Send + Sync + 'static` for each type parameter of this type.
///
/// # Compile errors
///
/// * This macro can't be used with `union`s.
#[proc_macro_derive(Protocol, attributes(async_proto))]
pub fn derive_protocol(input: TokenStream) -> TokenStream {
    let DeriveInput { attrs, ident, generics, data, .. } = parse_macro_input!(input);
    impl_protocol_inner(false, attrs, parse_quote!(#ident), generics, Some(data)).into()
}

struct ImplProtocolFor(Vec<(Vec<Attribute>, Path, Generics, Option<Data>)>);

impl Parse for ImplProtocolFor {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let mut decls = Vec::default();
        while !input.is_empty() {
            let attrs = Attribute::parse_outer(input)?;
            let lookahead = input.lookahead1();
            decls.push(if lookahead.peek(Token![enum]) {
                let enum_token = input.parse()?;
                let path = Path::parse_mod_style(input)?;
                let generics = input.parse()?;
                let content;
                let brace_token = braced!(content in input);
                let variants = content.parse_terminated(Variant::parse)?;
                (attrs, path, generics, Some(Data::Enum(DataEnum { enum_token, brace_token, variants })))
            } else if lookahead.peek(Token![struct]) {
                let struct_token = input.parse()?;
                let path = Path::parse_mod_style(input)?;
                let generics = input.parse()?;
                let lookahead = input.lookahead1();
                let fields = if lookahead.peek(Token![;]) {
                    Fields::Unit
                } else if lookahead.peek(Paren) {
                    let content;
                    let paren_token = parenthesized!(content in input);
                    let unnamed = content.parse_terminated(Field::parse_unnamed)?;
                    Fields::Unnamed(FieldsUnnamed { paren_token, unnamed })
                } else if lookahead.peek(Brace) {
                    let content;
                    let brace_token = braced!(content in input);
                    let named = content.parse_terminated(Field::parse_named)?;
                    Fields::Named(FieldsNamed { brace_token, named })
                } else {
                    return Err(lookahead.error())
                };
                let semi_token = input.peek(Token![;]).then(|| input.parse()).transpose()?;
                (attrs, path, generics, Some(Data::Struct(DataStruct { struct_token, fields, semi_token })))
            } else if lookahead.peek(Token![type]) {
                let _ = input.parse::<Token![type]>()?;
                let path = Path::parse_mod_style(input)?;
                let generics = input.parse()?;
                let _ = input.parse::<Token![;]>()?;
                (attrs, path, generics, None)
            } else {
                return Err(lookahead.error())
            });
        }
        Ok(ImplProtocolFor(decls))
    }
}

#[doc(hidden)]
#[proc_macro]
pub fn impl_protocol_for(input: TokenStream) -> TokenStream {
    let impls = parse_macro_input!(input as ImplProtocolFor)
        .0.into_iter()
        .map(|(attrs, path, generics, data)| impl_protocol_inner(true, attrs, path, generics, data));
    TokenStream::from(quote!(#(#impls)*))
}

struct Bitflags {
    name: Ident,
    repr: Ident,
}

impl Parse for Bitflags {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let name = input.parse()?;
        input.parse::<Token![:]>()?;
        let repr = input.parse()?;
        Ok(Self { name, repr })
    }
}

/// Implements `Protocol` for a type defined using the [`bitflags::bitflags`](https://docs.rs/bitflags/latest/bitflags/macro.bitflags.html) macro.
///
/// The type will be read via [`from_bits_truncate`](https://docs.rs/bitflags/latest/bitflags/example_generated/struct.Flags.html#method.from_bits_truncate), dropping any bits that do not correspond to flags.
///
/// # Usage
///
/// ```rust
/// bitflags::bitflags! {
///     struct Flags: u32 {
///         const A = 0b00000001;
///         const B = 0b00000010;
///         const C = 0b00000100;
///         const ABC = Self::A.bits | Self::B.bits | Self::C.bits;
///     }
/// }
///
/// async_proto::bitflags!(Flags: u32);
/// ```
#[proc_macro]
pub fn bitflags(input: TokenStream) -> TokenStream {
    let Bitflags { name, repr } = parse_macro_input!(input);
    let read_sync = if cfg!(feature = "read-sync") {
        quote! {
            fn read_sync(stream: &mut impl ::std::io::Read) -> ::core::result::Result<Self, ::async_proto::ReadError> {
                Ok(Self::from_bits_truncate(<#repr as ::async_proto::Protocol>::read_sync(stream)?))
            }
        }
    } else {
        quote!()
    };
    let write_sync = if cfg!(feature = "write-sync") {
        quote! {
            fn write_sync(&self, sink: &mut impl ::std::io::Write) -> ::core::result::Result<(), ::async_proto::WriteError> {
                <#repr as ::async_proto::Protocol>::write_sync(&self.bits(), sink)
            }
        }
    } else {
        quote!()
    };
    TokenStream::from(quote! {
        impl ::async_proto::Protocol for #name {
            fn read<'a, R: ::async_proto::tokio::io::AsyncRead + ::core::marker::Unpin + ::core::marker::Send + 'a>(stream: &'a mut R) -> ::std::pin::Pin<::std::boxed::Box<dyn ::std::future::Future<Output = ::core::result::Result<Self, ::async_proto::ReadError>> + ::core::marker::Send + 'a>> {
                ::std::boxed::Box::pin(async move {
                    Ok(Self::from_bits_truncate(<#repr as ::async_proto::Protocol>::read(stream).await?))
                })
            }

            fn write<'a, W: ::async_proto::tokio::io::AsyncWrite + ::core::marker::Unpin + ::core::marker::Send + 'a>(&'a self, sink: &'a mut W) -> ::std::pin::Pin<::std::boxed::Box<dyn ::std::future::Future<Output = ::core::result::Result<(), ::async_proto::WriteError>> + ::core::marker::Send + 'a>> {
                ::std::boxed::Box::pin(async move {
                    <#repr as ::async_proto::Protocol>::write(&self.bits(), sink).await
                })
            }

            #read_sync
            #write_sync
        }
    })
}
