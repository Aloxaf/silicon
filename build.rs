fn main() {
    cc::Build::new()
        .file("src/gauss/gauss.c")
        .cpp(false)
        .flag("-march=native")
        .opt_level(2)
        .compile("gauss");
}
