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

    fn variant_name(&self) -> syn::Ident {
        syn::Ident::from(self.method_name().as_ref().to_pascal_case())
    }

    fn method_name(&self) -> syn::Ident {
        self.method.ident.clone()
    }

    pub fn message_enum(&self) -> syn::Variant {
        syn::Variant {
            ident: self.variant_name(),
            attrs: Vec::new(),
            data: syn::VariantData::Tuple(self.fields()),
            discriminant: None,
        }
    }

    pub fn method(&self) -> syn::ImplItem {
        syn::ImplItem {
            ident: self.method_name(),
            vis: syn::Visibility::Inherited,
            defaultness: syn::Defaultness::Final,
            attrs: Vec::new(),
            node: syn::ImplItemKind::Method(self.get_signature(), self.get_block()),
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

    pub fn delegator(&self, message_name: syn::Ident) -> syn::Arm {
        let message_path = syn::Path {
            global: false,
            segments: vec![
                syn::PathSegment {
                    ident: message_name,
                    parameters: syn::PathParameters::none(),
                },
                syn::PathSegment {
                    ident: self.variant_name(),
                    parameters: syn::PathParameters::none(),
                },
            ],
        };
        let temp_idents = self.method_arg_names();
        let temp_pats = temp_idents
            .iter()
            .map(|ident| {
                syn::Pat::Ident(
                    syn::BindingMode::ByValue(syn::Mutability::Immutable),
                    ident.clone(),
                    None,
                )
            })
            .collect::<Vec<syn::Pat>>();
        let pattern = syn::Pat::TupleStruct(message_path, temp_pats, None);
        let method_call = self.method_call();
        let arm = syn::Arm {
            attrs: Vec::new(),
            pats: vec![pattern],
            guard: None,
            body: Box::new(method_call),
        };
        arm
    }

    pub fn ref_methods(&self, message_enum_name: syn::Ident) -> quote::Tokens {
        let method_name = self.method_name();
        let method_with_sender_name =
            syn::Ident::from(format!("{}_with_sender", method_name.as_ref()));
        let ask_method = syn::Ident::from(format!("ask_{}", method_name.as_ref()));
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
        let message_name = self.variant_name();
        quote! {
            pub fn #method_name(&self, #(#args),*)  {
                context::with(|context| {
                    self.#method_with_sender_name(#(#arg_names,)* &context.self_ref);
                })
            }

            pub fn #method_with_sender_name(
                &self,
                #(#args,)*
                akio_internal_sender: &ActorRef
            ) {
                self.inner.send_from(
                    #message_enum_name::#message_name(#(#arg_names,)*),
                    akio_internal_sender
                );
            }

            pub fn #ask_method<T>(&self, #(#args,)*) -> impl Future<Item = T, Error = ()>
                where T: Send + 'static
            {
                self.inner.ask::<T, #message_enum_name>(
                    #message_enum_name::#message_name(#(#arg_names,)*),
                )
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
            HookType::OnStart => syn::Ident::from("on_start_impl"),
            HookType::OnStop => syn::Ident::from("on_stop_impl"),
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
        impl_items.into_iter().for_each(|item| {
            if has_marker(&item, "actor_api") {
                message_methods.push(ActorMessageMethod::new(item));
            } else if HookType::has_marker(&item) {
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

    fn message_name(&self) -> syn::Ident {
        syn::Ident::from(format!("{}Message", self.name().as_ref()))
    }

    fn messages(&self) -> Vec<syn::Variant> {
        self.message_methods
            .iter()
            .map(|message_method| message_method.message_enum())
            .collect()
    }

    fn ref_methods(&self) -> Vec<quote::Tokens> {
        self.message_methods
            .iter()
            .map(|message_method| {
                message_method.ref_methods(self.message_name())
            })
            .collect()
    }

    fn message_delegators(&self) -> Vec<syn::Arm> {
        self.message_methods
            .iter()
            .map(|m| m.delegator(self.message_name()))
            .collect()
    }

    fn actor_impl(&self) -> Vec<syn::ImplItem> {
        self.message_methods
            .iter()
            .chain(self.hook_methods.iter().flat_map(|(_, methods)| methods))
            .map(ActorMessageMethod::method)
            .chain(self.rest.clone())
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
    let message_name = actor.message_name();
    let messages = actor.messages();
    let ref_methods = actor.ref_methods();
    let hook_methods = actor.hook_methods();
    let message_delegators = actor.message_delegators();
    let actor_impl = actor.actor_impl();
    let mod_name = syn::Ident::from(format!(
        "impl_module_{}",
        actor_name.as_ref().to_snake_case()
    ));
    quote!{
        mod #mod_name {
            use std::ops::Deref;

            #[derive(Clone)]
            pub struct #actor_ref_name {
                inner: ActorRef,
            }

            impl #actor_ref_name {
                pub fn new(actor_ref: ActorRef) -> Self {
                    Self {
                        inner: actor_ref
                    }
                }
                #(#ref_methods)*
            }

            impl Deref for #actor_ref_name {
                type Target = ActorRef;

                fn deref(&self) -> &Self::Target {
                    &self.inner
                }
            }

            pub enum #message_name {
                #(#messages),*
            }

            impl Actor for #actor_name {
                type Message = #message_name;

                fn handle_message(&mut self, message: Self::Message) {
                    match message {
                        #(#message_delegators)*
                    }
                }

                #(#hook_methods)*
            }

            impl #actor_name {
                #(#actor_impl)*
            }

            impl TypedActor for #actor_name {
                type RefType = #actor_ref_name;

                fn from_ref(actor_ref: ActorRef) -> #actor_ref_name {
                    #actor_ref_name::new(actor_ref)
                }
            }
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
