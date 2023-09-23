use core::panic;

use darling::FromDeriveInput;
use proc_macro2::{Ident, Span};
use quote::{quote, ToTokens};
use syn::{Data, DeriveInput, Type, Meta, DataStruct, Field};

enum FieldAttribute {
    Texture,
    Color,
    None,
}

#[derive(FromDeriveInput)]
#[darling(attributes(material))]
struct Opts {
    vertex: Option<String>,
    fragment: Option<String>,
    topology: Option<String>,
}

pub(crate) fn derive_material_internal(input: DeriveInput) -> proc_macro2::TokenStream {
    let opts = Opts::from_derive_input(&input).expect("Wrong options");
    let ident = input.ident;

    let Data::Struct(data) = input.data else {
        panic!("Cannot make non-struct type into Material!")
    };

    let vertex = get_vertex_path(&opts);
    let fragment = get_fragment_path(&opts);
    let input = get_shader_input(&opts, &data);

    let ident_builder = get_builder_struct_name(&ident);
    let fields = data.fields.iter().map(get_builder_struct_field);
    let functions = data.fields.iter().map(get_builder_impl_function);
    let build_function = data.fields.iter().map(get_builder_impl_build);

    quote! {
        #[::flatbox::assets::typetag::serde]
        impl ::flatbox::render::Material for #ident {
            #vertex
            #fragment
            #input
        }

        impl #ident {
            pub fn builder() -> #ident_builder {
                #ident_builder::new()
            }
        }

        #[derive(Clone, Default, Debug)]
        pub struct #ident_builder {
            #(#fields,)*
        }

        impl #ident_builder {
            pub fn new() -> Self {
                Self::default()
            }

            pub fn build(self) -> #ident { 
                #ident {
                    #(#build_function)*
                }
            }

            #(#functions)*
        }
    }
}

fn get_vertex_path(opts: &Opts) -> proc_macro2::TokenStream {
    match &opts.vertex {
        Some(path) => quote! {
            fn vertex() -> &'static [u32] {
                ::flatbox::render::include_glsl!(
                    #path, 
                    kind: vert,
                )
            }
        },
        None => panic!("Use proc macro attribute #[material(vertex = \"path\")] to set vertex shader path"),
    }
}

fn get_fragment_path(opts: &Opts) -> proc_macro2::TokenStream {
    match &opts.fragment {
        Some(path) => quote! {
            fn fragment() -> &'static [u32] {
                ::flatbox::render::include_glsl!(
                    #path, 
                    kind: frag,
                )
            }
        },
        None => panic!("Use proc macro attribute #[material(fragment = \"path\")] to set fragment shader path"),
    }
}

fn get_builder_struct_name(ident: &Ident) -> Ident {
    Ident::new(format!("{}Builder", ident.to_string()).as_str(), Span::call_site())
}

fn get_builder_struct_field(f: &Field) -> proc_macro2::TokenStream {
    let name = &f.ident;
    let ty = &f.ty;
    quote! {
        #name: #ty
    }
}

fn get_builder_impl_function(f: &Field) -> proc_macro2::TokenStream {
    let name = &f.ident;
    let ty = &f.ty;
    let attr = match f.attrs.get(0) {
        Some(attr) => {
            if let Meta::Path(path) = &attr.meta {
                match path.into_token_stream().to_string().as_str() {
                    "color" => FieldAttribute::Color,
                    "texture" => FieldAttribute::Texture,
                    _ => panic!("Invalid field attribute: \"{}\"", attr.into_token_stream().to_string().as_str()),
                }
            } else {
                panic!("Invalid field attribute: \"{}\"", attr.into_token_stream().to_string().as_str());
            }
        },
        None => FieldAttribute::None,
    };

    match attr {
        FieldAttribute::Color => quote! {
            pub fn #name(mut self, value: ::flatbox::render::Color<f32>) -> Self { 
                self.#name = value.into();
                self
            }
        },
        FieldAttribute::Texture => quote! {
            pub fn #name(mut self, value: ::flatbox::assets::AssetHandle<'T'>) -> Self { 
                self.#name = value.into();
                self
            }
        },
        _ => quote! {
            pub fn #name(mut self, value: #ty) -> Self { 
                self.#name = value;
                self
            }
        },
    }
}

