use anyhow::Result;
use wasmtime::Store;

wasmtime::component::bindgen!(in "tests/runtime/smoke");

#[derive(Default)]
pub struct MyImports {
    hit: bool,
}

impl test::smoke::imports::Host for MyImports {
    fn thunk(&mut self) {
        self.hit = true;
        println!("in the host");
    }
}

#[test]
fn run() -> Result<()> {
    crate::run_test(
        "smoke",
        |linker| Smoke::add_to_linker(linker, |x| &mut x.0),
        |store, component, linker| Smoke::instantiate(store, component, linker),
        run_test,
    )
}

fn run_test(exports: Smoke, store: &mut Store<crate::Wasi<MyImports>>) -> Result<()> {
    exports.call_thunk(&mut *store)?;

    assert!(store.data().0.hit);

    Ok(())
}
