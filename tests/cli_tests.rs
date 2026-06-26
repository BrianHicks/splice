use std::env::current_dir;

#[test]
fn cli_tests() {
    trycmd::TestCases::new()
        .env("NO_COLOR", "true")
        .env("LOG_LEVEL", "info")
        .env(
            "SPLICE_CACHE_ROOT",
            current_dir()
                .unwrap()
                .join("tests")
                .join("cache")
                .to_string_lossy(),
        )
        .case("tests/cmd/*.md")
        .case("README.md");
}
