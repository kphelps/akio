#![feature(proc_macro)]
#![recursion_limit="128"]

extern crate inflector;
extern crate proc_macro;
#[macro_use]
extern crate quote;
extern crate syn;
#[macro_use]
extern crate synom;

use proc_macro::TokenStream;
use syn::parse::{block, ident, ty};
use inflector::Inflector;


#[proc_macro]
pub fn actor(input: TokenStream) -> TokenStream {
    let source = input.to_string();
    let dsl_ast = parse_actor(&source).expect("Failed to parse actor DSL");
    let tokens = codegen_actor(&dsl_ast);
    println!("{}", tokens);
    tokens.parse().unwrap()
}

#[derive(Debug)]
struct ActorMessageField {
    name: syn::Ident,
    tipe: syn::Ty,
}

impl ActorMessageField {
    pub fn new(name: syn::Ident, tipe: syn::Ty) -> Self {
        Self {
            name: name,
            tipe: tipe,
        }
    }
}

#[derive(Debug)]
struct ActorMessage {
    name: syn::Ident,
    fields: Vec<ActorMessageField>,
    body: syn::Block,
}

impl ActorMessage {
    pub fn new(name: syn::Ident, fields: Vec<ActorMessageField>, body: syn::Block) -> Self {
        Self {
            name: name,
            fields: fields,
            body: body,
        }
    }

    pub fn fields_as_args(&self) -> Vec<syn::BareFnArg> {
        self.fields
            .iter()
            .map(|field| {
                     syn::BareFnArg {
                         name: Some(field.name.clone()),
                         ty: field.tipe.clone(),
                     }
                 })
            .collect()
    }

    pub fn field_names(&self) -> Vec<&syn::Ident> {
        self.fields.iter().map(|field| &field.name).collect()
    }
}

#[derive(Debug)]
struct ActorDefinition {
    name: syn::Ident,
    messages: Vec<ActorMessage>,
}

impl ActorDefinition {
    pub fn new(name: syn::Ident, messages: Vec<ActorMessage>) -> Self {
        Self {
            name: name,
            messages: messages,
        }
    }

    pub fn message_name(&self) -> syn::Ident {
        syn::Ident::from(format!("{}Message", self.name.as_ref()))
    }

    pub fn actor_ref_name(&self) -> syn::Ident {
        syn::Ident::from(format!("{}Ref", self.name.as_ref()))
    }
}

named! { parse_actor -> ActorDefinition,
    do_parse!(
        name: ident >> punct!(",") >>
        body: many0!(parse_message) >>

        (ActorDefinition::new(name, body))
    )
}

named! { parse_message -> ActorMessage,
    do_parse! (
        punct!("message") >>
        name: ident >>
        args: delimited!(punct!("("), terminated_list!(punct!(","), parse_message_field), punct!(")")) >>
        body: block >>

        (ActorMessage::new(name, args, body))
    )
}

named! { parse_message_field -> ActorMessageField,
    do_parse! (
        name: ident >>
        punct!(":") >>
        tipe: ty >>

        (ActorMessageField::new(name, tipe))
    )
}

fn codegen_actor(dsl_ast: &ActorDefinition) -> quote::Tokens {
    let message_enum = codegen_message_enum(dsl_ast);
    let actor_struct = codegen_actor_struct(dsl_ast);
    let actor_impl = codegen_actor_impl(dsl_ast);
    let actor_ref = codegen_ref(dsl_ast);
    quote!{
        #message_enum
        #actor_struct
        #actor_impl
        #actor_ref
    }
}

fn codegen_message_enum(dsl_ast: &ActorDefinition) -> quote::Tokens {
    let enum_name = dsl_ast.message_name();
    let message_variants = dsl_ast.messages.iter().map(codegen_message_variant);
    quote! {
        enum #enum_name {
            #(#message_variants,)*
        }
    }
}

fn codegen_message_variant(message_ast: &ActorMessage) -> quote::Tokens {
    let tipes = message_ast.fields.iter().map(|field| &field.tipe);
    let name = &message_ast.name;
    quote! {
        #name(#(#tipes,)*)
    }
}

fn codegen_actor_struct(dsl_ast: &ActorDefinition) -> quote::Tokens {
    let name = &dsl_ast.name;
    quote! {
        struct #name {
        }
    }
}

