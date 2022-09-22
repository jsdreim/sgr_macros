use proc_macro2::{Span, TokenStream, TokenTree};
use quote::TokenStreamExt;
use syn::{parse::{Parse, ParseStream}, Token};
use super::SgrFormat;


type SigilOutputConstFormat = Token![#];
type SigilOutputFormat = Token![%];
type SigilOutputString = Token![@];
type SigilRevertAll = Token![*];
type SigilRevertOff = Token![!];


#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Output {
    /// Resolves to a call to [`concat!`].
    ///
    /// Output: `&'static str` (literal)
    Concat,
    /// Resolves to a call to [`::const_format::formatcp!`].
    ///
    /// Output: `&'static str`
    ConstFormat,
    /// Resolves to a call to [`format_args!`].
    ///
    /// Output: [`Arguments`]
    Format,
    /// Resolves to a call to [`format!`].
    ///
    /// Output: [`String`]
    String,
}

impl Output {
    /*pub const fn accepts_template(&self) -> bool {
        match self {
            Self::Concat => false,
            Self::ConstFormat => true,
            Self::Format => true,
            Self::String => true,
        }
    }*/

    pub const fn needs_template(&self) -> bool {
        match self {
            Self::Concat => false,
            Self::ConstFormat => true,
            Self::Format => true,
            Self::String => true,
        }
    }

    pub fn has_sigil(&self) -> bool { *self != Self::Concat }
}

impl Parse for Output {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.parse::<SigilOutputString>().is_ok() {
            Ok(Self::String)
        } else if input.parse::<SigilOutputFormat>().is_ok() {
            Ok(Self::Format)
        } else if let Ok(mode_sigil) = input.parse::<SigilOutputConstFormat>() {
            if !cfg!(feature = "const") {
                return Err(syn::Error::new(
                    mode_sigil.span,
                    "mode sigil requires the \"const\" feature",
                ));
            }

            Ok(Self::ConstFormat)
        } else {
            Ok(Self::Concat)
        }
    }
}


#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Revert {
    One,
    All,
    None,
}

impl Revert {
    pub fn has_sigil(&self) -> bool { *self != Self::One }
}

impl Parse for Revert {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.parse::<SigilRevertOff>().is_ok() {
            Ok(Self::None)
        } else if input.parse::<SigilRevertAll>().is_ok() {
            Ok(Self::All)
        } else {
            Ok(Self::One)
        }
    }
}


#[derive(Clone, Copy, Debug)]
pub struct Behavior {
    pub output: Output,
    pub revert: Revert,
}

impl Parse for Behavior {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let output: Output = input.parse()?;
        let revert: Revert = input.parse()?;

        if output.has_sigil() || revert.has_sigil() {
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
        let get_more: bool;

        if behavior.output.needs_template() {
            //  Output mode requires a template literal.
            if let Ok(t) = input.parse() {
                template = Some(t);
                get_more = input.parse::<Token![,]>().is_ok();
            } else {
                template = Some(syn::LitStr::new("{}", Span::call_site()));
                get_more = true;
            }/*
        } else if behavior.output.accepts_template() {
            //  Output mode does not require a template literal, can still
            //      accept one.
            let fork = input.fork();
            let template_next = fork.parse::<syn::LitStr>().is_ok();
            let then_comma = fork.peek(Token![,]);

            if template_next && then_comma {
                template = Some(input.parse::<syn::LitStr>()?);
                get_more = input.parse::<Token![,]>().is_ok();
            } else {
                template = None;
                get_more = true;
            }*/
        } else {
            //  Output mode does not accept a template literal.
            template = None;
            get_more = true;
        }

        let mut contents = TokenStream::new();

        if get_more {
            while let Ok(token) = input.parse::<TokenTree>() {
                contents.append(token);
            }
        }

        Ok(Self { behavior, template, contents })
    }
}
