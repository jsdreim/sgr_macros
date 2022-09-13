mod base;
mod rgb;

pub use base::*;

use proc_macro2::TokenStream;
use quote::{quote, TokenStreamExt, ToTokens};
use syn::{parse::{Parse, ParseStream}, Token};
use rgb::Rgb;


pub struct SgrFormat {
    base: SgrBase,
    start: String,
    end: String,
}

impl ToTokens for SgrFormat {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let fmt: String = format!("\x1B[{}m", self.start);
        let revert: String = match self.base.behavior.revert {
            Revert::One => format!("\x1B[{}m", self.end),
            Revert::All => String::from("\x1B[m"),
            Revert::None => String::new(),
        };

        let mut content = TokenStream::new();
        content.append_all(&self.base.contents);

        let expr;

        match self.base.behavior.output {
            Output::Concat => {
                assert!(self.base.template.is_none());
                expr = quote!(concat!(concat!(#fmt, #content), #revert));
            }
            Output::Format => {
                let template = self.base.template.as_ref().unwrap();
                let temp_fmt = format!("{}{}{}", fmt, template.value(), revert);
                let temp_lit = syn::LitStr::new(&temp_fmt, template.span());

                expr = quote!(format_args!(#temp_lit, #content));
            }
            Output::String => {
                let template = self.base.template.as_ref().unwrap();
                let temp_fmt = format!("{}{}{}", fmt, template.value(), revert);
                let temp_lit = syn::LitStr::new(&temp_fmt, template.span());

                expr = quote!(format!(#temp_lit, #content));
            }
        }

        tokens.extend(expr);
    }
}


pub struct SgrRgb<const BG: bool> {
    format: SgrFormat,
}

impl<const BG: bool> Parse for SgrRgb<BG> {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let code: u8 = if BG { 48 } else { 38 };

        let Rgb { r, g, b } = input.parse()?;
        input.parse::<Token![;]>()?;
        let base: SgrBase = input.parse()?;
        let start = format!("{};2;{};{};{}", code, r, g, b);
        let end = format!("{}", code + 1);

        let format = base.into_format(start, end);

        Ok(Self { format })
    }
}

impl<const BG: bool> ToTokens for SgrRgb<BG> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.format.to_tokens(tokens)
    }
}


pub struct Sgr256<const BG: bool> {
    format: SgrFormat,
}

impl<const BG: bool> Parse for Sgr256<BG> {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let code: u8 = if BG { 48 } else { 38 };
        let color: u8 = input.parse::<syn::LitInt>()?
            .base10_parse()?;

        input.parse::<Token![;]>()?;
        let base: SgrBase = input.parse()?;
        let start = format!("{};5;{}", code, color);
        let end = format!("{}", code + 1);

        let format = base.into_format(start, end);

        Ok(Self { format })
    }
}

impl<const BG: bool> ToTokens for Sgr256<BG> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.format.to_tokens(tokens)
    }
}