fn codegen_actor_impl(dsl_ast: &ActorDefinition) -> quote::Tokens {
    let name = &dsl_ast.name;
    let message_name = dsl_ast.message_name();
    let actor_ref_name = dsl_ast.actor_ref_name();
    let mod_name = syn::Ident::from(format!("_impl_actor_{}", name.as_ref().to_snake_case()));
    let message_handlers = dsl_ast
        .messages
        .iter()
        .map(|message| codegen_message_handler(&message_name, message));
    quote!{
        mod #mod_name {
            use akio::{Actor, ActorFactory, context, TypedActor};
            use futures::prelude::*;
            impl Actor for #name {
                type Message = #message_name;

                fn handle_message(&mut self, message: Self::Message)
                    -> Box<Future<Item = (), Error = ()>>
                {
                    match message {
                        #(#message_handlers,)*
                    }
                }
            }

            impl TypedActor for #name {
                type RefType = #actor_ref_name;

                fn from_ref(actor_ref: &ActorRef) -> #actor_ref_name {
                    #actor_ref_name::new(actor_ref)
                }
            }

            impl #name {
                pub fn spawn(id: Uuid)
                    -> Box<Future<Item = #actor_ref_name, Error = ()> + Send>
                {
                    Box::new(
                        context::with_mut(|ctx| {
                            ctx.spawn(id, #name{})
                                .map(|actor_ref| Self::from_ref(&actor_ref))
                        })
                    )
                }

                pub fn with_children<F, R>(&self, f: F) -> R
                    where F: FnOnce(&ActorChildren) -> R
                {
                    context::with(|ctx| f(&ctx.children))
                }

                pub fn sender<T: TypedActor>(&self) -> T::RefType {
                    context::with(|ctx| T::from_ref(&ctx.sender))
                }
            }
        }
    }
}

fn codegen_message_handler(message_enum_name: &syn::Ident,
                           message_ast: &ActorMessage)
                           -> quote::Tokens {
    let message_name = &message_ast.name;
    let message_field_names = message_ast.field_names();
    let message_body = &message_ast.body;
    quote! {
        #message_enum_name::#message_name(#(#message_field_names,)*) => {
            #message_body
        }
    }
}

fn codegen_ref(dsl_ast: &ActorDefinition) -> quote::Tokens {
    let name = dsl_ast.actor_ref_name();
    let mod_name = syn::Ident::from(format!("_impl_actor_ref_{}", name.as_ref().to_snake_case()));
    let message_methods =
        dsl_ast
            .messages
            .iter()
            .map(|message| codegen_message_method(&dsl_ast.message_name(), message));
    quote! {
        mod #mod_name {
            use akio::ActorRef;
            use std::ops::Deref;
            use akio::context;
            pub struct #name {
                inner: ActorRef,
            }
            impl #name {
                pub fn new(actor_ref: &ActorRef) -> Self {
                    Self {
                        inner: actor_ref.clone()
                    }
                }
                #(#message_methods)*
            }
            impl Deref for #name {
                type Target = ActorRef;
                fn deref(&self) -> &Self::Target {
                    &self.inner
                }
            }
        }
        pub use #mod_name::#name;
    }
}

fn codegen_message_method(message_enum_name: &syn::Ident,
                          message_ast: &ActorMessage)
                          -> quote::Tokens {
    let message_name = &message_ast.name;
    let method_name = syn::Ident::from(message_name.as_ref().to_snake_case());
    let method_with_sender_name = syn::Ident::from(format!("{}_with_sender", method_name.as_ref()));
    let field_args = &message_ast.fields_as_args();
    let field_arg_names = &message_ast.field_names();
    quote! {
        pub fn #method_name(&self, #(#field_args,)*) {
            context::with(|context| {
                self.#method_with_sender_name(#(#field_arg_names,)* &context.self_ref);
            })
        }

        pub fn #method_with_sender_name(
            &self,
            #(#field_args,)*
            akio_internal_sender: &ActorRef
        ) {
            self.inner.send(
                #message_enum_name::#message_name(#(#field_arg_names,)*),
                akio_internal_sender
            );
        }
    }
}
