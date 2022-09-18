use std::str::FromStr;
use syn::{parenthesized, parse::{Parse, ParseStream}, Token};


const fn accept_value(value: u32) -> Result<u32, &'static str> {
    //  TODO: Maybe make this configurable somehow. Is there any situation where
    //      an alpha channel is meaningful for terminal text?
    if value <= 0x_00_FF_FF_FF {
        Ok(value)
    } else {
        Err("RGB color value exceeds 24 bits")
    }
}


fn expand_rgb_rrggbb(rgb: &str) -> Option<String> {
    match rgb.as_bytes() {
        [r, g, b] => Some(format!(
            "{r}{r}{g}{g}{b}{b}",
            r = char::from(*r),
            g = char::from(*g),
            b = char::from(*b),
        )),
        _ => None,
    }
}


#[derive(Clone, Copy, Debug)]
pub struct Rgb {
    pub a: u8,
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl FromStr for Rgb {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let inner_hex = s.strip_prefix("#")
            .or_else(|| s.strip_prefix("0x"))
            .or_else(|| s.strip_prefix("0X"));

        if let Some(number) = inner_hex {
            if let Some(rrggbb) = expand_rgb_rrggbb(number) {
                if let Ok(n) = u32::from_str_radix(&rrggbb, 16) {
                    return Ok(n.into());
                }
            }

            match u32::from_str_radix(number, 16) {
                Ok(n) => Ok(n.into()),
                Err(..) => Err(()),
            }
        } else {
            // match s.parse::<u32>() {
            //     Ok(n) => Ok(n.into()),
            //     Err(..) => Err(()),
            // }

            Err(())
        }
    }
}

impl Parse for Rgb {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.peek(syn::token::Paren) {
            let inner;
            parenthesized!(inner in input);

            let r = inner.parse::<RgbNumber>()?.0;
            let _ = inner.parse::<Token![,]>()?;
            let g = inner.parse::<RgbNumber>()?.0;
            let _ = inner.parse::<Token![,]>()?;
            let b = inner.parse::<RgbNumber>()?.0;

            Ok(Self { a: 0, r, g, b })
        } else if let Ok(_) = input.fork().parse::<syn::LitStr>() {
            let literal = input.parse::<syn::LitStr>()?;

            match literal.value().parse::<Self>() {
                Ok(rgb) => Ok(rgb),
                Err(..) => todo!(),
            }
        } else {
            let literal = input.parse::<syn::LitInt>()?;

            // if let Ok(rgb) = literal.to_string().parse::<Self>() {
            //     return Ok(rgb);
            // }

            match accept_value(literal.base10_parse()?) {
                Ok(color) => Ok(color.into()),
                Err(text) => Err(syn::Error::new(literal.span(), text)),
            }
        }
    }
}

impl From<u32> for Rgb {
    fn from(word: u32) -> Self {
        let [a, r, g, b] = word.to_be_bytes();
        Self { a, r, g, b }
    }
}

impl From<Rgb> for u32 {
    fn from(rgb: Rgb) -> Self {
        let Rgb { a, r, g, b } = rgb;
        Self::from_be_bytes([a, r, g, b])
    }
}


struct RgbNumber(u8);

impl Parse for RgbNumber {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        const MAX: f64 = u8::MAX as f64;
        const MIN: f64 = u8::MIN as f64;

        if let Ok(_) = input.fork().parse::<syn::LitInt>() {
            let literal = input.parse::<syn::LitInt>()?;
            let numeral = literal.base10_parse::<u8>()?;

            Ok(Self(numeral))
        } else {
            let literal = input.parse::<syn::LitFloat>()?;
            let numeral = literal.base10_parse::<f64>()?;

            if numeral < 0.0 {
                Err(syn::Error::new(
                    literal.span(),
                    "RGB channel cannot be negative",
                ))
            } else if 1.0 < numeral {
                Err(syn::Error::new(
                    literal.span(),
                    "RGB channel cannot exceed 1.0",
                ))
            } else {
                let value = numeral * MAX;

                debug_assert!((MIN..=MAX).contains(&value));

                Ok(Self(value as u8))
            }
        }
    }
}
