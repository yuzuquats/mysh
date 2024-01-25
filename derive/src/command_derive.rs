use std::ops::Deref;

use proc_macro::TokenStream;
use quote::quote;
use syn::FnArg::Typed;
use syn::Type;
use syn::{parse_macro_input, spanned::Spanned, FnArg, Ident, LitStr};

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
      name.expect(""),
      description.expect(""),
      long_description.expect(""),
    )
  };

  let ast = match syn::parse::<syn::ItemFn>(func.clone()) {
    Ok(ast) => ast,
    Err(err) => return input_and_compile_error(func, err),
  };
  let (info_ty, args_ty, info_ty_deref, args_ty_deref, func_name, func_name_future, call_func) = {
    let inputs = ast.sig.inputs.iter().collect::<Vec<&FnArg>>();
    let Some(Typed(info)) = inputs.get(0) else {
      return input_and_compile_error(func, syn::Error::new(ast.sig.inputs.span(), "no info"));
    };
    let Some(Typed(args)) = inputs.get(1) else {
      return input_and_compile_error(func, syn::Error::new(ast.sig.inputs.span(), "no args"));
    };
    let Type::Reference(info_ty) = &info.ty.deref() else {
      return input_and_compile_error(
        func,
        syn::Error::new(ast.sig.inputs.span(), "info is not ref"),
      );
    };
    let Type::Reference(args_ty) = &args.ty.deref() else {
      return input_and_compile_error(
        func,
        syn::Error::new(ast.sig.inputs.span(), "info is not ref"),
      );
    };

    let func_name = &ast.sig.ident;
    let mut call_func = ast.clone();
    call_func.sig.ident = Ident::new("call", ast.sig.ident.span());

    (
      info_ty,
      args_ty,
      &info_ty.elem,
      &args_ty.elem,
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
    pub struct #func_name_future {
      info: #info_ty_deref,
      args: #args_ty_deref,
    }

    impl std::future::Future for #func_name_future {
      type Output = Result<(), mysh::error::Error>;

      fn poll(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Self::Output> {
        let r = futures::executor::block_on(#func_name::call(&self.info, &self.args));
        std::task::Poll::Ready(r)
      }
    }

    #[allow(non_camel_case_types, missing_docs)]
    pub struct #func_name;

    impl #func_name {
      #call_func

      fn future(info: #info_ty, args: #args_ty) -> #func_name_future {
        #func_name_future { info: info.clone(), args: args.clone() }
      }
    }

    impl std::ops::Deref for #func_name {
      type Target = (dyn for<'a, 'b> Fn(&'a #info_ty_deref, &'b #args_ty_deref) -> #func_name_future);
      fn deref(&self) -> &Self::Target {
        &Self::future
      }
    }

    impl mysh::command::CommandMetadata<#info_ty_deref> for #func_name {
      fn name(&self) -> &'static str {
        #name
      }
      fn description(&self) -> &'static str {
        #description
      }
      fn long_description(&self) -> &'static str {
        #long_description
      }
      fn call_with_argv(&self, info: #info_ty, argv: Vec<String>) -> Result<(), mysh::error::Error> {
        let args = mysh::command_arg::parse_command_arg(argv)?;
        let r = futures::executor::block_on(#func_name::call(info, &args));
        Ok(())
      }
      fn help(&self) -> &'static str {
        Args::display_help()
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
