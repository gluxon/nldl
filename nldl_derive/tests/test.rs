#[test]
fn ui() {
    let t = trybuild::TestCases::new();
    t.pass("tests/ui/**/*-passes.rs");
    t.compile_fail("tests/ui/**/*-fails.rs");
}
