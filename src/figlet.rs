use crate::{path, Result};
use serde::{
    de::{self, Visitor},
    Deserialize,
};
use std::{
    collections::HashMap,
    error,
    fmt::{self, Display},
    fs::File,
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
    result,
};

#[rustfmt::skip]
const DOTS: [&str; 5] = [
    r#"   "#,
    r#" _ "#,
    r#"(_)"#,
    r#" _ "#,
    r#"(_)"#,
];

#[rustfmt::skip]
const ONE: [&str; 5] = [
    r#" _ "#,
    r#"/ |"#,
    r#"| |"#,
    r#"| |"#,
    r#"|_|"#,
];

#[rustfmt::skip]
const TWO: [&str; 5] = [
    r#" ____  "#,
    r#"|___ \ "#,
    r#"  __) |"#,
    r#" / __/ "#,
    r#"|_____|"#,
];

#[rustfmt::skip]
const THREE: [&str; 5] = [
    r#" _____ "#,
    r#"|___ / "#,
    r#"  |_ \ "#,
    r#" ___) |"#,
    r#"|____/ "#,
];

#[rustfmt::skip]
const FOUR: [&str; 5] = [
    r#" _  _   "#,
    r#"| || |  "#,
    r#"| || |_ "#,
    r#"|__   _|"#,
    r#"   |_|  "#,
];

#[rustfmt::skip]
const FIVE: [&str; 5] = [
    r#" ____  "#,
    r#"| ___| "#,
    r#"|___ \ "#,
    r#" ___) |"#,
    r#"|____/ "#,
];

#[rustfmt::skip]
const SIX: [&str; 5] = [
    r#"  __   "#,
    r#" / /_  "#,
    r#"| '_ \ "#,
    r#"| (_) |"#,
    r#" \___/ "#,
];

#[rustfmt::skip]
const SEVEN: [&str; 5] = [
    r#" _____ "#,
    r#"|___  |"#,
    r#"   / / "#,
    r#"  / /  "#,
    r#" /_/   "#,
];

#[rustfmt::skip]
const EIGHT: [&str; 5] = [
    r#"  ___  "#,
    r#" ( _ ) "#,
    r#" / _ \ "#,
    r#"| (_) |"#,
    r#" \___/ "#,
];

#[rustfmt::skip]
const NINE: [&str; 5] = [
    r#"  ___  "#,
    r#" / _ \ "#,
    r#"| (_) |"#,
    r#" \__, |"#,
    r#"   /_/ "#,
];

#[rustfmt::skip]
const ZERO: [&str; 5] = [
    r#"  ___  "#,
    r#" / _ \ "#,
    r#"| | | |"#,
    r#"| |_| |"#,
    r#" \___/ "#,
];

#[rustfmt::skip]
const EXCLAMATION: [&str; 5] = [
    r#" _ "#,
    r#"| |"#,
    r#"| |"#,
    r#"|_|"#,
    r#"(_)"#,
];

#[derive(Debug, Clone)]
/// FIGlet font character.
struct Char(Vec<String>);

