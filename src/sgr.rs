mod base;
mod rgb;

pub use base::*;

use proc_macro2::{Span, TokenStream};
use quote::{quote, TokenStreamExt, ToTokens};
use syn::{parse::{Parse, ParseStream}, Token};
use rgb::Rgb;


macro_rules! sgr {
    ($($param:literal)?) => { concat!("\x1B[", $($param,)? "m") };
}


pub struct SgrReset;

impl Parse for SgrReset {
    fn parse(_: ParseStream) -> syn::Result<Self> { Ok(Self) }
}

impl ToTokens for SgrReset {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        syn::LitStr::new(sgr!(), Span::call_site()).to_tokens(tokens)
    }
}


#[inline]
const fn color_bg<const BG: bool>() -> u8 {
    if BG { 48 } else { 38 }
}


pub trait SgrData {
    fn base(&self) -> &SgrBase;
    fn fmt_opening(&self) -> String;
    fn fmt_closing(&self) -> String;

    fn tokens(&self) -> TokenStream {
        let mut tokens = TokenStream::new();

        let base = self.base();
        let fmt: String = format!(sgr!("{}"), self.fmt_opening());
        let end: String = match base.behavior.revert {
            Revert::One => format!(sgr!("{}"), self.fmt_closing()),
            Revert::All => String::from(sgr!()),
            Revert::None => String::new(),
        };

        let mut content = TokenStream::new();
        content.append_all(base.contents.clone());

        let expr = match base.behavior.output {
            Output::Concat => {
                assert!(base.template.is_none());
                quote!(concat!(concat!(#fmt, #content), #end))
            }
            Output::ConstFormat => {
                let template = base.template.as_ref().unwrap();
                let temp_fmt = format!("{fmt}{}{end}", template.value());
                let temp_lit = syn::LitStr::new(&temp_fmt, template.span());

                quote!(::const_format::formatcp!(#temp_lit, #content))
            }
            Output::Format => {
                let template = base.template.as_ref().unwrap();
                let temp_fmt = format!("{fmt}{}{end}", template.value());
                let temp_lit = syn::LitStr::new(&temp_fmt, template.span());

                quote!(format_args!(#temp_lit, #content))
            }
            Output::String => {
                let template = base.template.as_ref().unwrap();
                let temp_fmt = format!("{fmt}{}{end}", template.value());
                let temp_lit = syn::LitStr::new(&temp_fmt, template.span());

                quote!(format!(#temp_lit, #content))
            }
        };

        tokens.extend(expr);
        tokens
    }
}


pub struct SgrFormat {
    base: SgrBase,
    opening: String,
    closing: String,
}

impl SgrData for SgrFormat {
    fn base(&self) -> &SgrBase { &self.base }
    fn fmt_opening(&self) -> String { self.opening.clone() }
    fn fmt_closing(&self) -> String { self.closing.clone() }
}

impl ToTokens for SgrFormat {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.extend(self.tokens());
    }
}


pub struct SgrRgb<const BG: bool> {
    base: SgrBase,
    rgb: Rgb,
}

impl<const BG: bool> SgrData for SgrRgb<BG> {
    fn base(&self) -> &SgrBase { &self.base }

    fn fmt_opening(&self) -> String {
        let Rgb { a: _, r, g, b } = &self.rgb;
        format!("{};2;{};{};{}", color_bg::<BG>(), r, g, b)
    }

    fn fmt_closing(&self) -> String {
        format!("{}", color_bg::<BG>() + 1)
    }
}

impl<const BG: bool> Parse for SgrRgb<BG> {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let rgb: Rgb = input.parse()?;
        let _: Token![;] = input.parse()?;
        let base: SgrBase = input.parse()?;

        Ok(Self { base, rgb })
    }
}

impl<const BG: bool> ToTokens for SgrRgb<BG> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.extend(self.tokens())
    }
}


pub struct Sgr256<const BG: bool> {
    base: SgrBase,
    color: u8,
}

impl<const BG: bool> SgrData for Sgr256<BG> {
    fn base(&self) -> &SgrBase { &self.base }

    fn fmt_opening(&self) -> String {
        format!("{};5;{}", color_bg::<BG>(), self.color)
    }

    fn fmt_closing(&self) -> String {
        format!("{}", color_bg::<BG>() + 1)
    }
}

impl<const BG: bool> Parse for Sgr256<BG> {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let color: u8 = input.parse::<syn::LitInt>()?.base10_parse()?;
        let _: Token![;] = input.parse()?;
        let base: SgrBase = input.parse()?;

        Ok(Self { base, color })
    }
}

impl<const BG: bool> ToTokens for Sgr256<BG> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.extend(self.tokens())
    }
}
