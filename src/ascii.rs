#[rustfmt::skip]
pub const DOTS: [&'static str; 5] = [
    r#"   "#,
    r#" _ "#,
    r#"(_)"#,
    r#" _ "#,
    r#"(_)"#,
];

#[rustfmt::skip]
pub const ONE: [&'static str; 5] = [
    r#" _ "#,
    r#"/ |"#,
    r#"| |"#,
    r#"| |"#,
    r#"|_|"#,
];

#[rustfmt::skip]
pub const TWO: [&'static str; 5] = [
    r#" ____  "#,
    r#"|___ \ "#,
    r#"  __) |"#,
    r#" / __/ "#,
    r#"|_____|"#,
];

#[rustfmt::skip]
pub const THREE: [&'static str; 5] = [
    r#" _____ "#,
    r#"|___ / "#,
    r#"  |_ \ "#,
    r#" ___) |"#,
    r#"|____/ "#,
];

#[rustfmt::skip]
pub const FOUR: [&'static str; 5] = [
    r#" _  _   "#,
    r#"| || |  "#,
    r#"| || |_ "#,
    r#"|__   _|"#,
    r#"   |_|  "#,
];

#[rustfmt::skip]
pub const FIVE: [&'static str; 5] = [
    r#" ____  "#,
    r#"| ___| "#,
    r#"|___ \ "#,
    r#" ___) |"#,
    r#"|____/ "#,
];

#[rustfmt::skip]
pub const SIX: [&'static str; 5] = [
    r#"  __   "#,
    r#" / /_  "#,
    r#"| '_ \ "#,
    r#"| (_) |"#,
    r#" \___/ "#,
];

#[rustfmt::skip]
pub const SEVEN: [&'static str; 5] = [
    r#" _____ "#,
    r#"|___  |"#,
    r#"   / / "#,
    r#"  / /  "#,
    r#" /_/   "#,
];

#[rustfmt::skip]
pub const EIGHT: [&'static str; 5] = [
    r#"  ___  "#,
    r#" ( _ ) "#,
    r#" / _ \ "#,
    r#"| (_) |"#,
    r#" \___/ "#,
];

#[rustfmt::skip]
pub const NINE: [&'static str; 5] = [
    r#"  ___  "#,
    r#" / _ \ "#,
    r#"| (_) |"#,
    r#" \__, |"#,
    r#"   /_/ "#,
];

#[rustfmt::skip]
pub const ZERO: [&'static str; 5] = [
    r#"  ___  "#,
    r#" / _ \ "#,
    r#"| | | |"#,
    r#"| |_| |"#,
    r#" \___/ "#,
];

pub trait Ascii {
    // Convert to ASCII art.
    fn to_ascii_art(&self) -> String;
}