impl From<[&'static str; 5]> for Char {
    fn from(array: [&'static str; 5]) -> Self {
        Self(array.map(|val| val.to_string()).to_vec())
    }
}

#[derive(Debug, Clone)]
/// FIGlet font.
pub struct Font {
    // Numbers
    zero: Char,
    one: Char,
    two: Char,
    three: Char,
    four: Char,
    five: Char,
    six: Char,
    seven: Char,
    eight: Char,
    nine: Char,

    // Letters
    // TODO
    // POMODORO COMPLETED
    // TIMER EXPIRED!
    // PAUSED

    // Symbols
    dots: Char,
    exclamation: Char,
}

impl Default for Font {
    fn default() -> Self {
        Self {
            // Numbers
            zero: ZERO.into(),
            one: ONE.into(),
            two: TWO.into(),
            three: THREE.into(),
            four: FOUR.into(),
            five: FIVE.into(),
            six: SIX.into(),
            seven: SEVEN.into(),
            eight: EIGHT.into(),
            nine: NINE.into(),

            // Letters
            // TODO

            // Symbols
            dots: DOTS.into(),
            exclamation: EXCLAMATION.into(),
        }
    }
}

impl Font {
    /// Parse [`Font`] from FIGlet font file (`.flf`).
    pub fn parse_flf(path: &Path) -> Result<Font> {
        // Check if `path` is file.
        if !path.is_file() {
            return Err(FontError::NotAFile(path.to_path_buf()).into());
        }
        // Check if `path` points to a file with the correct extension (`.flf`).
        if path.extension().is_none() || path.extension().unwrap() != "flf" {
            return Err(FontError::InvalidExtension(path.to_path_buf()).into());
        }

        // Read file into vector of strings (each string is a line).
        // This can't panic since we assured the file exists: safe to unwrap.
        let font_file = File::open(path).unwrap();
        let lines = BufReader::new(font_file)
            .lines()
            .collect::<result::Result<Vec<String>, _>>()?;

        // Retrurn invalid `.flf` FIGlet font file error.
        let invalid =
            |path: &Path| -> crate::error::Error { FontError::InvalidFile(path.into()).into() };

        // Check if font file starts with "flf2a", as
        // "The first five characters in the entire file must be 'flf2a'".
        if !lines[0].starts_with("flf2a") {
            return Err(invalid(path));
        }

        // FIGlet font file header.
        let header: Vec<&str> = lines[0][5..].split_whitespace().collect();

        // Parse usize from string and map error.
        let parse_num =
            |v: &str| -> Result<usize> { v.parse::<usize>().map_err(|_| invalid(path)) };

        // Select *Hardblank* and *Height* fields from FIGlet font file header.
        let hard_blank = header[0];
        let char_height = parse_num(header[1])?;

        // Initialize `i` to firs non-comment line (`header[5]` contains the number of comment
        // lines in file).
        let mut i = parse_num(header[5])? + 1;

        // Extract line *endmark*.
        let Some(endmark) = &lines[i].chars().last() else {
            return Err(invalid(path));
        };

        /// List of required characters for valid FIGlet font.
        const CHAR_LIST: [char; 95] = [
            ' ', '!', '"', '#', '$', '%', '&', '\'', '(', ')', '*', '+', ',', '-', '.', '/', '0',
            '1', '2', '3', '4', '5', '6', '7', '8', '9', ':', ';', '<', '=', '>', '?', '@', 'A',
            'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R',
            'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z', '[', '\\', ']', '^', '_', '`', 'a', 'b', 'c',
            'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't',
            'u', 'v', 'w', 'x', 'y', 'z', '{', '|', '}', '~',
        ];

        // Parsed characters.
        let mut chars: Vec<Char> = Vec::with_capacity(CHAR_LIST.len());
        // Loop until EOF or until all required characters are parsed.
        while i < lines.len() || chars.len() < CHAR_LIST.len() {
            while !&lines[i].ends_with(*endmark) {
                i += 1
            }

            let j = i + char_height;
            chars.push(Char(
                lines[i..j]
                    .iter()
                    .map(|line| line.replace(hard_blank, " ").replace(*endmark, ""))
                    .collect::<Vec<String>>(),
            ));

            i = j;
        }

        // Ensure all required characters are parsed, if not return invalid font file error.
        if chars.len() < CHAR_LIST.len() {
            return Err(invalid(path));
        }

        // Generate HashMap with `char` as key and `Char` as value.
        let mut map: HashMap<char, Char> = CHAR_LIST.into_iter().zip(chars).collect();

        // Construct Font (safe to unwrap, because we ensured the map contains required keys).
        Ok(Font {
            zero: map.remove(&'0').unwrap(),
            one: map.remove(&'1').unwrap(),
            two: map.remove(&'2').unwrap(),
            three: map.remove(&'3').unwrap(),
            four: map.remove(&'4').unwrap(),
            five: map.remove(&'5').unwrap(),
            six: map.remove(&'6').unwrap(),
            seven: map.remove(&'7').unwrap(),
            eight: map.remove(&'8').unwrap(),
            nine: map.remove(&'9').unwrap(),

            dots: map.remove(&':').unwrap(),
            exclamation: map.remove(&'!').unwrap(),
        })
    }

    /// Convert string to FIGlet text string.
    pub fn convert(&self, string: &str) -> String {
        let mut figlet_text: Vec<String> = vec!["".to_string(); self.zero.0.len()];

        for c in string.chars() {
            let figlet_char = &match c {
                // Numbers.
                '0' => &self.zero,
                '1' => &self.one,
                '2' => &self.two,
                '3' => &self.three,
                '4' => &self.four,
                '5' => &self.five,
                '6' => &self.six,
                '7' => &self.seven,
                '8' => &self.eight,
                '9' => &self.nine,

                // Letters.
                // TODO

                // Symbols.
                ':' => &self.dots,
                '!' => &self.exclamation,
                _ => panic!("unsupported figlet character"),
            }
            .0;

            for (i, line) in figlet_text.iter_mut().enumerate() {
                line.push_str(&figlet_char[i]);
            }
        }

        figlet_text.join("\n")
    }
}

struct FontVisitor;

impl<'de> Visitor<'de> for FontVisitor {
    type Value = Font;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("path to `.flf` (FIGlet font file)")
    }

    fn visit_str<E>(self, v: &str) -> result::Result<Self::Value, E>
    where
        E: de::Error,
    {
        // Absolutize and validate Path, then Parse `.flf` font file.
        let path = path::absolutize_path(v).map_err(de::Error::custom)?;
        Font::parse_flf(&path).map_err(de::Error::custom)
    }
}

impl<'de> Deserialize<'de> for Font {
    fn deserialize<D>(deserializer: D) -> result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_str(FontVisitor)
    }
}

/// FIGlet font parse error.
#[derive(Debug, Clone)]
pub enum FontError {
    /// Occurs when path is not a file.
    NotAFile(PathBuf),
    /// Occurs when path is file but extension is not `.flf`.
    InvalidExtension(PathBuf),
    /// Occurs when path is not a valid FIGlet font file.
    InvalidFile(PathBuf),
    /// Occurs when font file contains non-UTF8 text.
    NonUtf8,
}

impl Display for FontError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotAFile(path) => write!(f, "provided path `{}` is not a file", path.display()),
            Self::InvalidExtension(path) => write!(
                f,
                "provided path `{}` does not match FIGlet font file extension `.flf`",
                path.display()
            ),
            Self::InvalidFile(path) => {
                write!(
                    f,
                    "provided path `{}` is not a valid FIGlet font file",
                    path.display()
                )
            }
            Self::NonUtf8 => write!(f, "FIGlet font file contains invalid, non-UTF8 text"),
        }
    }
}

impl error::Error for FontError {}

pub trait Figlet {
    // Convert to FIGlet text.
    fn to_figlet(&self, font: &Font) -> String;
}
