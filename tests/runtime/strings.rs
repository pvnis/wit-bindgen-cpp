use anyhow::Result;
use wasmtime::Store;

wasmtime::component::bindgen!(in "tests/runtime/strings");

#[derive(Default)]
pub struct MyImports;

impl test::strings::imports::Host for MyImports {
    fn take_basic(&mut self, s: String) {
        assert_eq!(s, "latin utf16");
    }

    fn return_unicode(&mut self) -> String {
        "🚀🚀🚀 𠈄𓀀".to_string()
    }
}

#[test]
fn run() -> Result<()> {
    crate::run_test(
        "strings",
        |linker| Strings::add_to_linker(linker, |x| &mut x.0),
        |store, component, linker| Strings::instantiate(store, component, linker),
        run_test,
    )
}

fn run_test(exports: Strings, store: &mut Store<crate::Wasi<MyImports>>) -> Result<()> {
    exports.call_test_imports(&mut *store)?;
    assert_eq!(exports.call_return_empty(&mut *store)?, "");
    assert_eq!(exports.call_roundtrip(&mut *store, "str")?, "str");
    assert_eq!(
        exports.call_roundtrip(&mut *store, "🚀🚀🚀 𠈄𓀀")?,
        "🚀🚀🚀 𠈄𓀀"
    );
    Ok(())
}
