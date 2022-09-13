use proc_macro2::Span;
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
    /// Output: `&str` (literal)
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
    pub contents: Vec<proc_macro2::TokenTree>,
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
        let contents;

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

            let mut params = Vec::new();

            if can_continue {
                while let Ok(expr) = input.parse() {
                    params.push(expr);
                }
            }

            contents = params;
        } else {
            template = None;
            // contents = vec![input.parse()?];

            let mut params = Vec::new();

            while let Ok(expr) = input.parse() {
                params.push(expr);

                // if input.parse::<t![,]>().is_err() {
                //     break;
                // }
            }

            contents = params;
        }

        Ok(Self { behavior, template, contents })
    }
}
