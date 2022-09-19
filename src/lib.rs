//! This crate provides macros for ergonomically wrapping text in ANSI control
//!     sequences with SGR ("Select Graphic Rendition") parameters. These
//!     parameters are used to color text, as well as apply styling such as
//!     italics and underlining. Extensive information on the specific sequences
//!     is available on the Wikipedia page for [ANSI escape codes].
//!
//! Note that not all terminal emulators will support all of the effects
//!     provided in this crate. These macros exist solely to apply the relevant
//!     control sequences, and have no capability to discern whether the
//!     sequences will be meaningful.
//!
//! [ANSI escape codes]: https://en.wikipedia.org/wiki/ANSI_escape_code#SGR
//!
//! ## Modes
//!
//! There are three "output modes" to every macro in this crate: Literal,
//!     Format, and String. These determine the output type of the macro, and
//!     whether it can be called in `const` contexts. Additionally, there are
//!     three "reversion modes": Single, Total, and None. These determine what
//!     is to be done at the end of a macro call --- the formatting state that
//!     should be *reverted*.
//!
//! ### Output Modes
//!
//! The simplest output mode is Literal Mode. A string literal must be supplied,
//!     and all formatting is applied directly, at compile-time. The output of
//!     a Literal Mode macro invocation is a string literal, suitable for the
//!     value of a `const`, or as input to compile-time macros (such as
//!     [`concat!`] or another SGR macro).
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
//! The second mode is Format Mode. An invocation in this mode will resolve to a
//!     call to [`format_args!`]. This will return [`Arguments`] suitable as
//!     input to formatting macros such as [`format!`], [`println!`], and
//!     [`write!`]. This mode is enabled by placing a `%` sigil at the beginning
//!     of the call. After the sigil, a template literal may be provided.
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
//!     call to [`format!`], returning a fully-formed heap-allocated [`String`].
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
//! [`const_format::formatcp!`]: https://docs.rs/const_format/0.2.26/const_format/macro.formatcp.html
//!
//! If the "const" Cargo Feature is enabled, a fourth mode is available: Const
//!     Format Mode. An invocation in this mode will resolve to a call to
//!     [`const_format::formatcp!`], returning a static string slice. This
//!     output is NOT a string literal, however, and is not suitable as input to
//!     [`concat!`]. This mode is enabled with a `#` sigil at the beginning of
//!     the call.
//! ```
//! #[cfg(feature = "const")] {
//!     use sgr_macros::*;
//!
//!     const TEXT: &'static str = sgr_italic!(#*,
//!         "italic {r} {} {b}",
//!         green!(! "green"),
//!         b = blue!(! "blue"),
//!         r = red!(! "red"),
//!     );
//!
//!     assert_eq!(
//!         TEXT,
//!         "\x1B[3mitalic \x1B[31mred \x1B[32mgreen \x1B[34mblue\x1B[m",
//!     );
//! }
//! ```
//!
//! ### Reversion Modes
//!
//! By default, the result of every macro in this crate will end with another
//!     control sequence that undoes whatever formatting was set at the start.
//!     For example, the [`sgr_bold!`] macro will emit a control sequence to
//!     set bold intensity, the input parameters to the macro, and then a second
//!     control sequence to set normal intensity. Similarly, all coloring macros
//!     will set the default text color when they end.
//!
//! Some styles share a revert sequence, meaning that they cannot be safely
//!     nested; The end of the inner style will also revert the outer style.
//!     This is true of the following groups of macros:
//! - [`sgr_bold!`] and [`sgr_faint!`]
//! - [`sgr_blink!`] and [`sgr_blink2!`]
//! - [`sgr_super!`] and [`sgr_sub!`]
//! - All color macros (basic, indexed, and RGB) that do not end in `*_bg`.
//! - All color macros (basic, indexed, and RGB) that **do** end in `*_bg`.
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
//! assert_eq!(
//!     //  One way to address this is to specify that `sgr_uline!` and
//!     //      `sgr_italic!` should NOT revert, and that `sgr_bold!` should
//!     //      revert ALL formatting.
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
//! assert_eq!(
//!     //  Here, the color is not reset at the end of Blue, resulting in its
//!     //      blue formatting spilling out to the end of the string. This sort
//!     //      of conflict cannot be solved at compilation while using multiple
//!     //      macros, so it is best to avoid nesting colors whenever possible.
//!     red!(@ "Red, {}, Still Blue", blue!(! "Blue")),
//!     "\x1B[31mRed, \x1B[34mBlue, Still Blue\x1B[39m",
//! );
//! ```
//!
//! A comma is accepted, but not required, after sigils. This may be helpful for
//!     clarity, or in a case of a dereferenced or inverted argument:
//! ```
//! let text: &&str = &"Doubly-Referenced String";
//!
//! assert_eq!(
//!     sgr_macros::sgr_bold!(@**text), // Very unclear.
//!     "\x1B[1mDoubly-Referenced String\x1B[m",
//! );
//! assert_eq!(
//!     sgr_macros::sgr_bold!(@*, *text), // Much clearer.
//!     "\x1B[1mDoubly-Referenced String\x1B[m",
//! );
//!
//! let mask: u8 = 0b01001001;
//!
//! assert_eq!(
//!     sgr_macros::sgr_uline!(@ !mask), // Interpreted `!` as no-revert.
//!     "\x1B[4m73",
//! );
//! assert_eq!(
//!     sgr_macros::sgr_uline!(@, !mask), // Interpreted `!` as NOT operator.
//!     "\x1B[4m182\x1B[24m",
//! );
//! ```
//!
//! ## Macros
//!
//! ### Basic Color
//!
//! There are eight fundamental colors supported by SGR: Black, red, green,
//!     yellow, blue, magenta, cyan, and white. Each of these 8 colors has a
//!     "bright" variant, leading to 16 named colors. In addition, each of these
//!     16 named colors has two macros: One for *foreground* color, and one for
//!     *background* color.
//!
//! This results in 32 basic color macros; Four macros for each of the eight
//!     fundamental colors.
//! ```
//! assert_eq!(
//!     sgr_macros::cyan!("Bright Cyan Text"),
//!     "\x1B[36mBright Cyan Text\x1B[39m",
//! );
//! assert_eq!(
//!     sgr_macros::cyan_bg!("Text on Bright Cyan"),
//!     "\x1B[46mText on Bright Cyan\x1B[49m",
//! );
//! assert_eq!(
//!     sgr_macros::cyan_bright!("Bright Cyan Text"),
//!     "\x1B[96mBright Cyan Text\x1B[99m",
//! );
//! assert_eq!(
//!     sgr_macros::cyan_bright_bg!("Text on Bright Cyan"),
//!     "\x1B[106mText on Bright Cyan\x1B[109m",
//! );
//! ```
//!
// //! In addition, [`grey!`] and [`grey_bg!`] are provided as more clearly-named
// //!     versions of [`bright_black!`] and [`bright_black_bg!`].
//!
//! ### Indexed Color
//! [8-bit]: https://en.wikipedia.org/wiki/ANSI_escape_code#8-bit
//!
//! Two macros are provided for [8-bit] SGR color codes: [`color_256!`] and
//!     [`color_256_bg!`]. These macros use all the same mode sigils as detailed
//!     [above](#modes), but the first argument of the macro must be an 8-bit
//!     integer, specifying the color index, followed by a semicolon.
//! ```
//! assert_eq!(
//!     sgr_macros::color_256!(173; "Orange Text"),
//!     "\x1B[38;5;173mOrange Text\x1B[39m",
//! );
//! assert_eq!(
//!     sgr_macros::color_256_bg!(173; "Text on Orange"),
//!     "\x1B[48;5;173mText on Orange\x1B[49m",
//! );
//! assert_eq!(
//!     sgr_macros::color_256_bg!(173; *, "Text on Orange"),
//!     "\x1B[48;5;173mText on Orange\x1B[m",
//! );
//! ```
//!
//! ### RGB Color
//! [24-bit]: https://en.wikipedia.org/wiki/ANSI_escape_code#24-bit
//!
//! Two macros are provided for [24-bit] SGR color codes: [`color_rgb!`] and
//!     [`color_rgb_bg!`]. These macros use all the same mode sigils as detailed
//!     [above](#modes), but the first argument of the macro must be an RGB
//!     color value followed by a semicolon.
//! ```
//! assert_eq!(
//!     sgr_macros::color_rgb!(0x420311; "Maroon Text"),
//!     "\x1B[38;2;66;3;17mMaroon Text\x1B[39m",
//! );
//! assert_eq!(
//!     sgr_macros::color_rgb_bg!(0x420311; "Text on Maroon"),
//!     "\x1B[48;2;66;3;17mText on Maroon\x1B[49m",
//! );
//! ```
//!
//! For more information on the RGB color specification, see the documentation
//!     on the [`color_rgb!`] macro.
//!
//! ### Style
//!
//! Eleven macros are provided for various "styles" of text. These typically do
//!     not alter text color, but some aspects, such as text intensity, may be
//!     implemented by a terminal as changing color brightness or vividness.

