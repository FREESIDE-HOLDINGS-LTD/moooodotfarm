extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn, ReturnType, Type, TypeParamBound};


#[proc_macro_attribute]
pub fn application_handler(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);

    let func_name = &input.sig.ident;
    let block = &input.block;
    let visibility = &input.vis;
    let sig = &input.sig;
    let is_async = input.sig.asyncness.is_some();
    let returns_future = output_is_future(&input.sig);

    let expanded = if is_async {
        let call_expr = quote! { { #block } };
        quote! {
            #visibility #sig {
                let type_name_full = std::any::type_name::<Self>();
                let type_name = type_name_full
                    .split('<')
                    .next()
                    .unwrap_or(type_name_full)
                    .rsplit("::")
                    .next()
                    .unwrap_or(type_name_full);
                let start = std::time::Instant::now();
                let result = #call_expr;
                let duration = start.elapsed();
                match &result {
                    Err(error) => {
                        self.metrics.record_application_handler_call(type_name, crate::app::ApplicationHandlerCallResult::Error, crate::domain::time::Duration::new_from_std(duration));
                        log::error!("Application layer call failed: function=`{}` duration=`{:.2?}` error=`{}`", stringify!(#func_name), duration, error.to_string());
                    },
                    Ok(_) => {
                        self.metrics.record_application_handler_call(type_name, crate::app::ApplicationHandlerCallResult::Ok, crate::domain::time::Duration::new_from_std(duration));
                        log::debug!("Application layer call succeeded: function=`{}` duration=`{:.2?}`", stringify!(#func_name), duration);
                    }
                }
                result
            }
        }
    } else if returns_future {
        quote! {
            #visibility #sig {
                async move {
                    let type_name_full = std::any::type_name::<Self>();
                    let type_name = type_name_full
                        .split('<')
                        .next()
                        .unwrap_or(type_name_full)
                        .rsplit("::")
                        .next()
                        .unwrap_or(type_name_full);
                    let start = std::time::Instant::now();
                    let result = (|| #block)().await;
                    let duration = start.elapsed();
                    match &result {
                        Err(error) => {
                            self.metrics.record_application_handler_call(type_name, crate::app::ApplicationHandlerCallResult::Error, crate::domain::time::Duration::new_from_std(duration));
                            log::error!("Application layer call failed: function=`{}` duration=`{:.2?}` error=`{}`", stringify!(#func_name), duration, error.to_string());
                        },
                        Ok(_) => {
                            self.metrics.record_application_handler_call(type_name, crate::app::ApplicationHandlerCallResult::Ok, crate::domain::time::Duration::new_from_std(duration));
                            log::debug!("Application layer call succeeded: function=`{}` duration=`{:.2?}`", stringify!(#func_name), duration);
                        }
                    }
                    result
                }
            }
        }
    } else {
        let call_expr = quote! { (|| #block)() };
        quote! {
            #visibility #sig {
                let type_name_full = std::any::type_name::<Self>();
                let type_name = type_name_full
                    .split('<')
                    .next()
                    .unwrap_or(type_name_full)
                    .rsplit("::")
                    .next()
                    .unwrap_or(type_name_full);
                let start = std::time::Instant::now();
                let result = #call_expr;
                let duration = start.elapsed();
                match &result {
                    Err(error) => {
                        self.metrics.record_application_handler_call(type_name, crate::app::ApplicationHandlerCallResult::Error, crate::domain::time::Duration::new_from_std(duration));
                        log::error!("Application layer call failed: function=`{}` duration=`{:.2?}` error=`{}`", stringify!(#func_name), duration, error.to_string());
                    },
                    Ok(_) => {
                        self.metrics.record_application_handler_call(type_name, crate::app::ApplicationHandlerCallResult::Ok, crate::domain::time::Duration::new_from_std(duration));
                        log::debug!("Application layer call succeeded: function=`{}` duration=`{:.2?}`", stringify!(#func_name), duration);
                    }
                }
                result
            }
        }
    };

    TokenStream::from(expanded)
}

fn output_is_future(sig: &syn::Signature) -> bool {
    match &sig.output {
        ReturnType::Type(_, ty) => is_impl_future(ty),
        ReturnType::Default => false,
    }
}

fn is_impl_future(ty: &Type) -> bool {
    let impl_trait = match ty {
        Type::ImplTrait(impl_trait) => impl_trait,
        _ => return false,
    };

    impl_trait.bounds.iter().any(|bound| match bound {
        TypeParamBound::Trait(trait_bound) => trait_bound
            .path
            .segments
            .last()
            .map(|segment| segment.ident == "Future")
            .unwrap_or(false),
        _ => false,
    })
}
