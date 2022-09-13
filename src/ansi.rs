use proc_macro2::{Span, TokenStream};
use quote::{quote, TokenStreamExt, ToTokens};
use syn::{
    // braced,
    // bracketed,
    // parenthesized,
    parse::{Parse, ParseStream},
    Token,
};


#[derive(Clone, Copy, Debug)]
enum OutputMode {
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

impl OutputMode {
    const fn needs_template(&self) -> bool {
        match self {
            Self::Concat => false,
            Self::Format => true,
            Self::String => true,
        }
    }
}


#[derive(Clone, Copy, Debug)]
struct Behavior {
    mode: OutputMode,
    revert: bool,
}

impl Parse for Behavior {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mode = if input.parse::<Token![@]>().is_ok() {
            OutputMode::String
        } else if input.parse::<Token![%]>().is_ok() {
            OutputMode::Format
        } else {
            OutputMode::Concat
        };

        let no_revert = input.parse::<Token![!]>().is_ok();
        let revert = !no_revert;

        Ok(Self { mode, revert })
    }
}


#[derive(Clone)]
pub struct SgrFormat<const FMT: usize> {
    behavior: Behavior,
    template: Option<syn::LitStr>,
    contents: Vec<proc_macro2::TokenTree>,
}

impl<const FMT: usize> Parse for SgrFormat<FMT> {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let behavior: Behavior = input.parse()?;
        let template: Option<syn::LitStr>;
        let contents;

        if behavior.mode.needs_template() {
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

impl<const FMT: usize> ToTokens for SgrFormat<FMT> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let fmt = format!("\x1B[{FMT}m");
        let end = "\x1B[m";

        let mut content = TokenStream::new();

        // if let Some(template) = &self.template {
        //     content.append(template.token());
        //
        //     if !self.contents.is_empty() {
        //         content.extend(quote!(,));
        //         // content.append(syn::token::Comma);
        //     }
        // }

        // content.append_separated(&self.contents, quote!(,));
        content.append_all(&self.contents);

        let expr;

        match self.behavior.mode {
            OutputMode::Concat => {
                assert!(self.template.is_none());

                expr = if self.behavior.revert {
                    quote!(concat!(#fmt, #content, #end))
                } else {
                    quote!(concat!(#fmt, #content))
                };
            }
            OutputMode::Format => {
                let template = self.template.as_ref().unwrap();
                let temp_fmt = if self.behavior.revert {
                    format!("{}{}{}", fmt, template.value(), end)
                } else {
                    format!("{}{}", fmt, template.value())
                };

                let temp_lit = syn::LitStr::new(&temp_fmt, template.span());

                expr = quote!(format_args!(#temp_lit, #content));
            }
            OutputMode::String => {
                let template = self.template.as_ref().unwrap();
                let temp_fmt = if self.behavior.revert {
                    format!("{}{}{}", fmt, template.value(), end)
                } else {
                    format!("{}{}", fmt, template.value())
                };

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