mod sgr;

use proc_macro::TokenStream;
use quote::quote;
use sgr::*;


/// Color text with an 8-bit indexed color value.
///
/// # Usage
/// ```
/// assert_eq!(
///     sgr_macros::color_256!(173; "Orange Text"),
///     "\x1B[38;5;173mOrange Text\x1B[39m",
/// );
/// ```
///
/// Refer to the [crate] documentation for more information on more advanced
///     macro syntax.
#[proc_macro]
pub fn color_256(stream: TokenStream) -> TokenStream {
    let sgr_256 = syn::parse_macro_input!(stream as Sgr256<false>);
    quote!(#sgr_256).into()
}


/// Color the background with an 8-bit indexed color value.
///
/// # Usage
/// ```
/// assert_eq!(
///     sgr_macros::color_256_bg!(173; "Text on Orange"),
///     "\x1B[48;5;173mText on Orange\x1B[49m",
/// );
/// ```
///
/// Refer to the [crate] documentation for more information on more advanced
///     macro syntax.
#[proc_macro]
pub fn color_256_bg(stream: TokenStream) -> TokenStream {
    let sgr_256 = syn::parse_macro_input!(stream as Sgr256<true>);
    quote!(#sgr_256).into()
}


/// Color text with a 24-bit RGB value.
///
/// # Usage
///
/// There are several accepted formats for the color specification:
/// ```
/// use sgr_macros::*;
///
/// //  Integer Literal:
/// assert_eq!(
///     color_rgb!(0xAABBCC; "Blue-Grey Text"),
///     "\x1B[38;2;170;187;204mBlue-Grey Text\x1B[39m",
/// );
///
/// //  String Literal:
/// assert_eq!(
///     color_rgb!("#AABBCC"; "Blue-Grey Text"),
///     "\x1B[38;2;170;187;204mBlue-Grey Text\x1B[39m",
/// );
///
/// //  String Literal (3-digit):
/// assert_eq!(
///     color_rgb!("#ABC"; "Blue-Grey Text"),
///     "\x1B[38;2;170;187;204mBlue-Grey Text\x1B[39m",
/// );
///
/// //  Integer Tuple:
/// assert_eq!(
///     color_rgb!((255, 127, 63); "Orange Text"),
///     "\x1B[38;2;255;127;63mOrange Text\x1B[39m",
/// );
///
/// //  Float Tuple:
/// assert_eq!(
///     color_rgb!((1.0, 0.5, 0.25); "Orange Text"),
///     "\x1B[38;2;255;127;63mOrange Text\x1B[39m",
/// );
///
/// //  Mixed Tuple:
/// assert_eq!(
///     color_rgb!((0xFF, 127, 0.25); "Orange Text"),
///     "\x1B[38;2;255;127;63mOrange Text\x1B[39m",
/// );
/// ```
///
/// Only three channels are supported, as an Alpha channel is not applicable to
///     text in a terminal. The Integer Literal input format could hold a four
///     byte value, since it is a [`u32`], but use of the top byte will result
///     in a compile error.
///
/// Refer to the [crate] documentation for more information on more advanced
///     macro syntax.
#[proc_macro]
pub fn color_rgb(stream: TokenStream) -> TokenStream {
    let sgr_rgb = syn::parse_macro_input!(stream as SgrRgb<false>);
    quote!(#sgr_rgb).into()
}


/// Color the background with a 24-bit RGB value.
///
/// Refer to the [`color_rgb!`] macro for more information on the color format.
///
/// Refer to the [crate] documentation for more information on more advanced
///     macro syntax.
#[proc_macro]
pub fn color_rgb_bg(stream: TokenStream) -> TokenStream {
    let sgr_rgb = syn::parse_macro_input!(stream as SgrRgb<true>);
    quote!(#sgr_rgb).into()
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
    /// Make text **bold** or more intense.
    sgr_bold    = 1, 22;
    /// Make text faint or less intense.
    sgr_faint   = 2, 22;
    /// Make text *italic*.
    sgr_italic  = 3, 23;

    /// Underline text.
    sgr_uline   = 4, 24;
    /// Blink text slowly.
    sgr_blink   = 5, 25;
    /// Blink text quickly. Not widely supported.
    sgr_blink2  = 6, 25;

    /// Invert foreground and background colors.
    sgr_invert  = 7, 27;
    // /// Reverse foreground and background colors.
    // sgr_reverse = 7, 27;

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
    // /// Color the enclosed text grey.
    // ///
    // /// This macro is identical to [`black_bright!`].
    // grey            = 90, 99;

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
    // /// Put the enclosed text on a grey background.
    // ///
    // /// This macro is identical to [`black_bright_bg!`].
    // grey_bg             = 100, 109;

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
