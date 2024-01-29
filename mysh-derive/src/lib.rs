#![feature(extend_one)]

mod command_arg_derive;
mod command_derive;

use proc_macro::TokenStream;

#[proc_macro_derive(CommandArg)]
pub fn derive_command_arg(input: TokenStream) -> TokenStream {
  command_arg_derive::derive(input)
}

#[proc_macro_attribute]
pub fn command(attr: TokenStream, func: TokenStream) -> TokenStream {
  command_derive::command(attr, func)
}
