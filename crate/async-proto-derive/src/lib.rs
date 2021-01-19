//! Procedural macros for the `async-proto` crate.

#![deny(missing_docs, rust_2018_idioms, unused, unused_crate_dependencies, unused_import_braces, unused_lifetimes, unused_qualifications, warnings)]
#![forbid(unsafe_code)]

use {
    std::convert::TryFrom as _,
    convert_case::{
        Case,
        Casing as _,
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
        Variant,
        parse_macro_input,
    },
};

fn read_fields(sync: bool, read_error: &Ident, read_error_variants: &mut Vec<proc_macro2::TokenStream>, read_error_display_arms: &mut Vec<proc_macro2::TokenStream>, var: Option<&Ident>, fields: &Fields) -> proc_macro2::TokenStream {
    let read = if sync { quote!(::read_sync(stream)?) } else { quote!(::read(stream).await?) };
    match fields {
        Fields::Unit => quote!(),
        Fields::Unnamed(FieldsUnnamed { unnamed, .. }) => {
            let read_fields = unnamed.iter()
                .enumerate()
                .map(|(idx, Field { ty, .. })| {
                    let variant_name = Ident::new(&format!("{}Field{}", var.map(|var| var.to_string()).unwrap_or_default(), idx), Span::call_site());
                    read_error_variants.push(quote!(#variant_name(<#ty as ::async_proto::Protocol>::ReadError)));
                    read_error_display_arms.push(quote!(#read_error::#variant_name(e) => e.fmt(f)));
                    quote!(<#ty as ::async_proto::Protocol>#read)
                })
                .collect_vec();
            quote!((#(#read_fields,)*))
        }
        Fields::Named(FieldsNamed { named, .. }) => {
            let read_fields = named.iter()
                .map(|Field { ident, ty, .. }| {
                    let variant_name = Ident::new(&format!("{}{}", var.map(|var| var.to_string()).unwrap_or_default(), ident.as_ref().expect("missing ident in named field").to_string().to_case(Case::Pascal)), Span::call_site());
                    read_error_variants.push(quote!(#variant_name(<#ty as ::async_proto::Protocol>::ReadError)));
                    read_error_display_arms.push(quote!(#read_error::#variant_name(e) => e.fmt(f)));
                    quote!(#ident: <#ty as ::async_proto::Protocol>#read)
                })
                .collect_vec();
            quote!({ #(#read_fields,)* })
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

/// Implements the `Protocol` trait for this type.
///
/// The network representation is very simple:
///
/// * For `enum`s, it starts with a single [`u8`] representing the variant, starting with `0` for the first variant declared and so on.
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
    let DeriveInput { ident: ty, generics, data, .. } = parse_macro_input!(input as DeriveInput);
    if generics.lt_token.is_some() || generics.where_clause.is_some() { return quote!(compile_error!("generics not supported in derive(Protocol)")).into() }
    let read_error = Ident::new(&format!("{}ReadError", ty), Span::call_site());
    let (read_error_variants, read_error_display_arms, impl_read, impl_write, impl_read_sync, impl_write_sync) = match data {
        Data::Struct(DataStruct { fields, .. }) => {
            let mut read_error_variants = Vec::default();
            let mut read_error_display_arms = Vec::default();
            let read_fields_async = read_fields(false, &read_error, &mut read_error_variants, &mut read_error_display_arms, None, &fields);
            let write_fields_async = write_fields(false, &fields);
            let read_fields_sync = read_fields(true, &read_error, &mut read_error_variants, &mut read_error_display_arms, None, &fields);
            let write_fields_sync = write_fields(true, &fields);
            (
                read_error_variants,
                read_error_display_arms,
                quote!(::core::result::Result::Ok(#ty #read_fields_async)),
                quote! {
                    #write_fields_async
                    ::core::result::Result::Ok(())
                },
                quote!(::core::result::Result::Ok(#ty #read_fields_sync)),
                quote! {
                    #write_fields_sync
                    ::core::result::Result::Ok(())
                },
            )
        }
        Data::Enum(DataEnum { variants, .. }) => {
            let mut read_error_variants = vec![quote!(UnknownVariant(u8))];
            let mut read_error_display_arms = vec![quote!(#read_error::UnknownVariant(n) => write!(f, "unknown variant: {}", n))];
            let read_arms = variants.iter()
                .enumerate()
                .map(|(idx, Variant { ident: var, fields, .. })| {
                    let idx = u8::try_from(idx).expect("Protocol can't be derived for enums with more than u8::MAX variants");
                    let read_fields = read_fields(false, &read_error, &mut read_error_variants, &mut read_error_display_arms, Some(var), fields);
                    quote!(#idx => ::core::result::Result::Ok(#ty::#var #read_fields))
                })
                .collect_vec();
            let write_arms = variants.iter()
                .enumerate()
                .map(|(idx, Variant { ident: var, fields, .. })| {
                    let idx = u8::try_from(idx).expect("Protocol can't be derived for enums with more than u8::MAX variants");
                    let write_fields = write_fields(false, fields);
                    quote! {
                        #ty::#var => {
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
                    let read_fields = read_fields(true, &read_error, &mut read_error_variants, &mut read_error_display_arms, Some(var), fields);
                    quote!(#idx => ::core::result::Result::Ok(#ty::#var #read_fields))
                })
                .collect_vec();
            let write_sync_arms = variants.iter()
                .enumerate()
                .map(|(idx, Variant { ident: var, fields, .. })| {
                    let idx = u8::try_from(idx).expect("Protocol can't be derived for enums with more than u8::MAX variants");
                    let write_fields = write_fields(true, fields);
                    quote! {
                        #ty::#var => {
                            #idx.write(sink).await?;
                            #write_fields
                        }
                    }
                })
                .collect_vec();
            (
                read_error_variants,
                read_error_display_arms,
                quote! {
                    match <u8 as ::async_proto::Protocol>::read(stream).await? {
                        #(#read_arms,)*
                        n => ::core::result::Result::Err(#read_error::UnknownVariant(n)),
                    }
                },
                quote! {
                    match self {
                        #(#write_arms,)*
                    }
                    ::core::result::Result::Ok(())
                },
                quote! {
                    match <u8 as ::async_proto::Protocol>::read_sync(stream)? {
                        #(#read_sync_arms,)*
                        n => ::core::result::Result::Err(#read_error::UnknownVariant(n)),
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
        Data::Union(_) => return quote!(compile_error!("unions not supported in derive(Protocol)")).into(),
    };
    TokenStream::from(quote! {
        #[derive(Debug, ::async_proto::derive_more::From)]
        pub enum #read_error {
            Io(::std::io::Error),
            #(#read_error_variants,)*
        }

        impl ::std::fmt::Display for #read_error {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                match self {
                    #read_error::Io(e) => write!(f, "I/O error: {}", e),
                    #(#read_error_display_arms,)*
                }
            }
        }

        #[cfg(not(target_arch = "wasm32"))]
        #[::async_proto::async_trait::async_trait]
        impl ::async_proto::Protocol for #ty {
            type ReadError = #read_error;

            async fn read<'a, R: ::async_proto::tokio::io::AsyncRead + ::core::marker::Unpin + ::core::marker::Send + 'a>(stream: R) -> Result<#ty, #read_error> { #impl_read }
            async fn write<'a, W: ::async_proto::tokio::io::AsyncWrite + ::core::marker::Unpin + ::core::marker::Send + 'a>(&'a self, sink: W) -> ::std::io::Result<()> { #impl_write }
            fn read_sync<'a>(stream: impl ::std::io::Read + 'a) -> Result<Self, Self::ReadError> { #impl_read_sync }
            fn write_sync<'a>(&'a self, sink: impl ::std::io::Write + 'a) -> ::std::io::Result<()> { #impl_write_sync }
        }
    })
}
