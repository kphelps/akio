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
use std::collections::HashMap;
use syn::parse::{block, expr, ident, ty};
use inflector::Inflector;

#[proc_macro]
pub fn actor(input: TokenStream) -> TokenStream {
    let source = input.to_string();
    let dsl_ast = parse_actor(&source).expect("Failed to parse actor DSL");
    let tokens = codegen_actor(&dsl_ast);
    //println!("{}", tokens);
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

#[derive(Clone, Debug)]
struct ActorState {
    fields: Vec<ActorStateField>,
}

impl ActorState {
    pub fn new(fields: Vec<ActorStateField>) -> Self {
        Self { fields: fields }
    }
}

#[derive(Clone, Debug)]
struct ActorStateField {
    name: syn::Ident,
    tipe: syn::Ty,
    maybe_init: Option<syn::Expr>,
}

impl ActorStateField {
    pub fn new(name: syn::Ident, tipe: syn::Ty, maybe_init: Option<syn::Expr>) -> Self {
        Self {
            name: name,
            tipe: tipe,
            maybe_init: maybe_init,
        }
    }

    pub fn as_struct_field(&self) -> syn::Field {
        syn::Field {
            ident: Some(self.name.clone()),
            vis: syn::Visibility::Public,
            attrs: Vec::new(),
            ty: self.tipe.clone(),
        }
    }

    pub fn as_arg(&self) -> Option<syn::BareFnArg> {
        if self.maybe_init.is_none() {
            Some(syn::BareFnArg {
                     name: Some(self.name.clone()),
                     ty: self.tipe.clone(),
                 })
        } else {
            None
        }
    }

    pub fn as_struct_field_value(&self) -> syn::FieldValue {
        let expr = if self.maybe_init.is_none() {
            syn::Expr {
                node: syn::ExprKind::Path(None,
                                          syn::Path {
                                              global: false,
                                              segments: vec![syn::PathSegment {
                                                                 ident: self.name.clone(),
                                                                 parameters:
                                                                     syn::PathParameters::none(),
                                                             }],
                                          }),
                attrs: Vec::new(),
            }
        } else {
            self.maybe_init.as_ref().unwrap().clone()
        };
        syn::FieldValue {
            ident: self.name.clone(),
            expr: expr,
            is_shorthand: false,
            attrs: Vec::new(),
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
enum LifecycleHookType {
    OnStart,
    OnStop,
}

impl LifecycleHookType {
    pub fn parse(input: &str) -> Option<Self> {
        match input {
            "on_start" => Some(LifecycleHookType::OnStart),
            "on_stop" => Some(LifecycleHookType::OnStop),
            _ => None,
        }
    }

    pub fn to_impl_name(&self) -> syn::Ident {
        match *self {
            LifecycleHookType::OnStart => syn::Ident::from("on_start_impl"),
            LifecycleHookType::OnStop => syn::Ident::from("on_stop_impl"),
        }
    }
}

#[derive(Debug)]
struct ActorLifecycleHook {
    hook_name: String,
    body: syn::Block,
}

impl ActorLifecycleHook {
    pub fn new(name: syn::Ident, body: syn::Block) -> Self {
        Self {
            hook_name: name.as_ref().to_string(),
            body: body,
        }
    }
}

#[derive(Debug)]
enum ActorBodyElement {
    Message(ActorMessage),
    State(ActorState),
    Hook(ActorLifecycleHook),
}

#[derive(Debug)]
struct ActorDefinition {
    name: syn::Ident,
    messages: Vec<ActorMessage>,
    state: Option<ActorState>,
    lifecycle_hooks: HashMap<LifecycleHookType, Vec<ActorLifecycleHook>>,
}

impl ActorDefinition {
    pub fn new(name: syn::Ident, body_elements: Vec<ActorBodyElement>) -> Self {
        let mut state = None;
        let mut messages = Vec::new();
        let mut hooks = HashMap::new();
        body_elements
            .into_iter()
            .for_each(|element| match element {
                          ActorBodyElement::Message(message) => messages.push(message),
                          ActorBodyElement::State(state_element) => {
                              if state.is_none() {
                                  state = Some(state_element);
                              } else {
                                  panic!("Only one state block is allowed in an actor definition");
                              }
                          }
                          ActorBodyElement::Hook(hook) => {
                              let maybe_hook_type = LifecycleHookType::parse(&hook.hook_name);
                              if maybe_hook_type.is_some() {
                                  let hook_type = maybe_hook_type.unwrap();
                                  hooks.entry(hook_type).or_insert(Vec::new()).push(hook)
                              } else {
                                  panic!("Invalid hook name: {}", hook.hook_name);
                              }
                          }
                      });
        Self {
            name: name,
            messages: messages,
            state: state,
            lifecycle_hooks: hooks,
        }
    }

    pub fn message_name(&self) -> syn::Ident {
        syn::Ident::from(format!("{}Message", self.name.as_ref()))
    }

    pub fn actor_ref_name(&self) -> syn::Ident {
        syn::Ident::from(format!("{}Ref", self.name.as_ref()))
    }

    pub fn state_name(&self) -> syn::Ident {
        syn::Ident::from(format!("{}State", self.name.as_ref()))
    }

    pub fn state_fields(&self) -> Vec<ActorStateField> {
        self.state
            .clone()
            .map(|state| state.fields)
            .unwrap_or(Vec::new())
    }

    pub fn state_struct_fields(&self) -> Vec<syn::Field> {
        self.state_fields()
            .iter()
            .map(ActorStateField::as_struct_field)
            .collect()
    }

    pub fn state_struct_field_values(&self) -> Vec<syn::FieldValue> {
        self.state_fields()
            .iter()
            .map(ActorStateField::as_struct_field_value)
            .collect()
    }

    pub fn state_field_args(&self) -> Vec<syn::BareFnArg> {
        self.state_fields()
            .iter()
            .filter_map(ActorStateField::as_arg)
            .collect()
    }

    pub fn state_field_uninitialized_names(&self) -> Vec<syn::Ident> {
        self.state_fields()
            .iter()
            .filter_map(|field| match field.maybe_init.as_ref() {
                            Some(_) => None,
                            None => Some(field.name.clone()),
                        })
            .collect()
    }
}

named! { parse_actor -> ActorDefinition,
    do_parse!(
        name: ident >> punct!(",") >>
        body: many0!(parse_body) >>

        (ActorDefinition::new(name, body))
    )
}

named! { parse_body -> ActorBodyElement,
    alt!(
        parse_state => { ActorBodyElement::State }
        |
        parse_message => { ActorBodyElement::Message }
        |
        parse_lifecycle_hook => { ActorBodyElement::Hook }
    )
}

named! { parse_lifecycle_hook -> ActorLifecycleHook,
    do_parse! (
        punct!("hook") >>
        name: ident >>
        body: block >>

        (ActorLifecycleHook::new(name, body))
    )
}

named! { parse_state -> ActorState,
    do_parse! (
        punct!("state") >>
        fields: delimited!(
            punct!("{"),
            terminated_list!(punct!(","), parse_state_field),
            punct!("}")
        ) >>

        (ActorState::new(fields))
    )
}

named! { parse_state_field -> ActorStateField,
    do_parse! (
        name: ident >>
        punct!(":") >>
        tipe: ty >>
        maybe_init: option!(preceded!(punct!("="), expr)) >>

        (ActorStateField::new(name, tipe, maybe_init))
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
    let state_name = dsl_ast.state_name();
    let state_struct = codegen_actor_state_struct(&dsl_ast);
    let state_args = &dsl_ast.state_field_args();
    let state_field_names = &dsl_ast.state_field_uninitialized_names();
    quote! {
        #state_struct

        struct #name {
            #[allow(dead_code)]
            state: #state_name,
        }

        impl #name {
            pub fn new(#(#state_args),*) -> Self {
                Self {
                    state: #state_name::new(#(#state_field_names),*)
                }
            }
        }
    }
}

fn codegen_actor_state_struct(dsl_ast: &ActorDefinition) -> quote::Tokens {
    let name = dsl_ast.state_name();
    let state_fields = dsl_ast.state_struct_fields();
    let state_args = dsl_ast.state_field_args();
    let state_field_values = dsl_ast.state_struct_field_values();

    quote! {
        struct #name {
            #(#state_fields,)*
        }

        impl #name {
            pub fn new(#(#state_args),*) -> Self {
                Self {
                    #(#state_field_values,)*
                }
            }
        }
    }
}

fn codegen_actor_impl(dsl_ast: &ActorDefinition) -> quote::Tokens {
    let name = &dsl_ast.name;
    let message_name = dsl_ast.message_name();
    let actor_ref_name = dsl_ast.actor_ref_name();
    let mod_name = syn::Ident::from(format!("_impl_actor_{}", name.as_ref().to_snake_case()));
    let state_field_args = dsl_ast.state_field_args();
    let state_field_names = dsl_ast.state_field_uninitialized_names();
    let hook_methods = codegen_hook_methods(&dsl_ast);
    let message_handlers = dsl_ast
        .messages
        .iter()
        .map(|message| codegen_message_handler(&message_name, message));
    quote!{
        mod #mod_name {
            use akio::{Actor, context, TypedActor};
            impl Actor for #name {
                type Message = #message_name;

                fn handle_message(&mut self, message: Self::Message)
                {
                    match message {
                        #(#message_handlers,)*
                    }
                }

                #hook_methods
            }

            impl TypedActor for #name {
                type RefType = #actor_ref_name;

                fn from_ref(actor_ref: &ActorRef) -> #actor_ref_name {
                    #actor_ref_name::new(actor_ref)
                }
            }

            #[allow(dead_code)]
            impl #name {
                pub fn spawn(id: Uuid, #(#state_field_args),*) -> #actor_ref_name
                {
                    context::with_mut(|ctx| {
                        let actor_ref = ctx.self_ref.spawn(id, #name::new(#(#state_field_names),*));
                        Self::from_ref(&actor_ref)
                    })
                }

                pub fn with_children<F, R>(&self, f: F) -> R
                    where F: FnOnce(&ActorChildren) -> R
                {
                    context::with(|ctx| ctx.self_ref.with_children(f))
                }

                pub fn sender_ref(&self) -> ActorRef {
                    context::with(|ctx| ctx.sender.clone())
                }

                pub fn sender<T: TypedActor>(&self) -> T::RefType {
                    context::with(|ctx| T::from_ref(&ctx.sender))
                }
            }
        }
    }
}

