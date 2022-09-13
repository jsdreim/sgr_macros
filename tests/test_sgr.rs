use sgr_macros::*;


#[test]
fn test_sgr_concat() {
    let text: &str = green!("green text");
    assert_eq!(text, "\x1B[32mgreen text\x1B[39m");

    let text: &str = green!(! "green text non-reverted");
    assert_eq!(text, "\x1B[32mgreen text non-reverted");

    let text: &str = sgr_italic!(
        "italic, ",
        green!("italic green"),
        ", ",
        red!(!"red")
    );
    // eprintln!("a {text} z");
    assert_eq!(text, "\x1B[3mitalic, \x1B[32mitalic green\x1B[39m, \x1B[31mred\x1B[23m");
}


// #[test]
// fn test_sgr_format() {
// }


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
    assert_eq!(text, "\x1B[3mitalic \x1B[31mred \x1B[32mgreen\x1B[39m \x1B[34mblue\x1B[m");
}


#[test]
fn test_sgr_rgb() {
    let text = sgr_fg_rgb!(0xFF55FF; "RGB text");
    // eprintln!("{}", text);
    assert_eq!(text, "\x1B[38;2;255;85;255mRGB text\x1B[39m");

    let text = sgr_bg_rgb!(0x420311; "RGB text");
    // eprintln!("{}", text);
    assert_eq!(text, "\x1B[48;2;66;3;17mRGB text\x1B[49m");

    let text = sgr_fg_rgb!(#255,128,0; "RGB text");
    // eprintln!("{}", text);
    assert_eq!(text, "\x1B[38;2;255;128;0mRGB text\x1B[39m");

    let text = sgr_bg_rgb!(0x553355; ! "RGB text");
    // eprintln!("{}", text);
    assert_eq!(text, "\x1B[48;2;85;51;85mRGB text");

    let text = sgr_bg_rgb!(0x335555; * "RGB text");
    // eprintln!("{}", text);
    assert_eq!(text, "\x1B[48;2;51;85;85mRGB text\x1B[m");
}


#[test]
fn test_sgr_256() {
    let text = sgr_fg_256!(255; "Indexed-color text");
    // eprintln!("{}", text);
    assert_eq!(text, "\x1B[38;5;255mIndexed-color text\x1B[39m");

    let text = sgr_fg_256!(173; "Indexed-color text");
    // eprintln!("{}", text);
    assert_eq!(text, "\x1B[38;5;173mIndexed-color text\x1B[39m");

    let text = sgr_bg_256!(64; ! "Indexed-color text");
    // eprintln!("{}", text);
    assert_eq!(text, "\x1B[48;5;64mIndexed-color text");

    let text = sgr_bg_256!(128; * "Indexed-color text");
    // eprintln!("{}", text);
    assert_eq!(text, "\x1B[48;5;128mIndexed-color text\x1B[m");
}
