use proc_macro2::{Span, TokenStream};
use quote::{quote, TokenStreamExt, ToTokens};
use syn::{
    // braced,
    // bracketed,
    // parenthesized,
    parse::{Parse, ParseStream},
    Token,
};


type SigilOutputFormat = Token![%];
type SigilOutputString = Token![@];
type SigilRevertAll = Token![*];
type SigilRevertOff = Token![!];


#[derive(Clone, Copy, Debug, PartialEq)]
enum Output {
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
enum Revert {
    One,
    All,
    None,
}


#[derive(Clone, Copy, Debug)]
struct Behavior {
    output: Output,
    revert: Revert,
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
pub struct SgrFormat {
    behavior: Behavior,
    template: Option<syn::LitStr>,
    contents: Vec<proc_macro2::TokenTree>,

    pub start: String,
    pub end: String,
}

impl Parse for SgrFormat {
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

        Ok(Self {
            behavior,
            template,
            contents,
            start: String::new(),
            end: String::new(),
        })
    }
}

impl ToTokens for SgrFormat {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let fmt: String = format!("\x1B[{}m", self.start);
        let revert: String = match self.behavior.revert {
            Revert::One => format!("\x1B[{}m", self.end),
            Revert::All => String::from("\x1B[m"),
            Revert::None => String::new(),
        };

        let mut content = TokenStream::new();
        content.append_all(&self.contents);

        let expr;

        match self.behavior.output {
            Output::Concat => {
                assert!(self.template.is_none());
                expr = quote!(concat!(concat!(#fmt, #content), #revert));
            }
            Output::Format => {
                let template = self.template.as_ref().unwrap();
                let temp_fmt = format!("{}{}{}", fmt, template.value(), revert);
                let temp_lit = syn::LitStr::new(&temp_fmt, template.span());

                expr = quote!(format_args!(#temp_lit, #content));
            }
            Output::String => {
                let template = self.template.as_ref().unwrap();
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

        let (r, g, b) = if input.parse::<Token![#]>().is_ok() {
            let r = input.parse::<syn::LitInt>()?.base10_parse()?;
            let _ = input.parse::<Token![,]>()?;
            let g = input.parse::<syn::LitInt>()?.base10_parse()?;
            let _ = input.parse::<Token![,]>()?;
            let b = input.parse::<syn::LitInt>()?.base10_parse()?;

            (r, g, b)
        } else {
            let color_lit: syn::LitInt = input.parse()?;
            let color: u32 = color_lit.base10_parse()?;
            let [_, r, g, b] = color.to_be_bytes();

            (r, g, b)
        };

        input.parse::<Token![;]>()?;
        let mut format: SgrFormat = input.parse()?;
        format.start = format!("{};2;{};{};{}", code, r, g, b);
        format.end = format!("{}", code + 1);

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

        let color_lit: syn::LitInt = input.parse()?;
        let color: u8 = color_lit.base10_parse()?;

        input.parse::<Token![;]>()?;
        let mut format: SgrFormat = input.parse()?;
        format.start = format!("{};5;{}", code, color);
        format.end = format!("{}", code + 1);

        Ok(Self { format })
    }
}

impl<const BG: bool> ToTokens for Sgr256<BG> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.format.to_tokens(tokens)
    }
}
