//! `silicon` is a tool to create beautiful image of your source code.
//!
//! # Example
//!
//! ```
//! use syntect::easy::HighlightLines;
//! use syntect::util::LinesWithEndings;
//! use silicon::utils::init_syntect;
//! use silicon::formatter::ImageFormatterBuilder;
//!
//! let (ps, ts) = init_syntect();
//! let code = r#"
//! fn main() {
//!     println!("Hello, world!");
//! }
//! "#;
//!
//! let syntax = ps.find_syntax_by_token("rs").unwrap();
//! let theme = &ts.themes["Dracula"];
//!
//! let mut h = HighlightLines::new(syntax, theme);
//! let highlight = LinesWithEndings::from(&code)
//!     .map(|line| h.highlight(line, &ps))
//!     .collect::<Vec<_>>();
//!
//! let mut formatter = ImageFormatterBuilder::new().build().unwrap();
//! let image = formatter.format(&highlight, theme);
//!
//! image.save("hello.png").unwrap();
//! ```
#[macro_use]
extern crate log;
#[macro_use]
extern crate failure;

pub mod blur;
pub mod font;
pub mod formatter;
pub mod utils;
