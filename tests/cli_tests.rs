#[test]
fn cli_tests() {
    trycmd::TestCases::new()
        .env("NO_COLOR", "true")
        .env("LOG_LEVEL", "info")
        .case("tests/cmd/*.md")
        .case("README.md");
}
