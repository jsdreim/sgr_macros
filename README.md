# SGR Macros

[ANSI escape codes]: https://en.wikipedia.org/wiki/ANSI_escape_code#SGR

This crate provides macros for ergonomically wrapping text in ANSI control sequences with SGR ("Select Graphic Rendition") parameters. These parameters are used to color text, as well as apply styling such as italics and underlining. Extensive information on the specific sequences is available on the Wikipedia page for [ANSI escape codes].

More extensive code examples are available within the crate documentation.

## Modes

There are three "output modes" to every macro in this crate: Literal, Format, and String. These determine the output type of the macro, and whether it can be called in `const` contexts.

Additionally, there are three "reversion modes": Single, Total, and None. These determine what is to be done at the end of a macro call --- the formatting state that should be *reverted*.

### Output Modes

The simplest output mode is Literal Mode. A string literal must be supplied, and all formatting is applied directly, at compile-time. The output of a Literal Mode macro invocation is a `&str` literal suitable for input to `concat!`.

The second mode is Format Mode. An invocation in this mode will resolve to a call to `format_args!`. This will return `Arguments` suitable as input parameters for formatting macros such as `format!`, `println!`, and `write!`. This mode is enabled by placing a `%` sigil at the beginning of the call. After the sigil, a template literal may be provided.

The third mode is String Mode. An invocation in this mode will resolve to a call to `format!`, returning a fully-formed heap-allocated `String`. This mode is enabled with a `@` sigil at the beginning of the call, and it may also be provided a template literal.

If the "const" Cargo Feature is enabled, a fourth mode is available: Const Format Mode. An invocation in this mode will resolve to a call to [`formatcp!`], returning a static string slice. This output is NOT a string literal, however, and is not suitable as input to `concat!`. This mode is enabled with a `#` sigil at the beginning of the call.

### Reversion Modes

By default, the result of every macro in this crate will end with another control sequence that undoes whatever formatting was set at the start. For example, the `sgr_bold!` macro will emit a control sequence to set bold intensity, the input parameters to the macro, and then a second control sequence to set normal intensity. Similarly, all coloring macros will set the default text color when they end.

Some styles share a revert sequence, meaning that they cannot be safely nested; The end of the inner style will also revert the outer style. This is true of the following groups of macros:
- `sgr_bold!` and `sgr_faint!`
- `sgr_blink!` and `sgr_blink2!`
- `sgr_super!` and `sgr_sub!`
- All color macros (basic, indexed, and RGB) that do not end in `*_bg`.
- All color macros (basic, indexed, and RGB) that **do** end in `*_bg`.

To control the behavior of revert sequences, there are two more sigils: `!` to prevent reverting *any* formatting, and `*` to revert *all* formatting. Like the Output Mode sigils, these are placed at the beginning of a macro call. If an output sigil and a revert sigil are *both* used, the output sigil must be placed first (e.g. `@*` or `%!`).

A comma is accepted, but not required, after sigils. This may be helpful for clarity, or in a case of a dereferenced or inverted argument.

## Macros

### Basic Color

There are eight fundamental colors supported by SGR: Black, red, green, yellow, blue, magenta, cyan, and white. Each of these 8 colors has a "bright" variant, leading to 16 named colors. In addition, each of these 16 named colors has two macros: One for *foreground* color, and one for *background* color.

This results in 32 basic color macros; Four macros for each of the eight fundamental colors.

[//]: # (In addition, `grey!` and `grey_bg!` are provided as more clearly-named versions of `bright_black!` and `bright_black_bg!`.)

### Indexed Color
[8-bit]: https://en.wikipedia.org/wiki/ANSI_escape_code#8-bit

Two macros are provided for [8-bit] SGR color codes: `color_256!` and `color_256_bg!`. These macros use all the same mode sigils as detailed [above](#modes), but the first argument of the macro must be an 8-bit integer, specifying the color index, followed by a semicolon.

### RGB Color
[24-bit]: https://en.wikipedia.org/wiki/ANSI_escape_code#24-bit

Two macros are provided for [24-bit] SGR color codes: `color_rgb!` and `color_rgb_bg!`. These macros use all the same mode sigils as detailed [above](#modes), but the first argument of the macro must be an RGB color value followed by a semicolon.

For more information on the RGB color specification, see the documentation on the `color_rgb!` macro.

### Style

Eleven macros are provided for various "styles" of text. These typically do not alter text color, but some aspects, such as text intensity, may be implemented by changing color brightness or vividness.

## Cargo Features

If this library has the "const" Cargo feature enabled, support for the [const_format](https://crates.io/crates/const_format) crate will be available. This functionality is accessed via a new Output Mode. Using SGR macros with a `#` mode sigil will then also support template literals. At the time of this writing, this will resolve to a call to the [`formatcp!`] macro.


[`formatcp!`]: https://docs.rs/const_format/0.2.26/const_format/macro.formatcp.html
