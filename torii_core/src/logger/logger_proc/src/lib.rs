use proc_macro::TokenStream;
use quote::{quote};
use syn::{parse_macro_input, AttributeArgs, ItemFn, NestedMeta, Meta, Lit};

#[proc_macro_attribute]
pub fn engine_logger(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr as AttributeArgs);
    let mut enable_json = false;
    let mut enable_timing = false;
    let mut file_sink = None;

    for arg in args {
        match arg {
            NestedMeta::Meta(Meta::Path(path)) if path.is_ident("json") => {
                enable_json = true;
            }
            NestedMeta::Meta(Meta::Path(path)) if path.is_ident("timing") => {
                enable_timing = true;
            }
            NestedMeta::Meta(Meta::NameValue(nv)) if nv.path.is_ident("file_sink") => {
                if let Lit::Str(lit) = nv.lit {
                    file_sink = Some(lit.value());
                }
            }
            _ => {}
        }
    }

    let input_fn = parse_macro_input!(item as ItemFn);
    let sig = &input_fn.sig;
    let body = &input_fn.block;

    let json_layer = if enable_json {
        quote! { let stdout_layer = tracing_subscriber::fmt::layer().json(); }
    } else {
        quote! { let stdout_layer = tracing_subscriber::fmt::layer(); }
    };

    let file_layer = if let Some(path) = file_sink {
        let layer = if enable_json {
            quote! {
                let file = std::fs::File::create(#path).unwrap();
                let file_layer = tracing_subscriber::fmt::layer().json().with_writer(file);
            }
        } else {
            quote! {
                let file = std::fs::File::create(#path).unwrap();
                let file_layer = tracing_subscriber::fmt::layer().with_writer(file);
            }
        };
        Some(layer)
    } else {
        None
    };

    let timing_layer = if enable_timing {
        quote! {
            let timing_layer = logger_core::timing_layer::build();
        }
    } else {
        quote! {}
    };

    let mut registry = quote! {
        tracing_subscriber::registry()
            .with(filter)
            .with(stdout_layer)
    };

    if file_sink.is_some() {
        registry = quote! { #registry.with(file_layer) };
    }

    if enable_timing {
        registry = quote! { #registry.with(timing_layer) };
    }

    let expanded = quote! {
        #sig {
            let filter = tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));

            #json_layer
            #file_layer
            #timing_layer

            #registry.init();

            std::panic::set_hook(Box::new(|info| {
                tracing::error!("PANIC: {}", info);
            }));

            tracing::info!("Logger initialized via proc macro.");

            #body
        }
    };

    expanded.into()
}
