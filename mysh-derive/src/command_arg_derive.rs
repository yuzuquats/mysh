use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse_macro_input, Data, DataStruct, DeriveInput, Fields, LitStr};

pub fn derive(input: TokenStream) -> TokenStream {
  let input = parse_macro_input!(input as DeriveInput);

  let fields = match &input.data {
    Data::Struct(DataStruct {
      fields: Fields::Named(fields),
      ..
    }) => &fields.named,
    _ => panic!("expected a struct with named fields"),
  };

  let mut fields_as_shell_args = vec![];
  for field in fields {
    let Some(ident) = &field.ident else {
      continue;
    };
    let arg = format!("--{ident}: {}", field.ty.to_token_stream());
    fields_as_shell_args.push(quote! {
      #arg
    });
  }
  // let args = fields_as_shell_args.join("");

  let name = input.ident;
  let expanded = quote! {
    impl mysh::CommandArg for #name {
      fn display_help() -> Vec<&'static str> {
        vec![#(#fields_as_shell_args),*]
      }
    }
  };

  proc_macro::TokenStream::from(expanded)
}
