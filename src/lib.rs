//! This crate provides macros for ergonomically wrapping text in ANSI control
//!     sequences with SGR ("Select Graphic Rendition") parameters. These
//!     parameters are used to color text, as well as apply styling such as
//!     italics and underlining. Extensive information on the specific sequences
//!     is available on [Wikipedia].
//!
//! [Wikipedia]: https://en.wikipedia.org/wiki/ANSI_escape_code#SGR
//!
//! There are three "modes" of output to every macro in this crate: Literal,
//!     Format, and String. Additionally, there are three "modes" of reversion:
//!     Single, Total, and None.
//!
//! ## Output Modes
//!
//! The simplest output mode is Literal Mode. A string literal must be supplied,
//!     and all formatting is applied directly, at compile-time. The output of
//!     a Literal Mode macro invocation is a `&str` literal suitable for input
//!     to `concat!()`.
//! ```
//! use sgr_macros::*;
//!
//! let green: &'static str = green!("Green Text");
//! assert_eq!(green, "\x1B[32mGreen Text\x1B[39m");
//!
//! let bold: &'static str = sgr_bold!(green!("Bold Green Text"));
//! assert_eq!(bold, "\x1B[1m\x1B[32mBold Green Text\x1B[39m\x1B[22m");
//!
//! let concat: &'static str = concat!(sgr_bold!("Bold Text"), ", Normal Text");
//! assert_eq!(concat, "\x1B[1mBold Text\x1B[22m, Normal Text");
//! ```
//!
//! The second mode is Format Mode. Invocations in this mode resolve to format
//!     [`Arguments`], suitable as input parameters for formatting macros such
//!     as `format!()`, `println!()`, and `write!()`. This mode is enabled by
//!     placing a `%` sigil at the beginning of the call. After the sigil, a
//!     template literal may be provided.
//!
//! [`Arguments`]: std::fmt::Arguments
//! ```
//! use sgr_macros::*;
//!
//! fn lights(number: &str) -> String {
//!     format!("There are {} lights.", sgr_uline!(% number))
//! }
//!
//! let text: String = lights("five");
//! assert_eq!(text, "There are \x1B[4mfive\x1B[24m lights.");
//!
//! fn lights_alt(number: &str) -> String {
//!     format!("There are {}.", sgr_italic!(%"{} lights", number))
//! }
//!
//! let text: String = lights_alt("four");
//! assert_eq!(text, "There are \x1B[3mfour lights\x1B[23m.");
//! ```
//!
//! The third mode is String Mode. An invocation in this mode will resolve to a
//!     call to `format!()`, returning a fully-formed heap-allocated [`String`].
//!     This mode is enabled with a `@` sigil at the beginning of the call, and
//!     it may also be provided a template literal.
//! ```
//! use sgr_macros::*;
//!
//! fn status(ok: bool, msg: &str) -> String {
//!     if ok {
//!         blue_bright!(@ msg)
//!     } else {
//!         red!(@"ERROR: {}", msg)
//!     }
//! }
//!
//! let text: String = status(true, "Success.");
//! assert_eq!(text, "\x1B[94mSuccess.\x1B[99m");
//!
//! let text: String = status(false, "System is on fire.");
//! assert_eq!(text, "\x1B[31mERROR: System is on fire.\x1B[39m");
//! ```
//!
//! ## Reversion Modes
//!
//! By default, the result of every macro in this crate will end with another
//!     control sequence that undoes whatever formatting was set at the start.
//!     For example, the `sgr_bold!()` macro will emit a control sequence to
//!     set bold intensity, the input parameters to the macro, and then a second
//!     control sequence to set normal intensity. Similarly, all coloring macros
//!     will set the default text color when they end.
//!
//! NOTE: The formatting is reverted to the *default* state, ***not*** to the
//!     *previous* state; Nested coloring macros will interfere with each other.
//!
//! To control the behavior of revert sequences, there are two more sigils: `!`
//!     to prevent reverting *any* formatting, and `*` to revert *all*
//!     formatting. Like the Output Mode sigils, these are placed at the
//!     beginning of a macro call. If an output sigil and a revert sigil are
//!     *both* used, the output sigil must be placed first (e.g. `@*` or `%!`).
//! ```
//! use sgr_macros::*;
//!
//! assert_eq!(
//!     //  Here, several layered formatting codes are applied, and then cleared
//!     //      individually. This uses a considerable number of bytes compared
//!     //      to how it might be done manually.
//!     sgr_bold!(sgr_italic!(sgr_uline!("WHAM"))),
//!     "\x1B[1m\x1B[3m\x1B[4mWHAM\x1B[24m\x1B[23m\x1B[22m",
//! );
//!
//! assert_eq!(
//!     //  One way to address this is to specify that `sgr_uline!()` and
//!     //      `sgr_italic!()` do NOT revert, and that `sgr_bold!()` reverts
//!     //      ALL formatting.
//!     sgr_bold!(* sgr_italic!(! sgr_uline!(! "WHAM"))),
//!     "\x1B[1m\x1B[3m\x1B[4mWHAM\x1B[m",
//! );
//!
//! assert_eq!(
//!     //  Here, the color is reset to default twice: Once at the end of Blue,
//!     //      and again at the end of (Not) Red.
//!     red!(@ "Red, {}, (Not) Red", blue!("Blue")),
//!     "\x1B[31mRed, \x1B[34mBlue\x1B[39m, (Not) Red\x1B[39m",
//! );
//!
//! assert_eq!(
//!     //  Here, the color is not reset at the end of Blue, resulting in its
//!     //      blue formatting spilling out to the end of the string. This sort
//!     //      of conflict cannot be solved at compilation while using multiple
//!     //      macros, so it is best to avoid nesting colors whenever possible.
//!     red!(@ "Red, {}, Still Blue", blue!(! "Blue")),
//!     "\x1B[31mRed, \x1B[34mBlue, Still Blue\x1B[39m",
//! );
//! ```

