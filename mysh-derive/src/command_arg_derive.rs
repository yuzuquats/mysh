use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse_macro_input, Data, DataStruct, DeriveInput, Fields};

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
    let normalized_ty = field
      .ty
      .to_token_stream()
      .to_string()
      .replace(" >", ">")
      .replace(" < ", "<");
    let arg = format!("--{ident}: {normalized_ty}");
    fields_as_shell_args.push(quote! {
      #arg.to_string()
    });
  }
  // let args = fields_as_shell_args.join("");

  let name = input.ident;
  let expanded = quote! {
    impl mysh::CommandArg for #name {
      fn display_help() -> Vec<String> {
        vec![#(#fields_as_shell_args),*]
      }
    }
  };

  proc_macro::TokenStream::from(expanded)
}
