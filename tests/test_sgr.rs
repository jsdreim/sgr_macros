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
