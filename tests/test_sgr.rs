use std::fmt::Arguments;
use sgr_macros::*;


macro_rules! edbg {
    ($v:expr) => {{
        eprintln!("{}", $v);
        $v
    }};
}


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
    assert_eq!(
        text,
        "\x1B[3mitalic \x1B[31mred \x1B[32mgreen\x1B[39m \x1B[34mblue\x1B[m",
    );
}


#[test]
fn test_sgr_rgb() {
    //  Literal `0xRRGGBB` hex format.
    assert_eq!(
        color_rgb_bg!(0x420311; "RGB text"),
        "\x1B[48;2;66;3;17mRGB text\x1B[49m",
    );

    //  Maximum value.
    assert_eq!(
        color_rgb!(0xFFFFFF; "RGB text"),
        "\x1B[38;2;255;255;255mRGB text\x1B[39m",
    );

    // //  Above maximum value; Should not compile.
    // assert_eq!(
    //     color_rgb!(0x1000000; "RGB text"),
    //     "\x1B[38;2;255;255;255mRGB text\x1B[39m",
    // );

    assert_eq!(
        color_rgb!(0xFF55FF; "RGB text"),
        "\x1B[38;2;255;85;255mRGB text\x1B[39m",
    );

    //  String "0xRRGGBB" hex format.
    assert_eq!(
        color_rgb!("0xFF55FF"; "RGB text"),
        "\x1B[38;2;255;85;255mRGB text\x1B[39m",
    );

    //  String "#RRGGBB" hex format.
    assert_eq!(
        color_rgb!("#FF55FF"; "RGB text"),
        "\x1B[38;2;255;85;255mRGB text\x1B[39m",
    );

    //  Tuple format (all integers).
    assert_eq!(
        color_rgb!((255, 127, 0); "RGB text"),
        "\x1B[38;2;255;127;0mRGB text\x1B[39m",
    );

    //  Tuple format (with floats).
    assert_eq!(
        color_rgb!((1.0, 0.5, 0); "RGB text"),
        "\x1B[38;2;255;127;0mRGB text\x1B[39m",
    );

    //  Confirm equivalence between zero forms.
    assert_eq!(
        color_rgb!(0x000; "RGB text"),
        color_rgb!(0x000000; "RGB text"),
    );
    assert_eq!(
        color_rgb!(0x000; "RGB text"),
        color_rgb!(0; "RGB text"),
    );

    //  Confirm equivalence between nonzero forms.
    assert_eq!(
        color_rgb!(0xFF7F00; "RGB text"),
        color_rgb!("#FF7F00"; "RGB text"),
    );
    assert_eq!(
        color_rgb!(0xFF7F00; "RGB text"),
        color_rgb!((0xFF, 0x7F, 0x00); "RGB text"),
    );
    assert_eq!(
        color_rgb!(0xFF7F00; "RGB text"),
        color_rgb!((255, 127, 0); "RGB text"),
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
    assert_eq!(
        color_rgb_bg!(0x553355; ! "RGB text"),
        "\x1B[48;2;85;51;85mRGB text",
    );

    //  Confirm that all-revert mode works properly.
    assert_eq!(
        color_rgb_bg!(0x335555; * "RGB text"),
        "\x1B[48;2;51;85;85mRGB text\x1B[m",
    );
}


#[test]
fn test_sgr_256() {
    assert_eq!(
        color_256!(255; "Indexed-color text"),
        "\x1B[38;5;255mIndexed-color text\x1B[39m",
    );

    assert_eq!(
        color_256!(173; "Indexed-color text"),
        "\x1B[38;5;173mIndexed-color text\x1B[39m",
    );

    assert_eq!(
        color_256_bg!(64; ! "Indexed-color text"),
        "\x1B[48;5;64mIndexed-color text",
    );

    assert_eq!(
        color_256_bg!(128; * "Indexed-color text"),
        "\x1B[48;5;128mIndexed-color text\x1B[m",
    );
}


#[test]
fn test_unified_color() {
    //  Test all foreground set modes.
    assert_eq!(
        edbg!(color!("green"; "Green Text")),
        "\x1B[32mGreen Text\x1B[39m",
    );
    assert_eq!(
        edbg!(color!(73; "Indexed Text")),
        "\x1B[38;5;73mIndexed Text\x1B[39m",
    );
    assert_eq!(
        edbg!(color!(0xFF7F00; "RGB Text")),
        "\x1B[38;2;255;127;0mRGB Text\x1B[39m",
    );

    //  Test all background set modes.
    assert_eq!(
        edbg!(color!(in "green"; "Text on Green")),
        "\x1B[42mText on Green\x1B[49m",
    );
    assert_eq!(
        edbg!(color!(in 73; "Text on Indexed")),
        "\x1B[48;5;73mText on Indexed\x1B[49m",
    );
    assert_eq!(
        edbg!(color!(in 0xFF7F00; "Text on RGB")),
        "\x1B[48;2;255;127;0mText on RGB\x1B[49m",
    );

    //  Test combined fg+bg set modes.
    assert_eq!(
        edbg!(color!("bright yellow" in "red"; "Yellow Text on Red")),
        "\x1B[93;41mYellow Text on Red\x1B[39;49m",
    );
    assert_eq!(
        edbg!(color!(0xFF7F00 in 73; "RGB Text on Indexed")),
        "\x1B[38;2;255;127;0;48;5;73mRGB Text on Indexed\x1B[39;49m",
    );
    assert_eq!(
        edbg!(color!(73 in 0xFF7F00; "Indexed Text on RGB")),
        "\x1B[38;5;73;48;2;255;127;0mIndexed Text on RGB\x1B[39;49m",
    );

    //  Test revert specification.
    assert_eq!(
        edbg!(color!("green"; "Green Text"; in "red")),
        "\x1B[32mGreen Text\x1B[39;41m",
    );
    assert_eq!(
        edbg!(color!("green"; "Green Text"; "bright yellow" in "red")),
        "\x1B[32mGreen Text\x1B[93;41m",
    );
}
