#[test]
fn make_client() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/make_client/not_found.rs");
    t.compile_fail("tests/make_client/hello.rs");
    t.compile_fail("tests/make_client/pathed.rs");
    t.pass("tests/make_client/with_as.rs");
}