fn get_builder_impl_build(f: &Field) -> proc_macro2::TokenStream {
    let name = &f.ident;
    quote! {
        #name: self.#name,
    }
}

fn get_shader_input(
    opts: &Opts,
    data: &DataStruct,
) -> proc_macro2::TokenStream {
    let topology = match &opts.topology {
        Some(topology) => match topology.as_str() {
            "point_list" => quote! { ::flatbox::render::ShaderTopology::POINT_LIST },
            "line_list" => quote! { ::flatbox::render::ShaderTopology::LINE_LIST },
            "line_strip" => quote! { ::flatbox::render::ShaderTopology::LINE_STRIP },
            "triangle_list" => quote! { ::flatbox::render::ShaderTopology::TRIANGLE_LIST },
            "triangle_strip" => quote! { ::flatbox::render::ShaderTopology::TRIANGLE_STRIP },
            "triangle_fan" => quote! { ::flatbox::render::ShaderTopology::TRIANGLE_FAN },
            "line_list_with_adjacency" => quote! { ::flatbox::render::ShaderTopology::LINE_LIST_WITH_ADJACENCY },
            "line_strip_with_adjacency" => quote! { ::flatbox::render::ShaderTopology::LINE_STRIP_WITH_ADJACENCY },
            "triangle_list_with_adjacency" => quote! { ::flatbox::render::ShaderTopology::TRIANGLE_LIST_WITH_ADJACENCY },
            "triangle_strip_with_adjacency" => quote! { ::flatbox::render::ShaderTopology::TRIANGLE_STRIP_WITH_ADJACENCY },
            _ => panic!("Unsupported topology \"{}\"", topology),
        },
        None => quote! { ::flatbox::render::ShaderTopology::TRIANGLE_LIST },
    };

    let format = data.fields.iter().map(|f| {
        let ty = &f.ty;
        match ty {
            Type::Array(array) => {
                match array.into_token_stream().to_string().as_str() {
                    "[f32 ; 1]" => quote! { ::flatbox::render::ShaderInputFormat::R32_SFLOAT },
                    "[f32 ; 2]" => quote! { ::flatbox::render::ShaderInputFormat::R32G32_SFLOAT },
                    "[f32 ; 3]" => quote! { ::flatbox::render::ShaderInputFormat::R32G32B32_SFLOAT },
                    "[f32 ; 4]" => quote! { ::flatbox::render::ShaderInputFormat::R32G32B32A32_SFLOAT },
                    _ => panic!("Unsupported input format: \"{}\"", array.into_token_stream().to_string().as_str())
                }
            },
            Type::Path(path) => {
                match path.into_token_stream().to_string().as_str() {
                    "f32" => quote! { ::flatbox::render::ShaderInputFormat::R32_SFLOAT },
                    "u32" => quote! { ::flatbox::render::ShaderInputFormat::R8G8B8A8_UINT },
                    "i32" => quote! { ::flatbox::render::ShaderInputFormat::R8G8B8A8_SINT },
                    _ => panic!("Unsupported input format: \"{}\"", path.into_token_stream().to_string().as_str())
                }
            },
            _ => panic!("Unsupported input format"),
        }
    });

    quote! {
        fn input() -> ::flatbox::render::ShaderInput {
            let mut location = 3;
            let mut offset = 0;
            let mut attributes = vec![];
            #(
                attributes.push(
                    ::flatbox::render::ShaderInputAttribute{
                        binding: 1,
                        location: location,
                        offset: offset,
                        format: #format,
                    }
                );

                offset += match #format {
                    ::flatbox::render::ShaderInputFormat::R8G8B8A8_UINT
                        | ::flatbox::render::ShaderInputFormat::R8G8B8A8_SINT 
                        | ::flatbox::render::ShaderInputFormat::R32_SFLOAT => 4,
                    ::flatbox::render::ShaderInputFormat::R32G32_SFLOAT => 8,
                    ::flatbox::render::ShaderInputFormat::R32G32B32_SFLOAT => 12,
                    ::flatbox::render::ShaderInputFormat::R32G32B32A32_SFLOAT => 16,
                    _ => 0,
                };

                location += 1;
            )*
            let instance_size = offset as usize;

            ::flatbox::render::ShaderInput {
                attributes,
                instance_size,
                topology: #topology,
            }
        }
    }
}