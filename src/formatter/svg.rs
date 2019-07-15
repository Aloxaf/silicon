use syntect::highlighting::{FontStyle, Style, Theme};
use crate::font::FontCollection;

trait ToHtml {
    fn to_html(&self) -> String;
}

impl ToHtml for syntect::highlighting::Color {
    fn to_html(&self) -> String {
        format!("#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
    }
}

fn get_style(style: FontStyle) -> &'static str {
    if style.contains(FontStyle::ITALIC) {
        "italic"
    } else {
        "normal"
    }
}

fn get_weight(style: FontStyle) -> &'static str {
    if style.contains(FontStyle::BOLD) {
        "bold"
    } else {
        "normal"
    }
}

// TODO: Create a Formatter trait ??
pub struct SVGFormatter {
    width: u32,
    height: u32,
}

impl SVGFormatter {
    fn get_svg_size(v: &[&str]) -> (u32, u32) {
        let font = FontCollection::new(&[("monospace", 17.0)]).unwrap();
        let height = font.get_font_height() * (v.len() + 1) as u32;
        let width = v.iter().map(|s| font.get_text_len(s)).max().unwrap();
        (width, height)
    }

    // TODO: don't concat string...
    pub fn format(&mut self, v: &[Vec<(Style, &str)>], theme: &Theme) -> String {
        let mut svg = format!(
            r#"<svg width="{}" height="{}" style="border: 0px solid black" xmlns="http://www.w3.org/2000/svg">"#,
            self.width + 20 * 2, self.height + 20 * 2
        );

        svg.push_str(&format!(
            r#"<rect width="100%" height="100%" fill="{}"/>"#,
            "#aaaaff"
        ));

        svg.push_str(&format!(
            r#"<rect x="{}" y="{}" width="{}" height="{}" fill="{}"/>"#,
            20, 20, self.width, self.height,
            theme.settings.background.unwrap().to_html())
        );

        let line_height = self.height as usize / v.len();

        for (i, line) in v.iter().enumerate() {
            let mut text = format!(
                r#"<text x="{}" y="{}" font-family="monospace" font-size="17px">"#,
                20, (i + 1) * line_height + 20
            );
            for (style, content) in line {
                let tspan = format!(
                    r#"<tspan fill="{fill}" font-style="{font_style}" font-weight="{font_weight}">{content}</tspan>"#,
                    fill = style.foreground.to_html(),
                    font_style = get_style(style.font_style),
                    font_weight = get_weight(style.font_style),
                    content = content.replace(' ', "&#160;"),
                );
                text.push_str(&tspan);
            }
            text.push_str("</text>");
            svg.push_str(&text);
        }

        svg.push_str("</svg>");
        svg
    }
}

#[cfg(test)]
mod tests {
    use crate::formatter::SVGFormatter;

    #[test]
    fn test() {
        use syntect::easy::HighlightLines;
        use syntect::parsing::SyntaxSet;
        use syntect::highlighting::ThemeSet;
        use syntect::util::LinesWithEndings;
        use syntect::dumps::from_binary;

        let code = r#"fn factorial(n: u64) -> u64 {
    match n {
        0 => 1,
        _ => n * factorial(n - 1),
    }
}

fn main() {
    println!("10! = {}", factorial(10));
}
"#;

        let ps = from_binary::<SyntaxSet>(include_bytes!("../../assets/syntaxes.bin"));
        let ts = from_binary::<ThemeSet>(include_bytes!("../../assets/themes.bin"));

        let syntax = ps.find_syntax_by_extension("rs").unwrap();
        let mut h = HighlightLines::new(syntax, &ts.themes["Dracula"]);

        let (width, height) = SVGFormatter::get_svg_size(&code.split('\n').collect::<Vec<_>>());

        let highlight = LinesWithEndings::from(&code)
            .map(|line| h.highlight(line, &ps))
            .collect::<Vec<_>>();

        let x = (SVGFormatter { width, height }).format(&highlight, &ts.themes["Dracula"]);

        std::fs::write("test.svg", x).unwrap();
    }
}
