use std::fmt::Arguments;
use sgr_macros::*;


#[test]
fn test_sgr_concat() {
    let text: &str = green!("green text");
    assert_eq!(text, "\x1B[32mgreen text\x1B[39m");

    let text: &str = green!(! "green text non-reverted");
    assert_eq!(text, "\x1B[32mgreen text non-reverted");

    //  NOTE: This is not a feature, just a side effect of reading all tokens,
    //      combined with the tokens being passed directly into `concat!`. It
    //      does not seem to be a negative effect, but this behavior probably
    //      should not be guaranteed going forward.
    let text: &str = sgr_italic!(*,
        "Italic, ",
        green!("Italic Green"),
        ", ",
        red!(!"Italic Red"),
    );
    // eprintln!("a {text} z");
    assert_eq!(
        text,
        "\x1B[3mItalic, \x1B[32mItalic Green\x1B[39m, \x1B[31mItalic Red\x1B[m",
    );
}


#[cfg(feature = "const")]
#[test]
fn test_sgr_const() {
    let text: &str = green!("green text");
    assert_eq!(text, "\x1B[32mgreen text\x1B[39m");

    let text: &str = green!(! "green text non-reverted");
    assert_eq!(text, "\x1B[32mgreen text non-reverted");

    let text: &str = sgr_italic!(#*,
        "italic {r} {} {b}",
        green!("green"),
        b = blue!(! "blue"),
        r = red!(! "red"),
    );
    // eprintln!("a {text} z");
    assert_eq!(
        text,
        "\x1B[3mitalic \x1B[31mred \x1B[32mgreen\x1B[39m \x1B[34mblue\x1B[m",
    );
}


// #[cfg(not(feature = "const"))]
// #[test]
// #[should_panic] // TODO: Any way to specify that it should not *compile*?
// fn test_sgr_const_fail() {
//     let _err: &str = sgr_italic!(#*,
//         "italic {r} {} {b}",
//         green!("green"),
//         b = blue!(! "blue"),
//         r = red!(! "red"),
//     );
//     // eprintln!("a {_err} z");
// }


#[test]
fn test_sgr_format() {
    let args: Arguments = green!(% "green text");
    let text: String = args.to_string();
    assert_eq!(text, "\x1B[32mgreen text\x1B[39m");

    let args: Arguments = green!(%* "green text");
    let text: String = args.to_string();
    assert_eq!(text, "\x1B[32mgreen text\x1B[m");

    let args: Arguments = green!(%! "green text");
    let text: String = args.to_string();
    assert_eq!(text, "\x1B[32mgreen text");
}


#[test]
fn test_sgr_string() {
    let text: String = green!(@ "green text");
    assert_eq!(text, "\x1B[32mgreen text\x1B[39m");

    let text: String = green!(@! "green text non-reverted");
    assert_eq!(text, "\x1B[32mgreen text non-reverted");

    let text: String = sgr_italic!(@*,
        "italic {r} {} {b}",
        green!("green"),
        b = blue!(! "blue"),
        r = red!(! "red"),
    );
    // eprintln!("a {text} z");
    assert_eq!(
        text,
        "\x1B[3mitalic \x1B[31mred \x1B[32mgreen\x1B[39m \x1B[34mblue\x1B[m",
    );
}


#[test]
fn test_sgr_rgb() {
    //  Literal `0xRRGGBB` hex format.
    let text = color_rgb_bg!(0x420311; "RGB text");
    // eprintln!("{}", text);
    assert_eq!(text, "\x1B[48;2;66;3;17mRGB text\x1B[49m");

    let text = color_rgb!(0xFF55FF; "RGB text");
    // eprintln!("{}", text);
    assert_eq!(text, "\x1B[38;2;255;85;255mRGB text\x1B[39m");

    //  String "0xRRGGBB" hex format.
    let text = color_rgb!("0xFF55FF"; "RGB text");
    // eprintln!("{}", text);
    assert_eq!(text, "\x1B[38;2;255;85;255mRGB text\x1B[39m");

    //  String "#RRGGBB" hex format.
    let text = color_rgb!("#FF55FF"; "RGB text");
    // eprintln!("{}", text);
    assert_eq!(text, "\x1B[38;2;255;85;255mRGB text\x1B[39m");

    //  Tuple format (all integers).
    let text = color_rgb!((255, 127, 0); "RGB text");
    // eprintln!("{}", text);
    assert_eq!(text, "\x1B[38;2;255;127;0mRGB text\x1B[39m");

    //  Tuple format (with floats).
    let text = color_rgb!((1.0, 0.5, 0); "RGB text");
    // eprintln!("{}", text);
    assert_eq!(text, "\x1B[38;2;255;127;0mRGB text\x1B[39m");

    //  Confirm equivalence between zero forms.
    assert_eq!(
        color_rgb!(0x000; "RGB text"),
        color_rgb!(0x000000; "RGB text"),
    );
    assert_eq!(
        color_rgb!(0x000; "RGB text"),
        color_rgb!(0; "RGB text"),
    );

    //  Confirm that expansion of `RGB` into `RRGGBB` works properly.
    assert_eq!(
        color_rgb!("#ABC"; "RGB text"),
        color_rgb!("#AABBCC"; "RGB text"),
    );
    // assert_eq!(
    //     sgr_fg_rgb!(0xABC; "RGB text"),
    //     sgr_fg_rgb!(0xAABBCC; "RGB text"),
    // );
    // assert_eq!(
    //     sgr_fg_rgb!(0xFFF; "RGB text"),
    //     sgr_fg_rgb!(0xFFFFFF; "RGB text"),
    // );
    // assert_eq!(
    //     sgr_fg_rgb!("#ABC"; "RGB text"),
    //     sgr_fg_rgb!(0xAABBCC; "RGB text"),
    // );
    // assert_eq!(
    //     sgr_fg_rgb!(0xABC; "RGB text"),
    //     sgr_fg_rgb!("#AABBCC"; "RGB text"),
    // );

    //  Confirm that non-revert mode works properly.
    let text = color_rgb_bg!(0x553355; ! "RGB text");
    // eprintln!("{}", text);
    assert_eq!(text, "\x1B[48;2;85;51;85mRGB text");

    //  Confirm that all-revert mode works properly.
    let text = color_rgb_bg!(0x335555; * "RGB text");
    // eprintln!("{}", text);
    assert_eq!(text, "\x1B[48;2;51;85;85mRGB text\x1B[m");
}


#[test]
fn test_sgr_256() {
    let text = color_256!(255; "Indexed-color text");
    // eprintln!("{}", text);
    assert_eq!(text, "\x1B[38;5;255mIndexed-color text\x1B[39m");

    let text = color_256!(173; "Indexed-color text");
    // eprintln!("{}", text);
    assert_eq!(text, "\x1B[38;5;173mIndexed-color text\x1B[39m");

    let text = color_256_bg!(64; ! "Indexed-color text");
    // eprintln!("{}", text);
    assert_eq!(text, "\x1B[48;5;64mIndexed-color text");

    let text = color_256_bg!(128; * "Indexed-color text");
    // eprintln!("{}", text);
    assert_eq!(text, "\x1B[48;5;128mIndexed-color text\x1B[m");
}
