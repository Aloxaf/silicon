use syntect::dumps::dump_to_file;
use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSet;

fn create_theme_dump() {
    let mut default = ThemeSet::load_defaults();
    let from_folder = ThemeSet::load_from_folder("./assets/themes").expect("failed to load themes");

    default.themes.extend(from_folder.themes.into_iter());

    dump_to_file(&default, "./assets/themes.bin").expect("failed to dump");
}

fn create_syntax_dump() {
    let mut builder = SyntaxSet::load_defaults_newlines().into_builder();
    builder.add_plain_text_syntax();

    builder
        .add_from_folder("./assets/syntaxes", true)
        .expect("failed to load syntaxes");

    let default = builder.build();

    dump_to_file(&default, "./assets/syntaxes.bin").expect("failed to dump");
}

fn main() {
    create_theme_dump();
    create_syntax_dump();

    cc::Build::new()
        .file("src/gauss/gauss.c")
        .cpp(false)
        .flag("-march=native")
        .opt_level(2)
        .compile("gauss");
}
