#![deny(missing_docs, rust_2018_idioms, unused, unused_crate_dependencies, unused_import_braces, unused_lifetimes, unused_qualifications, warnings)]
#![forbid(unsafe_code)]

//! Procedural macros for the `async-proto` crate.

use {
    std::{
        convert::TryFrom as _,
        iter,
    },
    itertools::Itertools as _,
    proc_macro::TokenStream,
    proc_macro2::Span,
    quote::quote,
    syn::{
        Data,
        DataEnum,
        DataStruct,
        DeriveInput,
        Field,
        Fields,
        FieldsUnnamed,
        FieldsNamed,
        Ident,
        Path,
        PathArguments,
        PathSegment,
        Token,
        Variant,
        braced,
        parenthesized,
        parse::{
            Parse,
            ParseStream,
            Result,
        },
        parse_macro_input,
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
                .map(|Field { ty, .. }| {
                    quote!(<#ty as #async_proto_crate::Protocol>#read?)
                })
                .collect_vec();
            quote!((#(#read_fields,)*))
        }
        Fields::Named(FieldsNamed { named, .. }) => {
            let read_fields = named.iter()
                .map(|Field { ident, ty, .. }| {
                    quote!(#ident: <#ty as #async_proto_crate::Protocol>#read?)
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

fn impl_protocol_inner(internal: bool, qual_ty: Path, data: Data) -> proc_macro2::TokenStream {
    let async_proto_crate = if internal { quote!(crate) } else { quote!(::async_proto) };
    let (impl_read, impl_write, impl_read_sync, impl_write_sync) = match data {
        Data::Struct(DataStruct { fields, .. }) => {
            let fields_pat = fields_pat(&fields);
            let read_fields_async = read_fields(internal, false, &fields);
            let write_fields_async = write_fields(false, &fields);
            let read_fields_sync = read_fields(internal, true, &fields);
            let write_fields_sync = write_fields(true, &fields);
            (
                quote!(::core::result::Result::Ok(#qual_ty #read_fields_async)),
                quote! {
                    let #qual_ty #fields_pat = self;
                    #write_fields_async
                    ::core::result::Result::Ok(())
                },
                quote!(::core::result::Result::Ok(#qual_ty #read_fields_sync)),
                quote! {
                    let #qual_ty #fields_pat = self;
                    #write_fields_sync
                    ::core::result::Result::Ok(())
                },
            )
        }
        Data::Enum(DataEnum { variants, .. }) => {
            if variants.is_empty() {
                (
                    quote!(::core::result::Result::Err(#async_proto_crate::ReadError::ReadNever)),
                    quote!(match *self {}),
                    quote!(::core::result::Result::Err(#async_proto_crate::ReadError::ReadNever)),
                    quote!(match *self {}),
                )
            } else {
                let read_arms = variants.iter()
                    .enumerate()
                    .map(|(idx, Variant { ident: var, fields, .. })| {
                        let idx = u8::try_from(idx).expect("Protocol can't be derived for enums with more than u8::MAX variants");
                        let read_fields = read_fields(internal, false, fields);
                        quote!(#idx => ::core::result::Result::Ok(#qual_ty::#var #read_fields))
                    })
                    .collect_vec();
                let write_arms = variants.iter()
                    .enumerate()
                    .map(|(idx, Variant { ident: var, fields, .. })| {
                        let idx = u8::try_from(idx).expect("Protocol can't be derived for enums with more than u8::MAX variants");
                        let fields_pat = fields_pat(&fields);
                        let write_fields = write_fields(false, fields);
                        quote! {
                            #qual_ty::#var #fields_pat => {
                                #idx.write(sink).await?;
                                #write_fields
                            }
                        }
                    })
                    .collect_vec();
                let read_sync_arms = variants.iter()
                    .enumerate()
                    .map(|(idx, Variant { ident: var, fields, .. })| {
                        let idx = u8::try_from(idx).expect("Protocol can't be derived for enums with more than u8::MAX variants");
                        let read_fields = read_fields(internal, true, fields);
                        quote!(#idx => ::core::result::Result::Ok(#qual_ty::#var #read_fields))
                    })
                    .collect_vec();
                let write_sync_arms = variants.iter()
                    .enumerate()
                    .map(|(idx, Variant { ident: var, fields, .. })| {
                        let idx = u8::try_from(idx).expect("Protocol can't be derived for enums with more than u8::MAX variants");
                        let fields_pat = fields_pat(&fields);
                        let write_fields = write_fields(true, fields);
                        quote! {
                            #qual_ty::#var #fields_pat => {
                                #idx.write_sync(sink)?;
                                #write_fields
                            }
                        }
                    })
                    .collect_vec();
                (
                    quote! {
                        match <u8 as #async_proto_crate::Protocol>::read(stream).await? {
                            #(#read_arms,)*
                            n => ::core::result::Result::Err(#async_proto_crate::ReadError::UnknownVariant(n)),
                        }
                    },
                    quote! {
                        match self {
                            #(#write_arms,)*
                        }
                        ::core::result::Result::Ok(())
                    },
                    quote! {
                        match <u8 as #async_proto_crate::Protocol>::read_sync(stream)? {
                            #(#read_sync_arms,)*
                            n => ::core::result::Result::Err(#async_proto_crate::ReadError::UnknownVariant(n)),
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
        Data::Union(_) => return quote!(compile_error!("unions not supported in derive(Protocol)");).into(),
    };
    let read_sync = if cfg!(feature = "read-sync") {
        quote! {
            fn read_sync(mut stream: &mut impl ::std::io::Read) -> Result<Self, #async_proto_crate::ReadError> { #impl_read_sync }
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
    quote! {
        impl #async_proto_crate::Protocol for #qual_ty {
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
/// * For non-empty `enum`s, the representation starts with a single [`u8`] representing the variant, starting with `0` for the first variant declared and so on.
/// * Then follow the `Protocol` representations of any fields of the `struct` or variant, in the order declared.
///
/// This representation can waste bandwidth for some types, e.g. `struct`s with multiple [`bool`] fields. For those, you may want to implement `Protocol` manually.
///
/// # Compile errors
///
/// * This macro can't be used with `union`s.
/// * This macro currently can't be used with `enum`s with more than [`u8::MAX`] variants.
#[proc_macro_derive(Protocol)]
pub fn derive_protocol(input: TokenStream) -> TokenStream {
    let DeriveInput { ident, generics, data, .. } = parse_macro_input!(input as DeriveInput);
    if generics.lt_token.is_some() || generics.where_clause.is_some() { return quote!(compile_error!("generics not supported in derive(Protocol)");).into() } //TODO
    impl_protocol_inner(false, Path { leading_colon: None, segments: iter::once(PathSegment { ident, arguments: PathArguments::None }).collect() }, data).into()
}

struct ImplProtocolFor(Vec<(Path, Data)>);

impl Parse for ImplProtocolFor {
    fn parse(input: ParseStream<'_>) -> Result<ImplProtocolFor> {
        let mut decls = Vec::default();
        while !input.is_empty() {
            let lookahead = input.lookahead1();
            decls.push(if lookahead.peek(Token![enum]) {
                let enum_token = input.parse()?;
                let path = input.parse()?;
                let content;
                let brace_token = braced!(content in input);
                let variants = content.parse_terminated(Variant::parse)?;
                (path, Data::Enum(DataEnum { enum_token, brace_token, variants }))
            } else if lookahead.peek(Token![struct]) {
                let struct_token = input.parse()?;
                let path = input.parse()?;
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
                (path, Data::Struct(DataStruct { struct_token, fields, semi_token }))
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
        .map(|(path, data)| impl_protocol_inner(true, path, data));
    TokenStream::from(quote!(#(#impls)*))
}
