use proc_macro2::{Span, TokenStream, TokenTree};
use quote::TokenStreamExt;
use syn::{parse::{Parse, ParseStream}, Token};
use super::SgrFormat;


type SigilOutputFormat = Token![%];
type SigilOutputString = Token![@];
type SigilRevertAll = Token![*];
type SigilRevertOff = Token![!];


#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Output {
    /// Resolves to a call to `concat!()`.
    ///
    /// Output: `&'static str`
    Concat,
    /// Resolves to a call to `format_args!()`.
    ///
    /// Output: [`Arguments`]
    Format,
    /// Resolves to a call to `format!()`.
    ///
    /// Output: [`String`]
    String,
}

impl Output {
    const fn needs_template(&self) -> bool {
        match self {
            Self::Concat => false,
            Self::Format => true,
            Self::String => true,
        }
    }
}


#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Revert {
    One,
    All,
    None,
}


#[derive(Clone, Copy, Debug)]
pub struct Behavior {
    pub output: Output,
    pub revert: Revert,
}

impl Parse for Behavior {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut sigil = false;

        let output = if input.parse::<SigilOutputString>().is_ok() {
            sigil = true;
            Output::String
        } else if input.parse::<SigilOutputFormat>().is_ok() {
            sigil = true;
            Output::Format
        } else {
            Output::Concat
        };

        let revert = if input.parse::<SigilRevertOff>().is_ok() {
            sigil = true;
            Revert::None
        } else if input.parse::<SigilRevertAll>().is_ok() {
            sigil = true;
            Revert::All
        } else {
            Revert::One
        };

        if sigil {
            //  Accept, but do not require, a comma after mode sigils.
            input.parse::<Token![,]>().ok();
        }

        Ok(Self { output, revert })
    }
}


#[derive(Clone)]
pub struct SgrBase {
    pub behavior: Behavior,
    pub template: Option<syn::LitStr>,
    pub contents: TokenStream,
}

impl SgrBase {
    pub const fn into_format(self, start: String, end: String) -> SgrFormat {
        SgrFormat { base: self, opening: start, closing: end }
    }
}

impl Parse for SgrBase {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let behavior: Behavior = input.parse()?;
        let template: Option<syn::LitStr>;
        let mut contents = TokenStream::new();

        if behavior.output.needs_template() {
            let expect_comma: bool;

            // template = Some(input.parse()?);
            // expect_comma = true;

            if let Ok(literal) = input.parse() {
                template = Some(literal);
                expect_comma = true;
            } else {
                template = Some(syn::LitStr::new("{}", Span::call_site()));
                expect_comma = false;
            }

            let can_continue = if expect_comma {
                input.parse::<Token![,]>().is_ok()
            } else {
                true
            };

            if can_continue {
                while let Ok(token) = input.parse::<TokenTree>() {
                    contents.append(token);
                }
            }

            // assert!(input.is_empty());
        } else if cfg!(feature = "const_format") {
            let fork = input.fork();
            let literal_next = fork.parse::<syn::LitStr>().is_ok();
            let then_comma = fork.peek(Token![,]);
            let expect_comma: bool;

            if literal_next && then_comma {
                template = Some(input.parse::<syn::LitStr>()?);
                expect_comma = true;
            } else {
                template = None;
                expect_comma = false;
            }

            let can_continue = if expect_comma {
                input.parse::<Token![,]>().is_ok()
            } else {
                true
            };

            if can_continue {
                while let Ok(token) = input.parse::<TokenTree>() {
                    contents.append(token);
                }
            }

            // assert!(input.is_empty());
        } else {
            template = None;

            while let Ok(token) = input.parse::<TokenTree>() {
                contents.append(token);
            }

            // assert!(input.is_empty());
        }

        Ok(Self { behavior, template, contents })
    }
}
