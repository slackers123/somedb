use proc_macro::TokenStream;
use quote::quote;
use syn::Field;

#[proc_macro_derive(Storable)]
pub fn derive_storable(item: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(item as syn::DeriveInput);

    if input.generics.const_params().next().is_some()
        || input.generics.lifetimes().next().is_some()
        || input.generics.type_params().next().is_some()
    {
        panic!("Storables cannot have any generics.");
    }

    let ident = &input.ident;

    match input.data {
        syn::Data::Struct(s) => match s.fields {
            syn::Fields::Named(n) => {
                let names: Vec<_> = n.named.iter().map(|n| n.ident.as_ref().unwrap()).collect();
                let types: Vec<_> = n.named.iter().map(|n| n.ty.clone()).collect();

                quote! {
                    #[automatically_derived]
                    unsafe impl somedb::storable::Storable for #ident {
                        fn type_hash() -> somedb::type_hash::TypeHash {
                            use somedb::type_hash::TypeHash;
                            let field_names = &[#(stringify!(#names)),*];
                            let field_types = &[#(#types::type_hash()),*];

                            unsafe {
                                TypeHash::new(
                                    std::any::type_name::<Self>(),
                                    field_names,
                                    field_types,
                                )
                            }
                        }

                        fn inner_encoded(&self) -> Vec<u8> {
                            let mut bytes = Vec::new();
                            #(bytes.append(&mut self.#names.encoded());)*

                            bytes
                        }

                        fn decoded(mut reader: somedb::byte_reader::ByteReader) -> Self {
                            #(let #names = #types::decoded(reader.reader_for_block());)*
                            #ident {
                                #(#names),*
                            }
                        }
                    }
                }
                .into()
            }
            syn::Fields::Unit => panic!("Unit variants are not yet supported"),
            syn::Fields::Unnamed(_) => panic!("Unnamed fields are not yet supported"),
        },
        syn::Data::Enum(_) => panic!("Enums are not yet supported."),
        syn::Data::Union(_) => panic!("Unions are not yet supported."),
    }
}

#[proc_macro_derive(Entity, attributes(entity_id))]
pub fn derive_entity(item: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(item as syn::DeriveInput);

    if input.generics.const_params().next().is_some()
        || input.generics.lifetimes().next().is_some()
        || input.generics.type_params().next().is_some()
    {
        panic!("Entities cannot have any generics (yet).");
    }

    let ident = &input.ident;

    match input.data {
        syn::Data::Struct(s) => match s.fields {
            syn::Fields::Named(n) => {
                let id_field: &Field = n
                    .named
                    .iter()
                    .find(|f| {
                        f.attrs
                            .iter()
                            .find(|a| a.meta.path().is_ident("entity_id"))
                            .is_some()
                    })
                    .expect("there must be an Id");

                let generate_id = if id_field
                    .attrs
                    .iter()
                    .find(|a| a.meta.path().is_ident("entity_id"))
                    .unwrap()
                    .parse_nested_meta(|m| {
                        if m.path.is_ident("auto_generate") {
                            Ok(())
                        } else {
                            Err(m.error("invalid entity_id attribute"))
                        }
                    })
                    .is_ok()
                {
                    quote! {const GENERATE_ID: bool = true}
                } else {
                    quote! {const GENERATE_ID: bool = false}
                };
                let id_field_name = id_field.ident.clone().unwrap();
                let id_field_type = id_field.ty.clone();

                quote! {
                    #[automatically_derived]
                    impl somedb::entity::Entity for #ident {
                        type Id = #id_field_type;
                        #generate_id;

                        fn get_id(&self) -> #id_field_type {
                            self.#id_field_name
                        }

                        fn set_id(&mut self, id: Self::Id) {
                            self.#id_field_name = id;
                        }
                    }
                }
                .into()
            }
            syn::Fields::Unit => panic!("Unit variants are not yet supported"),
            syn::Fields::Unnamed(_) => panic!("Unnamed fields are not yet supported"),
        },
        syn::Data::Enum(_) => panic!("Enums are not yet supported."),
        syn::Data::Union(_) => panic!("Unions are not yet supported."),
    }
}

#[proc_macro_attribute]
pub fn entity(
    _metadata: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let input: proc_macro2::TokenStream = input.into();
    // FIXME: Clone should only be derived if it isn't already since
    //        this is really annoying right now.
    let output = quote! {
        #[derive(Clone, somedb::Storable, somedb::Entity)]
        #input
    };
    output.into()
}