fn codegen_hook_methods(dsl_ast: &ActorDefinition) -> quote::Tokens {
    let hook_methods = dsl_ast
        .lifecycle_hooks
        .iter()
        .map(|(k, v)| codegen_lifecycle_hook_method(k, v))
        .collect::<Vec<quote::Tokens>>();

    quote! {
        #(#hook_methods)*
    }
}

fn codegen_lifecycle_hook_method(tipe: &LifecycleHookType,
                                 hooks: &Vec<ActorLifecycleHook>)
                                 -> quote::Tokens {
    let method_name = tipe.to_impl_name();
    let hook_blocks = hooks.iter().map(|hook| &hook.body);
    quote!{
        fn #method_name(&mut self) {
            #(#hook_blocks)*
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
            use akio::prelude::*;
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
    let ask_method = syn::Ident::from(format!("ask_{}", method_name.as_ref()));
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
            self.inner.send_from(
                #message_enum_name::#message_name(#(#field_arg_names,)*),
                akio_internal_sender
            );
        }

        pub fn #ask_method<T>(&self, #(#field_args,)*) -> Box<Future<Item = T, Error = ()> + Send>
            where T: Send + 'static
        {
            self.inner.ask::<T, #message_enum_name>(
                #message_enum_name::#message_name(#(#field_arg_names,)*),
            )
        }
    }
}