mod old;
mod sgr;

use proc_macro::TokenStream;
use quote::quote;
use sgr::*;


/// Color text with an 8-bit indexed color value.
///
/// # Usage
/// ```
/// assert_eq!(
///     sgr_macros::color_256!(173; "text"),
///     "\x1B[38;5;173mtext\x1B[39m",
/// );
/// ```
///
/// Refer to the [crate] documentation for more information on more advanced
///     macro syntax.
#[proc_macro]
pub fn color_256(stream: TokenStream) -> TokenStream {
    let sgr_256 = syn::parse_macro_input!(stream as Sgr256<false>);
    let tokens = sgr_256.tokens();
    quote!(#tokens).into()
}


/// Color the background with an 8-bit indexed color value.
///
/// # Usage
/// ```
/// assert_eq!(
///     sgr_macros::color_256_bg!(173; "text"),
///     "\x1B[48;5;173mtext\x1B[49m",
/// );
/// ```
///
/// Refer to the [crate] documentation for more information on more advanced
///     macro syntax.
#[proc_macro]
pub fn color_256_bg(stream: TokenStream) -> TokenStream {
    let sgr_256 = syn::parse_macro_input!(stream as Sgr256<true>);
    let tokens = sgr_256.tokens();
    quote!(#tokens).into()
}


/// Color text with a 24-bit RGB value.
///
/// There are several accepted formats for the color specification:
/// - `color_rgb!(0xAABBCC; "text")` (Integer Literal)
/// - `color_rgb!("#AABBCC"; "text")` (String Literal; Also accepts `"#ABC"`)
/// - `color_rgb!((255, 127, 63); "text")` (Integer Tuple)
/// - `color_rgb!((1.0, 0.5, 0.25); "text")` (Float Tuple)
///
/// Refer to the [crate] documentation for more information on more advanced
///     macro syntax.
#[proc_macro]
pub fn color_rgb(stream: TokenStream) -> TokenStream {
    let sgr_rgb = syn::parse_macro_input!(stream as SgrRgb<false>);
    let tokens = sgr_rgb.tokens();
    quote!(#tokens).into()
}


/// Color the background with a 24-bit RGB value.
///
/// Refer to [`color_rgb`] for more information on the color format.
///
/// Refer to the [crate] documentation for more information on more advanced
///     macro syntax.
///
/// [`color_rgb`]: crate::color_rgb!
#[proc_macro]
pub fn color_rgb_bg(stream: TokenStream) -> TokenStream {
    let sgr_rgb = syn::parse_macro_input!(stream as SgrRgb<true>);
    let tokens = sgr_rgb.tokens();
    quote!(#tokens).into()
}


macro_rules! def_sgr {
    ($(
    $(#[$attr:meta])*
    $name:ident = $start:expr, $end:expr;
    )*) => {
        $($(#[$attr])*
        ///
        /// Refer to the [crate] documentation for more information on more
        ///     advanced macro syntax.
        #[proc_macro]
        pub fn $name(stream: TokenStream) -> TokenStream {
            let sgr_base = syn::parse_macro_input!(stream as SgrBase);

            let sgr_fmt = sgr_base.into_format(
                format!("{}", $start),
                format!("{}", $end),
            );

            let tokens = sgr_fmt.tokens();
            quote!(#tokens).into()
        })*
    };
}

def_sgr! {
    /// Make text bold or increased intensity.
    sgr_bold    = 1, 22;
    /// Make text faint or decreased intensity.
    sgr_faint   = 2, 22;
    /// Make text italic.
    sgr_italic  = 3, 23;

    /// Underline text.
    sgr_uline   = 4, 24;
    /// Blink text slowly.
    sgr_blink   = 5, 25;
    /// Blink text quickly. Not widely supported.
    sgr_blink2  = 6, 25;

    /// Invert foreground and background colors.
    sgr_invert  = 7, 27;
    /// Make text invisible. Not widely supported.
    sgr_conceal = 8, 28;
    /// Show text with a horizontal strike, crossing it out.
    sgr_strike  = 9, 29;
}

def_sgr! {
    /// Superscript. Not widely supported.
    sgr_super   = 73, 75;
    /// Subscript. Not widely supported.
    sgr_sub     = 74, 75;
}

def_sgr! {
    /// Color the enclosed text black.
    black           = 30, 39;
    /// Color the enclosed text bright black (grey).
    black_bright    = 90, 99;

    /// Color the enclosed text red.
    red             = 31, 39;
    /// Color the enclosed text bright red.
    red_bright      = 91, 99;

    /// Color the enclosed text green.
    green           = 32, 39;
    /// Color the enclosed text bright green.
    green_bright    = 92, 99;

    /// Color the enclosed text yellow.
    yellow          = 33, 39;
    /// Color the enclosed text bright yellow.
    yellow_bright   = 93, 99;

    /// Color the enclosed text blue.
    blue            = 34, 39;
    /// Color the enclosed text bright blue.
    blue_bright     = 94, 99;

    /// Color the enclosed text magenta.
    magenta         = 35, 39;
    /// Color the enclosed text bright magenta.
    magenta_bright  = 95, 99;

    /// Color the enclosed text cyan.
    cyan            = 36, 39;
    /// Color the enclosed text bright cyan.
    cyan_bright     = 96, 99;

    /// Color the enclosed text white.
    white           = 37, 39;
    /// Color the enclosed text bright white.
    white_bright    = 97, 99;
}

def_sgr! {
    /// Put the enclosed text on a black background.
    black_bg            =  40,  49;
    /// Put the enclosed text on a bright black (grey) background.
    black_bright_bg     = 100, 109;

    /// Put the enclosed text on a red background.
    red_bg              =  41,  49;
    /// Put the enclosed text on a bright red background.
    red_bright_bg       = 101, 109;

    /// Put the enclosed text on a green background.
    green_bg            =  42,  49;
    /// Put the enclosed text on a bright green background.
    green_bright_bg     = 102, 109;

    /// Put the enclosed text on a yellow background.
    yellow_bg           =  43,  49;
    /// Put the enclosed text on a bright yellow background.
    yellow_bright_bg    = 103, 109;

    /// Put the enclosed text on a blue background.
    blue_bg             =  44,  49;
    /// Put the enclosed text on a bright blue background.
    blue_bright_bg      = 104, 109;

    /// Put the enclosed text on a magenta background.
    magenta_bg          =  45,  49;
    /// Put the enclosed text on a bright magenta background.
    magenta_bright_bg   = 105, 109;

    /// Put the enclosed text on a cyan background.
    cyan_bg             =  46,  49;
    /// Put the enclosed text on a bright cyan background.
    cyan_bright_bg      = 106, 109;

    /// Put the enclosed text on a white background.
    white_bg            =  47,  49;
    /// Put the enclosed text on a bright white background.
    white_bright_bg     = 107, 109;
}
