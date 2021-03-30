extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{self, Data, DeriveInput, Fields, parse_macro_input};

#[proc_macro_derive(Entity)]
pub fn entity_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    if let Data::Enum(data) = input.data {
        //data.variants.iter().map(|a|(&a.ident, &a.fields));
        let idents = data.variants.iter().map(|a|&a.ident);
        let idents2 = data.variants.iter().map(|a|&a.ident);
        let idents3 = data.variants.iter().map(|a|&a.ident);
        let idents4 = data.variants.iter().map(|a|&a.ident);
        let idents5 = data.variants.iter().map(|a|&a.ident);
        let name = input.ident;
        let expanded = quote! {
            impl Entity for #name {
                fn collide(&self, ray: &Ray) -> ColliderResult {
                    match self {
                        #( #name::#idents(a) => a.collide(ray),)*
                    }
                }
                fn material(&self) -> Option<&Material> {
                    match self {
                        #( #name::#idents2(a) => a.material(),)*
                    }
                }
                fn bounding_box(&self) -> AABB {
                    match self {
                        #( #name::#idents3(a) => a.bounding_box(),)*
                    }
                }
                fn position(&self) -> Point3<f64> {
                    match self {
                        #( #name::#idents4(a) => a.position(),)*
                    }
                }
                fn translate(&mut self, vec: Vector3<f64>) {
                    match self {
                        #( #name::#idents5(a) => a.translate(vec),)*
                    }
                }
            }
        };
        proc_macro::TokenStream::from(expanded)
    } else {
        panic!("Can only derive entity with an Enum");
    }
}