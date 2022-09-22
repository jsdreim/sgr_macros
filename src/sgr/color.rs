use proc_macro2::TokenTree;
use super::*;


#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
#[repr(u8)]
pub enum ColorNamed {
    Black = 30,
    Red = 31,
    Green = 32,
    Yellow = 33,
    Blue = 34,
    Magenta = 35,
    Cyan = 36,
    White = 37,
}


#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct ColorBasic {
    color: ColorNamed,
    bright: bool,
    background: bool,
}

impl ColorBasic {
    pub const fn code(&self) -> u8 {
        let mut color = self.color as u8;

        if self.bright { color += 60; }
        if self.background { color += 10; }

        color
    }
}

impl Parse for ColorBasic {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let literal = input.parse::<syn::LitStr>()?;
        let value = literal.value().to_lowercase();

        let bright: bool;
        let color_name: &str;

        if let Some(base) = value.strip_prefix("bright ") {
            bright = true;
            color_name = base;
        } else {
            bright = false;
            color_name = value.as_str();
        }

        let color = match color_name {
            "black" => ColorNamed::Black,
            "red" => ColorNamed::Red,
            "green" => ColorNamed::Green,
            "yellow" => ColorNamed::Yellow,
            "blue" => ColorNamed::Blue,
            "magenta" => ColorNamed::Magenta,
            "cyan" => ColorNamed::Cyan,
            "white" => ColorNamed::White,
            _ => return Err(syn::Error::new(literal.span(), "invalid color")),
        };

        Ok(Self { color, bright, background: false })
    }
}


struct Indexed(u8);

impl Parse for Indexed {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let literal = input.fork().parse::<syn::LitInt>()?;
        let index = literal.base10_parse::<u8>()?;

        Ok(Self(index))
    }
}


#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum ColorAny {
    Basic(ColorBasic),
    Indexed(u8),
    Rgb(Rgb),
}

impl Parse for ColorAny {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if let Ok(color) = input.parse::<ColorBasic>() {
            Ok(Self::Basic(color))
        } else if let Ok(Indexed(idx)) = input.parse() {
            Ok(Self::Indexed(idx))
        } else {
            Ok(Self::Rgb(input.parse()?))
        }
    }
}


#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ColorRevert {
    Set(ColorAny),
    Reset,
    ResetAll,
    ResetNone,
}

impl Parse for ColorRevert {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.fork().parse::<ColorAny>().is_ok() {
            Ok(Self::Set(input.parse()?))
        } else if input.parse::<Token![!]>().is_ok() {
            Ok(Self::ResetNone)
        } else if input.parse::<Token![*]>().is_ok() {
            Ok(Self::ResetAll)
        } else {
            Ok(Self::Reset)
        }
    }
}


pub struct SgrColor {
    pub color_set: ColorAny,
    pub output: Output,
    pub template: Option<syn::LitStr>,
    pub contents: TokenStream,
    pub color_revert: ColorRevert,
}

impl Parse for SgrColor {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let color_set: ColorAny = input.parse()?;
        input.parse::<Token![;]>()?;
        let output: Output = input.parse()?;

        if output.has_sigil() {
            //  Accept, but do not require, a comma after an output sigil.
            input.parse::<Token![,]>().ok();
        }

        let template: Option<syn::LitStr>;
        let get_more: bool;

        if output.needs_template() {
            //  Output mode requires a template literal.
            if let Ok(t) = input.parse() {
                template = Some(t);
                get_more = input.parse::<Token![,]>().is_ok();
            } else {
                template = Some(syn::LitStr::new("{}", Span::call_site()));
                get_more = true;
            }
        } else {
            //  Output mode does not accept a template literal.
            template = None;
            get_more = true;
        }

        let mut contents = TokenStream::new();

        if get_more {
            while let Ok(token) = input.parse::<TokenTree>() {
                contents.append(token);

                if input.peek(Token![;]) {
                    break;
                }
            }
        }

        let color_revert: ColorRevert;

        if input.parse::<Token![;]>().is_ok() {
            color_revert = input.parse()?;
        } else {
            color_revert = ColorRevert::Reset;
        }

        Ok(Self {
            color_set,
            output,
            template,
            contents,
            color_revert,
        })
    }
}
