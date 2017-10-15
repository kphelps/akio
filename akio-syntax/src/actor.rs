use inflector::Inflector;
use quote;
use std::collections::HashMap;
use syn;

struct ActorMessageMethod {
    method: syn::ImplItem,
}

impl ActorMessageMethod {
    pub fn new(item: syn::ImplItem) -> Self {
        Self {
            method: item,
        }
    }

    fn message_name(&self, actor_name: &syn::Ident) -> syn::Ident {
        syn::Ident::from(
            format!(
                "{}Message{}",
                actor_name,
                self.method_name().as_ref().to_pascal_case()
            )
        )
    }

    fn method_name(&self) -> syn::Ident {
        self.method.ident.clone()
    }

    pub fn message_struct(&self, actor_name: &syn::Ident) -> quote::Tokens {
        let name = self.message_name(&actor_name);
        let fields = self.fields();
        quote! {
            pub struct #name(#(#fields,)*);
        }
    }

    pub fn method(&self) -> quote::Tokens {
        let method_name = self.method_name();
        let inner_return_type = self.inner_return_type();
        let return_type = quote!{ ActorResponse<#inner_return_type> };
        let inputs = self.get_signature().decl.inputs.clone();
        let block = self.get_block();
        quote! {
            fn #method_name(#(#inputs,)*) -> #return_type #block
        }
    }

    fn method_arg_names(&self) -> Vec<syn::Ident> {
        self.fields()
            .iter()
            .enumerate()
            .map(|(i, _)| format!("temp_{}", i))
            .map(syn::Ident::from)
            .collect()
    }

    fn method_call_args(&self) -> Vec<syn::Expr> {
        let mut method_call_idents = vec![syn::Ident::from("self")];
        method_call_idents.extend(self.method_arg_names());
        method_call_idents
            .iter()
            .map(|ident| {
                syn::Expr {
                    attrs: Vec::new(),
                    node: syn::ExprKind::Path(None, syn::Path::from(ident.as_ref())),
                }
            })
            .collect()
    }

    fn method_call(&self) -> syn::Expr {
        syn::Expr {
            node: syn::ExprKind::MethodCall(
                self.method_name(),
                Vec::new(),
                self.method_call_args(),
            ),
            attrs: Vec::new(),
        }
    }

    pub fn handler_impl(
        &self,
        actor_name: syn::Ident,
    ) -> quote::Tokens {
        let message_name = self.message_name(&actor_name);
        let response_type = self.inner_return_type();
        let message_unpackers = self.fields()
            .iter()
            .enumerate()
            .map(|(i, _)| syn::Ident::from(format!("{}", i)))
            .collect::<Vec<syn::Ident>>();
        let method_name = self.method_name();
        quote! {
            impl MessageHandler<#message_name> for #actor_name {
                type Response = #response_type;

                fn handle(&mut self, message: #message_name)
                    -> ActorResponse<Self::Response>
                {
                    self.#method_name(#(message.#message_unpackers,)*)
                }
            }
        }
    }

    pub fn future_return_type(&self) -> quote::Tokens {
        let return_type = self.inner_return_type();
        quote! {
            Box<Future<Item = #return_type, Error = ()> + Send>
        }
    }

    pub fn inner_return_type(&self) -> quote::Tokens {
        match self.get_signature().decl.output {
            syn::FunctionRetTy::Default => quote!{ () },
            syn::FunctionRetTy::Ty(ty) => quote!{ #ty },
        }
    }

    pub fn ref_method_signatures(&self) -> quote::Tokens {
        let method_name = self.method_name();
        let send_method_name =
            syn::Ident::from(format!("send_{}", method_name.as_ref()));
        let return_type = self.future_return_type();
        let arg_names = &self.fields()
            .iter()
            .enumerate()
            .map(|(i, _)| format!("arg_{}", i))
            .map(syn::Ident::from)
            .collect::<Vec<syn::Ident>>();
        let args = &arg_names
            .clone()
            .into_iter()
            .zip(self.fields())
            .map(|(arg_name, field)| {
                syn::BareFnArg {
                    name: Some(arg_name),
                    ty: field.ty,
                }
            })
            .collect::<Vec<syn::BareFnArg>>();
        quote! {
            fn #method_name(&self, #(#args,)*) -> #return_type;
            fn #send_method_name(&self, #(#args,)*);
        }
    }

    pub fn ref_methods(&self, actor_name: &syn::Ident) -> quote::Tokens {
        let method_name = self.method_name();
        let send_method_name =
            syn::Ident::from(format!("send_{}", method_name.as_ref()));
        let return_type = self.future_return_type();
        let arg_names = &self.fields()
            .iter()
            .enumerate()
            .map(|(i, _)| format!("arg_{}", i))
            .map(syn::Ident::from)
            .collect::<Vec<syn::Ident>>();
        let args = &arg_names
            .clone()
            .into_iter()
            .zip(self.fields())
            .map(|(arg_name, field)| {
                syn::BareFnArg {
                    name: Some(arg_name),
                    ty: field.ty,
                }
            })
            .collect::<Vec<syn::BareFnArg>>();
        let message_name = self.message_name(actor_name);
        quote! {
            fn #method_name(&self, #(#args,)*) -> #return_type
            {
                Box::new(self.request(#message_name(#(#arg_names,)*)).flatten())
            }

            fn #send_method_name(&self, #(#args,)*) {
                self.send(#message_name(#(#arg_names,)*))
            }
        }
    }

    fn fields(&self) -> Vec<syn::Field> {
        let inputs = self.get_signature().decl.inputs;
        inputs
            .into_iter()
            .filter_map(|input| {
                match input {
                    syn::FnArg::Captured(_, tipe) => Some(tipe),
                    syn::FnArg::Ignored(tipe) => Some(tipe),
                    _ => None,
                }
            })
            .map(|tipe| {
                syn::Field {
                    ident: None,
                    vis: syn::Visibility::Inherited,
                    attrs: Vec::new(),
                    ty: tipe,
                }
            })
            .collect()
    }

    fn get_signature(&self) -> syn::MethodSig {
        match self.method.node {
            syn::ImplItemKind::Method(ref sig, _) => sig.clone(),
            _ => panic!("[actor_api] must decorate a method"),
        }
    }

    fn get_block(&self) -> syn::Block {
        match self.method.node {
            syn::ImplItemKind::Method(_, ref block) => block.clone(),
            _ => panic!("[actor_api] must decorate a method"),
        }
    }
}

#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
enum HookType {
    OnStart,
    OnStop,
}

impl HookType {
    pub fn has_marker(impl_item: &syn::ImplItem) -> bool {
        Self::get(impl_item).is_some()
    }

    pub fn get(impl_item: &syn::ImplItem) -> Option<Self> {
        if has_marker(impl_item, "on_start") {
            Some(HookType::OnStart)
        } else if has_marker(impl_item, "on_stop") {
            Some(HookType::OnStop)
        } else {
            None
        }
    }

    pub fn method_name(&self) -> syn::Ident {
        match *self {
            HookType::OnStart => syn::Ident::from("on_start"),
            HookType::OnStop => syn::Ident::from("on_stop"),
        }
    }
}

struct ActorImpl {
    // TODO: ignoring generics for now
    _generics: syn::Generics,
    tipe: syn::Ty,
    message_methods: Vec<ActorMessageMethod>,
    hook_methods: HashMap<HookType, Vec<ActorMessageMethod>>,
    rest: Vec<syn::ImplItem>,
}

fn has_marker(impl_item: &syn::ImplItem, name: &str) -> bool {
    impl_item
        .attrs
        .iter()
        .any(|attr| !attr.is_sugared_doc && attr.value.name() == name)
}

impl ActorImpl {
    fn new(generics: syn::Generics, tipe: syn::Ty, impl_items: Vec<syn::ImplItem>) -> Self {
        let mut message_methods = Vec::new();
        let mut hook_methods = HashMap::new();
        let mut rest = Vec::new();
        impl_items.into_iter().for_each(|mut item| {
            if has_marker(&item, "actor_api") {
                message_methods.push(ActorMessageMethod::new(item));
            } else if HookType::has_marker(&item) {
                item.ident = syn::Ident::from(format!("_hook_{}", item.ident.as_ref()));
                hook_methods
                    .entry(HookType::get(&item).unwrap())
                    .or_insert(Vec::new())
                    .push(ActorMessageMethod::new(item))
            } else {
                rest.push(item);
            }
        });
        Self {
            _generics: generics,
            tipe: tipe,
            message_methods: message_methods,
            hook_methods: hook_methods,
            rest: rest,
        }
    }

    fn name(&self) -> syn::Ident {
        match self.tipe {
            syn::Ty::Path(_, ref path) => path.segments.last().unwrap().ident.clone(),
            _ => panic!("Invalid actor name"),
        }
    }

    fn ref_name(&self) -> syn::Ident {
        syn::Ident::from(format!("{}Ref", self.name().as_ref()))
    }

    fn messages(&self) -> Vec<quote::Tokens> {
        self.message_methods
            .iter()
            .map(|message_method| message_method.message_struct(&self.name()))
            .collect()
    }

    fn ref_method_signatures(&self) -> Vec<quote::Tokens> {
        self.message_methods
            .iter()
            .map(|message_method| {
                message_method.ref_method_signatures()
            })
            .collect()
    }

    fn ref_methods(&self) -> Vec<quote::Tokens> {
        self.message_methods
            .iter()
            .map(|message_method| {
                message_method.ref_methods(&self.name())
            })
            .collect()
    }

    fn message_handler_impls(&self) -> Vec<quote::Tokens> {
        self.message_methods
            .iter()
            .map(|m| m.handler_impl(self.name()))
            .collect()
    }

    fn actor_impl(&self) -> Vec<quote::Tokens> {
        self.message_methods
            .iter()
            .chain(self.hook_methods.iter().flat_map(|(_, methods)| methods))
            .map(ActorMessageMethod::method)
            .chain(self.rest.iter().map(|x| quote!(#x)))
            .collect()
    }

    fn hook_methods(&self) -> Vec<quote::Tokens> {
        self.hook_methods
            .iter()
            .map(|(hook_type, methods)| {
                let hook_name = hook_type.method_name();
                let method_calls = methods.iter().map(ActorMessageMethod::method_call);
                quote! {
                    fn #hook_name(&mut self) {
                        #(#method_calls;)*
                    }
                }
            })
            .collect()
    }
}


pub fn codegen_actor_impl(ast: syn::Item) -> quote::Tokens {
    let actor = match ast.node {
        syn::ItemKind::Impl(_, _, generics, None, tipe, impl_items) => {
            ActorImpl::new(generics, *tipe, impl_items)
        }
        syn::ItemKind::Impl(_, _, _, Some(_), _, _) => {
            panic!("#[actor_impl] should be applied to an `impl ActorStruct {}` block")
        }
        _ => panic!("#[actor_impl] can only be used on `impl` blocks"),
    };
    let actor_name = actor.name();
    let actor_ref_name = actor.ref_name();
    let messages = actor.messages();
    let ref_method_signatures = actor.ref_method_signatures();
    let ref_methods = actor.ref_methods();
    let hook_methods = actor.hook_methods();
    let message_handler_impls = actor.message_handler_impls();
    let actor_impl = actor.actor_impl();
    let mod_name = syn::Ident::from(format!(
        "impl_module_{}",
        actor_name.as_ref().to_snake_case()
    ));
    quote!{
        mod #mod_name {
            pub trait #actor_ref_name {
                #(#ref_method_signatures)*
            }

            impl #actor_ref_name for ActorRef<#actor_name> {
                #(#ref_methods)*
            }

            #(#messages)*

            impl Actor for #actor_name {
                #(#hook_methods)*
            }

            impl #actor_name {
                #(#actor_impl)*
            }

            #(#message_handler_impls)*
        }
        pub use self::#mod_name::#actor_ref_name;
    }
}

pub fn codegen_actor_api(ast: syn::Item) -> quote::Tokens {
    quote!{ #ast }
}

pub fn codegen_actor_on_start(ast: syn::Item) -> quote::Tokens {
    quote!{ #ast }
}

pub fn codegen_actor_on_stop(ast: syn::Item) -> quote::Tokens {
    quote!{ #ast }
}
