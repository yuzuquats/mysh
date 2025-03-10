use std::ops::Deref;
use std::str::FromStr;

use proc_macro::TokenStream;
use proc_macro2::{Delimiter, Group};
use quote::{quote, ToTokens};
use syn::FnArg::{self, Typed};
use syn::{parse_macro_input, spanned::Spanned, Ident, LitStr};
use syn::{AngleBracketedGenericArguments, GenericArgument, ItemFn, PathArguments, Type};

pub fn command(attr: TokenStream, func: TokenStream) -> TokenStream {
  let (name, description, long_description) = {
    let mut name: Option<LitStr> = None;
    let mut description: Option<LitStr> = None;
    let mut long_description: Option<LitStr> = None;
    let attr_parser = syn::meta::parser(|meta| {
      let Some(ident) = meta.path.get_ident() else {
        // should err? not recognized
        return Ok(());
      };
      if ident == "name" {
        name = Some(meta.value()?.parse()?);
        return Ok(());
      }
      if ident == "description" {
        description = Some(meta.value()?.parse()?);
        return Ok(());
      };
      if ident == "long_description" {
        long_description = Some(meta.value()?.parse()?);
        return Ok(());
      }
      Ok(())
    });
    parse_macro_input!(attr with attr_parser);
    (
      name.expect("name required"),
      description.expect("description required"),
      match long_description {
        Some(s) => {
          let mut st = proc_macro2::TokenStream::from_str("Some").expect("");
          st.extend_one(
            Group::new(Delimiter::Parenthesis, s.into_token_stream()).into_token_stream(),
          );
          st
        }
        None => proc_macro2::TokenStream::from_str("None").expect(""),
      },
    )
  };

  let ast = match syn::parse::<ItemFn>(func.clone()) {
    Ok(ast) => ast,
    Err(err) => return input_and_compile_error(func, err),
  };
  let (info_ty, args_ty_turbo, args_ty, func_name, func_name_future, call_func) = {
    let inputs = ast.sig.inputs.iter().collect::<Vec<&FnArg>>();
    let Some(Typed(info)) = inputs.get(0) else {
      return input_and_compile_error(func, syn::Error::new(ast.sig.inputs.span(), "no info"));
    };
    let Some(Typed(args)) = inputs.get(1) else {
      return input_and_compile_error(func, syn::Error::new(ast.sig.inputs.span(), "no args"));
    };
    if let Type::Reference(_) = &info.ty.deref() {
      return input_and_compile_error(
        func,
        syn::Error::new(ast.sig.inputs.span(), "info is a ref"),
      );
    };
    if let Type::Reference(_) = &args.ty.deref() {
      return input_and_compile_error(
        func,
        syn::Error::new(ast.sig.inputs.span(), "info is a ref"),
      );
    };

    let func_name = &ast.sig.ident;
    let mut call_func = ast.clone();
    call_func.sig.ident = Ident::new("call", ast.sig.ident.span());

    let args_ty_turbo = type_to_turbo_fish(args.ty.deref());

    (
      info.ty.deref(),
      syn::parse_str::<Type>(&args_ty_turbo).expect("couldn't parse args"),
      args.ty.deref(),
      func_name,
      Ident::new(
        &format!("__{func_name}_future"),
        proc_macro2::Span::call_site(),
      ),
      call_func,
    )
  };

  let output = quote! {
    #[allow(non_camel_case_types, missing_docs)]
    pub struct #func_name;

    #[allow(non_camel_case_types, missing_docs)]
    pub struct #func_name_future {
      inner: std::pin::Pin<Box<dyn mysh::futures::Future<Output = mysh::Result<mysh::json::Value>>>>,
    }

    impl std::future::Future for #func_name_future {
      type Output = mysh::Result<mysh::json::Value>;

      fn poll(mut self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Self::Output> {
        self.inner.as_mut().poll(cx)
      }
    }

    impl #func_name {
      #call_func

      fn future(info: #info_ty, args: #args_ty) -> #func_name_future {
        use anyhow::Context;
        let inner = Box::pin(async move {
          let r = #func_name::call(info, args).await?;
          Ok(mysh::json::to_value(r).context("Return value not json-able")?)
        });
        #func_name_future { inner }
      }
    }

    impl std::ops::Deref for #func_name {
      type Target = (dyn Fn(#info_ty, #args_ty) -> #func_name_future);
      fn deref(&self) -> &Self::Target {
        &Self::future
      }
    }

    impl mysh::CommandMetadata<#info_ty> for #func_name {
      fn name(&self) -> &'static str {
        #name
      }
      fn description(&self) -> &'static str {
        #description
      }
      fn long_description(&self) -> Option<&'static str> {
        #long_description
      }
      fn call_with_argv(&self, info: #info_ty, argv: Vec<String>)
      -> mysh::Result<
          std::pin::Pin<Box<dyn mysh::futures::Future<Output = mysh::Result<mysh::json::Value>>>>
      > {
        use anyhow::Context;
        let args = mysh::parse_arguments(argv)?;
        Ok(Box::pin(async move {
          let r = #func_name::call(info, args).await?;
          Ok(mysh::json::to_value(r).context("Return value not json-able")?)
        }))
      }
      fn help(&self) -> Vec<String> {
        use mysh::CommandArg;
        #args_ty_turbo::display_help()
      }
    }
  };

  proc_macro::TokenStream::from(output)
}

/// Converts the error to a token stream and appends it to the original input.
///
/// Returning the original input in addition to the error is good for IDEs which can gracefully
/// recover and show more precise errors within the macro body.
///
/// See <https://github.com/rust-analyzer/rust-analyzer/issues/10468> for more info.
fn input_and_compile_error(mut item: TokenStream, err: syn::Error) -> TokenStream {
  let compile_err = TokenStream::from(err.to_compile_error());
  item.extend(compile_err);
  item
}

fn type_to_turbo_fish(ty: &Type) -> String {
  match ty {
    Type::Path(type_path) => {
      let segments = &type_path.path.segments;
      let mut result = String::new();

      for (i, segment) in segments.iter().enumerate() {
        if i > 0 {
          result.push_str("::");
        }
        result.push_str(&segment.ident.to_string());

        if let PathArguments::AngleBracketed(AngleBracketedGenericArguments { args, .. }) =
          &segment.arguments
        {
          if !args.is_empty() {
            result.push_str("::<");
            for (j, arg) in args.iter().enumerate() {
              if j > 0 {
                result.push_str(", ");
              }
              result.push_str(&match arg {
                GenericArgument::Type(arg_ty) => quote::quote!(#arg_ty).to_string(),
                GenericArgument::Lifetime(lt) => quote::quote!(#lt).to_string(),
                GenericArgument::Const(c) => quote::quote!(#c).to_string(),
                GenericArgument::Constraint(c) => quote::quote!(#c).to_string(),
                GenericArgument::AssocType(t) => quote::quote!(#t).to_string(),
                GenericArgument::AssocConst(c) => quote::quote!(#c).to_string(),
                _ => todo!(),
              });
            }
            result.push('>');
          }
        }
      }

      result
    }
    _ => panic!("Unsupported type"),
  }
}
