mod ansi;
mod old;

use proc_macro::TokenStream;
use quote::quote;
use ansi::{SgrFormat, SgrRgb, Sgr256};


#[proc_macro]
pub fn sgr_bg_rgb(stream: TokenStream) -> TokenStream {
    let fmt_def = syn::parse_macro_input!(stream as SgrRgb<true>);
    quote!(#fmt_def).into()
}


#[proc_macro]
pub fn sgr_fg_rgb(stream: TokenStream) -> TokenStream {
    let fmt_def = syn::parse_macro_input!(stream as SgrRgb<false>);
    quote!(#fmt_def).into()
}


#[proc_macro]
pub fn sgr_bg_256(stream: TokenStream) -> TokenStream {
    let fmt_def = syn::parse_macro_input!(stream as Sgr256<true>);
    quote!(#fmt_def).into()
}


#[proc_macro]
pub fn sgr_fg_256(stream: TokenStream) -> TokenStream {
    let fmt_def = syn::parse_macro_input!(stream as Sgr256<false>);
    quote!(#fmt_def).into()
}


macro_rules! def_sgr {
    ($(
    $(#[$attr:meta])*
    $name:ident = $start:literal $(, $end:literal)?;
    )*) => {
        $($(#[$attr])*
        #[proc_macro]
        pub fn $name(stream: TokenStream) -> TokenStream {
            let mut fmt_def = syn::parse_macro_input!(stream as SgrFormat);
            fmt_def.start = format!("{}", $start);
            $(fmt_def.end = format!("{}", $end);)?

            quote!(#fmt_def).into()
        })*
    };
}

def_sgr! {
    sgr_bold = 1, 22;
    sgr_faint = 2, 22;
    sgr_italic = 3, 23;
    sgr_uline = 4, 24;
    sgr_blink = 5, 25;
    sgr_blink2 = 6, 25;
    sgr_invert = 7, 27;
    sgr_conceal = 8, 28;
    sgr_strike = 9, 29;
}

def_sgr! {
    black = 30, 39;
    black_bright = 90, 99;

    red = 31, 39;
    red_bright = 91, 99;

    green = 32, 39;
    green_bright = 92, 99;

    yellow = 33, 39;
    yellow_bright = 93, 99;

    blue = 34, 39;
    blue_bright = 94, 99;

    magenta = 35, 39;
    magenta_bright = 95, 99;

    cyan = 36, 39;
    cyan_bright = 96, 99;

    white = 37, 39;
    white_bright = 97, 99;
}

def_sgr! {
    bg_black = 40, 49;
    bg_black_bright = 100, 109;

    bg_red = 41, 49;
    bg_red_bright = 101, 109;

    bg_green = 42, 49;
    bg_green_bright = 102, 109;

    bg_yellow = 43, 49;
    bg_yellow_bright = 103, 109;

    bg_blue = 44, 49;
    bg_blue_bright = 104, 109;

    bg_magenta = 45, 49;
    bg_magenta_bright = 105, 109;

    bg_cyan = 46, 49;
    bg_cyan_bright = 106, 109;

    bg_white = 47, 49;
    bg_white_bright = 107, 109;
}
