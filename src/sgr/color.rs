use proc_macro2::TokenTree;
use super::*;


const COLOR_SET: u8 = 38;
const COLOR_RESET: u8 = 39;

const OFFSET_BG: u8 = 10;
const OFFSET_BRIGHT: u8 = 60;


const fn color_bg(color: u8, bg: bool) -> u8 {
    if bg {
        color + OFFSET_BG
    } else {
        color
    }
}


const fn color_bright(color: u8, bright: bool) -> u8 {
    if bright {
        color + OFFSET_BRIGHT
    } else {
        color
    }
}


const fn color_set(bg: bool) -> u8 {
    color_bg(COLOR_SET, bg)
}


const fn color_reset(bg: bool) -> u8 {
    color_bg(COLOR_RESET, bg)
}


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
}

impl ColorBasic {
    pub const fn code(&self, bg: bool) -> u8 {
        let color = self.color as u8;
        let color = color_bright(color, self.bright);
        let color = color_bg(color, bg);
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

        Ok(Self { color, bright })
    }
}


#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct Indexed(u8);

impl Parse for Indexed {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let literal = input.parse::<syn::LitInt>()?;
        let index = literal.base10_parse::<u8>()?;

        Ok(Self(index))
    }
}


#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum ColorAny {
    Basic(ColorBasic),
    Indexed(Indexed),
    Rgb(Rgb),
}

impl ColorAny {
    fn params(&self, bg: bool) -> String {
        match self {
            Self::Basic(color) => format!("{}", color.code(bg)),
            Self::Indexed(idx) => format!("{};5;{}", color_set(bg), idx.0),
            Self::Rgb(rgb) => {
                let Rgb { a: _, r, g, b } = rgb;
                format!("{};2;{};{};{}", color_set(bg), r, g, b)
            }
        }
    }
}

impl Parse for ColorAny {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.fork().parse::<ColorBasic>().is_ok() {
            Ok(Self::Basic(input.parse()?))
        } else if input.fork().parse::<Indexed>().is_ok() {
            Ok(Self::Indexed(input.parse()?))
        } else {
            Ok(Self::Rgb(input.parse()?))
        }
    }
}


#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct ColorSetPair {
    pub fg: Option<ColorAny>,
    pub bg: Option<ColorAny>,
}

impl Parse for ColorSetPair {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let start = input.span();

        let fg = input.parse().ok();
        let bg = match input.parse::<Token![in]>() {
            Ok(_) => Some(input.parse()?),
            Err(_) => None,
        };

        if fg.is_none() && bg.is_none() {
            Err(syn::Error::new(start, "empty color"))
        } else {
            Ok(Self { fg, bg })
        }
    }
}

impl SgrCode for ColorSetPair {
    fn params(&self) -> Option<Cow<str>> {
        let mut colors = Vec::with_capacity(2);

        if let Some(color) = &self.fg {
            colors.push(color.params(false));
        }

        if let Some(color) = &self.bg {
            colors.push(color.params(true));
        }

        Some(Cow::Owned(colors.join(";")))
    }
}


#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ColorRevert {
    Set(ColorAny),
    Reset,
    NoReset,
}

impl ColorRevert {
    fn params(&self, bg: bool) -> Option<String> {
        match self {
            Self::Set(color) => Some(color.params(bg)),
            Self::Reset => Some(format!("{}", color_reset(bg))),
            Self::NoReset => None,
        }
    }
}

impl Parse for ColorRevert {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.parse::<Token![!]>().is_ok() {
            Ok(Self::NoReset)
        } else if let Ok(color) = input.parse() {
            Ok(Self::Set(color))
        } else {
            Ok(Self::Reset)
        }
    }
}


#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ColorRevertPair {
    Pair {
        fg: Option<ColorRevert>,
        bg: Option<ColorRevert>,
    },
    ResetAll,
    ResetNone,
}

impl ColorRevertPair {
    const fn reset_either() -> Self {
        Self::Pair {
            fg: Some(ColorRevert::Reset),
            bg: Some(ColorRevert::Reset),
        }
    }
}

impl Parse for ColorRevertPair {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.parse::<Token![*]>().is_ok() {
            Ok(Self::ResetAll)
        } else {
            let fg = input.parse().ok();
            let bg = match input.parse::<Token![in]>() {
                Ok(_) => Some(input.parse()?),
                Err(_) => None,
            };

            if fg.is_none() && bg.is_none() {
                Ok(Self::ResetNone)
            } else {
                Ok(Self::Pair { fg, bg })
            }
        }
    }
}

impl SgrCode for ColorRevertPair {
    fn params(&self) -> Option<Cow<str>> {
        match self {
            Self::Pair { fg: None, bg: None } => None,
            Self::Pair { fg: Some(fg), bg: None } => {
                fg.params(false).map(Cow::Owned)
            }
            Self::Pair { fg: None, bg: Some(bg) } => {
                bg.params(true).map(Cow::Owned)
            }
            Self::Pair { fg: Some(fg), bg: Some(bg) } => {
                let mut colors = Vec::with_capacity(2);

                colors.extend(fg.params(false));
                colors.extend(bg.params(true));

                Some(Cow::Owned(colors.join(";")))
            }
            Self::ResetAll => Some(Cow::Borrowed("39;49")),
            Self::ResetNone => None,
        }
    }
}


pub struct SgrColor {
    pub color_set: ColorSetPair,
    pub output: Output,
    pub template: Option<syn::LitStr>,
    pub contents: TokenStream,
    pub color_revert: ColorRevertPair,
}

impl Parse for SgrColor {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let color_set: ColorSetPair = input.parse()?;
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

        let color_revert: ColorRevertPair;

        if input.parse::<Token![;]>().is_ok() {
            color_revert = input.parse()?;
        } else {
            color_revert = ColorRevertPair::reset_either();
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

impl SgrData for SgrColor {
    type CodeOpening = ColorSetPair;
    type CodeClosing = ColorRevertPair;

    fn fmt_opening(&self) -> Self::CodeOpening {
        self.color_set
    }

    fn fmt_closing(&self) -> Self::CodeClosing {
        match self.color_revert {
            ColorRevertPair::Pair { fg, bg } => ColorRevertPair::Pair {
                fg: self.color_set.fg.and(fg),
                bg: self.color_set.bg.and(bg),
            },
            other => other,
        }
    }

    fn contents(&self) -> TokenStream {
        self.contents.clone()
    }

    fn template(&self) -> Option<syn::LitStr> {
        self.template.clone()
    }

    fn output(&self) -> Output {
        self.output
    }
}

impl ToTokens for SgrColor {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.tokens().to_tokens(tokens);
    }
}
