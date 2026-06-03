#[test]
fn cli_tests() {
    trycmd::TestCases::new()
        .env("NO_COLORS", "true")
        .case("tests/cmd/*.md")
        .case("README.md");
}
