mod ansi;
mod old;

use proc_macro::TokenStream;
use quote::quote;
use ansi::SgrFormat;


macro_rules! def_sgr {
    ($(
    $(#[$attr:meta])*
    $name:ident = $code:expr;
    )*) => {$(
    $(#[$attr])*
    #[proc_macro]
    pub fn $name(stream: TokenStream) -> TokenStream {
        let fmt_def = syn::parse_macro_input!(stream as SgrFormat<$code>);
        quote!(#fmt_def).into()
    }
    )*};
}

def_sgr! {
    sgr_bold = 1;
    sgr_faint = 2;
    sgr_italic = 3;
    sgr_uline = 4;
    sgr_blink = 5;
    // sgr_blink2 = 6;
    sgr_invert = 7;
    sgr_strike = 9;

    // sgr_conceal = 8;
    // sgr_reveal = 28;
}

def_sgr! {
    black = 30;
    black_bright = 90;

    red = 31;
    red_bright = 91;

    green = 32;
    green_bright = 92;

    yellow = 33;
    yellow_bright = 93;

    blue = 34;
    blue_bright = 94;

    magenta = 35;
    magenta_bright = 95;

    cyan = 36;
    cyan_bright = 96;

    white = 37;
    white_bright = 97;
}

def_sgr! {
    bg_black = 40;
    bg_black_bright = 100;

    bg_red = 41;
    bg_red_bright = 101;

    bg_green = 42;
    bg_green_bright = 102;

    bg_yellow = 43;
    bg_yellow_bright = 103;

    bg_blue = 44;
    bg_blue_bright = 104;

    bg_magenta = 45;
    bg_magenta_bright = 105;

    bg_cyan = 46;
    bg_cyan_bright = 106;

    bg_white = 47;
    bg_white_bright = 107;
}


/*macro_rules! def_color_pair {
    ($($name:ident, $name2:ident = $code:literal;)*) => {$(
    // #[proc_macro]
    // pub fn $name(stream: TokenStream) -> TokenStream {
    //     let color = syn::parse_macro_input!(stream as SgrFormat<$code>);
    //     quote!(#color).into()
    // }
    // #[proc_macro]
    // pub fn $name2(stream: TokenStream) -> TokenStream {
    //     let color = syn::parse_macro_input!(stream as SgrFormat<{$code + 60}>);
    //     quote!(#color).into()
    // }
    def_sgr! {
        $name = $code;
        $name2 = {$code + 60};
    }
    )*};
}

def_color_pair! {
    black, black_bright = 30;
}*/
