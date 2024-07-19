use anyhow::Result;
use wasmtime::Store;

wasmtime::component::bindgen!(in "tests/runtime/flavorful");

use exports::test::flavorful::test::*;
use test::flavorful::test as test_imports;

#[derive(Default)]
pub struct MyImports {
    errored: bool,
}

impl test_imports::Host for MyImports {
    fn f_list_in_record1(&mut self, ty: test_imports::ListInRecord1) {
        assert_eq!(ty.a, "list_in_record1");
    }

    fn f_list_in_record2(&mut self) -> test_imports::ListInRecord2 {
        test_imports::ListInRecord2 {
            a: "list_in_record2".to_string(),
        }
    }

    fn f_list_in_record3(&mut self, a: test_imports::ListInRecord3) -> test_imports::ListInRecord3 {
        assert_eq!(a.a, "list_in_record3 input");
        test_imports::ListInRecord3 {
            a: "list_in_record3 output".to_string(),
        }
    }

    fn f_list_in_record4(&mut self, a: test_imports::ListInAlias) -> test_imports::ListInAlias {
        assert_eq!(a.a, "input4");
        test_imports::ListInRecord4 {
            a: "result4".to_string(),
        }
    }

    fn f_list_in_variant1(
        &mut self,
        a: test_imports::ListInVariant1V1,
        b: test_imports::ListInVariant1V2,
    ) {
        assert_eq!(a.unwrap(), "foo");
        assert_eq!(b.unwrap_err(), "bar");
    }

    fn f_list_in_variant2(&mut self) -> Option<String> {
        Some("list_in_variant2".to_string())
    }

    fn f_list_in_variant3(&mut self, a: test_imports::ListInVariant3) -> Option<String> {
        assert_eq!(a.unwrap(), "input3");
        Some("output3".to_string())
    }

    fn errno_result(&mut self) -> Result<(), test_imports::MyErrno> {
        if self.errored {
            return Ok(());
        }
        test_imports::MyErrno::A.to_string();
        format!("{:?}", test_imports::MyErrno::A);
        fn assert_error<T: std::error::Error>() {}
        assert_error::<test_imports::MyErrno>();
        self.errored = true;
        Err(test_imports::MyErrno::B)
    }

    fn list_typedefs(
        &mut self,
        a: test_imports::ListTypedef,
        b: test_imports::ListTypedef3,
    ) -> (test_imports::ListTypedef2, test_imports::ListTypedef3) {
        assert_eq!(a, "typedef1");
        assert_eq!(b.len(), 1);
        assert_eq!(b[0], "typedef2");
        (b"typedef3".to_vec(), vec!["typedef4".to_string()])
    }

    fn list_of_variants(
        &mut self,
        bools: Vec<bool>,
        results: Vec<Result<(), ()>>,
        enums: Vec<test_imports::MyErrno>,
    ) -> (Vec<bool>, Vec<Result<(), ()>>, Vec<test_imports::MyErrno>) {
        assert_eq!(bools, [true, false]);
        assert_eq!(results, [Ok(()), Err(())]);
        assert_eq!(
            enums,
            [test_imports::MyErrno::Success, test_imports::MyErrno::A]
        );
        (
            vec![false, true],
            vec![Err(()), Ok(())],
            vec![test_imports::MyErrno::A, test_imports::MyErrno::B],
        )
    }
}

#[test]
fn run() -> Result<()> {
    crate::run_test(
        "flavorful",
        |linker| Flavorful::add_to_linker(linker, |x| &mut x.0),
        |store, component, linker| Flavorful::instantiate(store, component, linker),
        run_test,
    )
}

fn run_test(exports: Flavorful, store: &mut Store<crate::Wasi<MyImports>>) -> Result<()> {
    exports.call_test_imports(&mut *store)?;
    let exports = exports.test_flavorful_test();

    exports.call_f_list_in_record1(
        &mut *store,
        &ListInRecord1 {
            a: "list_in_record1".to_string(),
        },
    )?;
    assert_eq!(
        exports.call_f_list_in_record2(&mut *store)?.a,
        "list_in_record2"
    );

    assert_eq!(
        exports
            .call_f_list_in_record3(
                &mut *store,
                &ListInRecord3 {
                    a: "list_in_record3 input".to_string()
                }
            )?
            .a,
        "list_in_record3 output"
    );

    assert_eq!(
        exports
            .call_f_list_in_record4(
                &mut *store,
                &ListInAlias {
                    a: "input4".to_string()
                }
            )?
            .a,
        "result4"
    );

    exports.call_f_list_in_variant1(
        &mut *store,
        &Some("foo".to_string()),
        &Err("bar".to_string()),
    )?;
    assert_eq!(
        exports.call_f_list_in_variant2(&mut *store)?,
        Some("list_in_variant2".to_string())
    );
    assert_eq!(
        exports.call_f_list_in_variant3(&mut *store, &Some("input3".to_string()))?,
        Some("output3".to_string())
    );

    assert!(exports.call_errno_result(&mut *store)?.is_err());
    MyErrno::A.to_string();
    format!("{:?}", MyErrno::A);
    fn assert_error<T: std::error::Error>() {}
    assert_error::<MyErrno>();

    let (a, b) = exports.call_list_typedefs(
        &mut *store,
        &"typedef1".to_string(),
        &vec!["typedef2".to_string()],
    )?;
    assert_eq!(a, b"typedef3");
    assert_eq!(b.len(), 1);
    assert_eq!(b[0], "typedef4");
    Ok(())
}
