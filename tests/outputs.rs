use trycmd::TestCases;

#[test]
fn validate_outputs() {
    #[cfg(target_os = "linux")]
    TestCases::new()
        .case("tests/cmd/*.toml")
        .case("README.md")
        .run();

    #[cfg(not(target_os = "linux"))]
    TestCases::new().case("tests/cmd/*.toml").run();
}
