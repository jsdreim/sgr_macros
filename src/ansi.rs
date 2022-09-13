use proc_macro2::{Span, TokenStream};
use quote::{quote, TokenStreamExt, ToTokens};
use syn::{
    // braced,
    // bracketed,
    // parenthesized,
    parse::{Parse, ParseStream},
    Token,
};


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

        let output = if input.parse::<Token![@]>().is_ok() {
            sigil = true;
            Output::String
        } else if input.parse::<Token![%]>().is_ok() {
            sigil = true;
            Output::Format
        } else {
            Output::Concat
        };

        let revert = if input.parse::<Token![!]>().is_ok() {
            sigil = true;
            Revert::None
        } else if input.parse::<Token![*]>().is_ok() {
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


// pub struct SgrRgb {
//     color: u32,
//     behavior: Behavior,
// }
//
// impl Parse for SgrRgb {
//     fn parse(input: ParseStream) -> Result<Self> {
//         input.parse::<t![#]>()?;
//         let color_lit = input.parse::<syn::LitInt>()?;
//         let color: u32 = color_lit.to_string().parse().unwrap();
//
//         dbg!(color);
//
//         todo!()
//     }
// }
//
// impl ToTokens for SgrRgb {
//     fn to_tokens(&self, tokens: &mut TokenStream) {
//         todo!()
//     }
// }
//
//
// pub struct Sgr256 {
//     color: u8,
//     behavior: Behavior,
// }
//
// impl Parse for Sgr256 {
//     fn parse(input: ParseStream) -> Result<Self> {
//         todo!()
//     }
// }
//
// impl ToTokens for Sgr256 {
//     fn to_tokens(&self, tokens: &mut TokenStream) {
//         todo!()
//     }
// }
