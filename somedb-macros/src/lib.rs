use proc_macro::TokenStream;
use quote::quote;

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
                    impl somedb::storable::Storable for #ident {
                        fn type_hash() -> somedb::type_hash::TypeHash {
                            use somedb::type_hash::TypeHash;
                            let field_names = &[#(stringify!(#names)),*];
                            let field_types = &[#(#types::type_hash()),*];

                            unsafe {
                                TypeHash::new(
                                    type_name::<Self>(),
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
