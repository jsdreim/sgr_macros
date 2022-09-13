use sgr_macros::*;


#[test]
fn test_sgr_concat() {
    let text: &str = green!("green text");
    assert_eq!(text, "\x1B[32mgreen text\x1B[m");

    let text: &str = green!(! "green text non-reverted");
    assert_eq!(text, "\x1B[32mgreen text non-reverted");

    let text: &str = sgr_bold!(
        "bold, ",
        green!("bold green"),
        ", ",
        red!(!"red")
    );
    // eprintln!("{text}");
    assert_eq!(text, "\x1B[1mbold, \x1B[32mbold green\x1B[m, \x1B[31mred\x1B[m");
}


// #[test]
// fn test_sgr_format() {
// }


#[test]
fn test_sgr_string() {
    let text: String = green!(@ "green text");
    assert_eq!(text, "\x1B[32mgreen text\x1B[m");

    let text: String = green!(@! "green text non-reverted");
    assert_eq!(text, "\x1B[32mgreen text non-reverted");

    let text: String = sgr_bold!(@
        "bold {r} {} {b}",
        green!(! "green"),
        b = blue!(! "blue"),
        r = red!(! "red"),
    );
    // eprintln!("{text}");
    assert_eq!(text, "\x1B[1mbold \x1B[31mred \x1B[32mgreen \x1B[34mblue\x1B[m");
}
