#[test]
fn cli_tests() {
    trycmd::TestCases::new()
        .env("NO_COLORS", "true")
        .env("RUST_LOG", "trace")
        .case("tests/cmd/*.md")
        .case("README.md");
}
