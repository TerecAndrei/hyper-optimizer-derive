use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(InputData, attributes(domain))]
pub fn input_data_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let data = match input.data {
        syn::Data::Struct(data) => data,
        _ => unimplemented!("The MyMacroHere derive macro is implemented only for structs"),
    };
    let name = input.ident;

    let mut get_domains_code = quote! {};
    let mut from_deserializer_code = quote! {};
    let mut field_names = quote! {};
    match data.fields {
        syn::Fields::Named(named) => {
            for field in named.named {
                let name = &field.ident.unwrap();
                let field_type = match field.ty {
                    syn::Type::Path(t) => t,
                    _ => unimplemented!(),
                };
                field_names.extend(quote! {
                    stringify!(#name),
                });
                let domain_field = field
                    .attrs
                    .into_iter()
                    .filter_map(|attr| match attr.meta {
                        syn::Meta::List(attr) if attr.path.is_ident("domain") => {
                            let args = match attr.parse_args::<syn::Expr>() {
                                Ok(v) => v,
                                Err(_) => {
                                    panic!("Expected a range.")
                                }
                            };
                            Some(args)
                        }
                        _ => None,
                    })
                    .collect::<Vec<_>>();
                if domain_field.len() == 0 {
                    panic!(
                        "Field {} should contain the attribute #[domain(range)]",
                        name.to_string(),
                    );
                }
                if domain_field.len() != 1 {
                    panic!(
                        "Field {} should contain only one domain field",
                        name.to_string(),
                    );
                }

                let domain = domain_field.into_iter().next().unwrap();
                let domain = match domain {
                    syn::Expr::Range(range) => range,
                    _ => panic!("This should be a range!"),
                };
                if field_type.path.is_ident("i32") {
                    from_deserializer_code.extend(quote! {
                        #name: deserializer.next_i32(),
                    });
                    get_domains_code.extend(quote! {
                        .add_i32_range(#domain, stringify!(#name).to_owned())
                    });
                } else if field_type.path.is_ident("f64") {
                    from_deserializer_code.extend(quote! {
                        #name: deserializer.next_f64(),
                    });
                    get_domains_code.extend(quote! {
                        .add_f64_range(#domain, stringify!(#name).to_owned())
                    });
                } else {
                    panic!("Data fields can only contain i32 and f32 fields!")
                }
            }
        }
        _ => unimplemented!("The struct needs to have named values!"),
    }
    let output = quote! {
        #[automatically_derived]
        impl hyper_optimizer::library::InputData for #name {
            fn get_domains(domains: hyper_optimizer::library::DomainBuilder) -> hyper_optimizer::library::DomainBuilder {
                domains
                    #get_domains_code
            }

            fn from_deserializer<T: Iterator<Item = hyper_optimizer::library::Value>>(mut deserializer: hyper_optimizer::library::InputDeserializer<T>) -> Self {
                Self {
                    #from_deserializer_code
                }
            }
        }
    };
    output.into()
}
