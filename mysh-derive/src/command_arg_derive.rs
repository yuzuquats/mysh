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
    fields_as_shell_args.push(format!("--{ident}:{}", field.ty.to_token_stream()))
  }
  let args = fields_as_shell_args.join("\n");

  let name = input.ident;
  let expanded = quote! {
    impl mysh::command_arg::CommandArg for #name {
      fn display_help() -> &'static str {
          #args
      }
    }
  };

  proc_macro::TokenStream::from(expanded)
}
