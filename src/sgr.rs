macro_rules! sgr {
    ($($param:literal)?) => { concat!("\x1B[", $($param,)? "m") };
}

mod base;
mod color;
mod rgb;
mod traits;

pub use base::*;
pub use traits::*;

use std::borrow::Cow;
use proc_macro2::{Span, TokenStream};
use quote::{TokenStreamExt, ToTokens};
use syn::{parse::{Parse, ParseStream}, Token};
use rgb::Rgb;


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


pub struct StringRevert {
    pub revert: Revert,
    pub string: String,
}

impl SgrCode for StringRevert {
    fn params(&self) -> Option<Cow<str>> {
        match self.revert {
            Revert::One => Some(Cow::Borrowed(&self.string)),
            Revert::All => Some(Cow::Borrowed("")),
            Revert::None => None,
        }
    }
}


pub struct SgrFormat {
    base: SgrBase,
    opening: String,
    closing: String,
}

impl SgrData for SgrFormat {
    type CodeOpening = String;
    type CodeClosing = StringRevert;

    fn fmt_opening(&self) -> Self::CodeOpening {
        self.opening.clone()
    }

    fn fmt_closing(&self) -> Self::CodeClosing {
        StringRevert {
            revert: self.base.behavior.revert,
            string: self.closing.clone(),
        }
    }

    fn contents(&self) -> TokenStream {
        self.base.contents.clone()
    }

    fn template(&self) -> Option<syn::LitStr> {
        self.base.template.clone()
    }

    fn output(&self) -> Output {
        self.base.behavior.output
    }
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
    type CodeOpening = String;
    type CodeClosing = StringRevert;

    fn fmt_opening(&self) -> Self::CodeOpening {
        let Rgb { a: _, r, g, b } = &self.rgb;
        format!("{};2;{};{};{}", color_bg::<BG>(), r, g, b)
    }

    fn fmt_closing(&self) -> Self::CodeClosing {
        StringRevert {
            revert: self.base.behavior.revert,
            string: format!("{}", color_bg::<BG>() + 1),
        }
    }

    fn contents(&self) -> TokenStream {
        self.base.contents.clone()
    }

    fn template(&self) -> Option<syn::LitStr> {
        self.base.template.clone()
    }

    fn output(&self) -> Output {
        self.base.behavior.output
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
    type CodeOpening = String;
    type CodeClosing = StringRevert;

    fn fmt_opening(&self) -> Self::CodeOpening {
        format!("{};5;{}", color_bg::<BG>(), self.color)
    }

    fn fmt_closing(&self) -> Self::CodeClosing {
        StringRevert {
            revert: self.base.behavior.revert,
            string: format!("{}", color_bg::<BG>() + 1),
        }
    }

    fn contents(&self) -> TokenStream {
        self.base.contents.clone()
    }

    fn template(&self) -> Option<syn::LitStr> {
        self.base.template.clone()
    }

    fn output(&self) -> Output {
        self.base.behavior.output
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
