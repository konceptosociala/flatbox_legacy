use proc_macro::{self, TokenStream};
use syn::{parse_macro_input, DeriveInput};

mod material;

use self::material::derive_material_internal;

#[proc_macro_derive(Material, attributes(material, texture, color))]
pub fn derive_material(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let output = derive_material_internal(input);

    output.into()
}