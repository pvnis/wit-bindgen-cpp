use anyhow::Result;
use wasmtime::Store;

wasmtime::component::bindgen!(in "tests/runtime/variants");

use test::variants::test as test_imports;

#[derive(Default)]
pub struct MyImports;

impl test_imports::Host for MyImports {
    fn roundtrip_option(&mut self, a: Option<f32>) -> Option<u8> {
        a.map(|x| x as u8)
    }

    fn roundtrip_result(&mut self, a: Result<u32, f32>) -> Result<f64, u8> {
        match a {
            Ok(a) => Ok(a.into()),
            Err(b) => Err(b as u8),
        }
    }

    fn roundtrip_enum(&mut self, a: test_imports::E1) -> test_imports::E1 {
        assert_eq!(a, a);
        a
    }

    fn invert_bool(&mut self, a: bool) -> bool {
        !a
    }

    fn variant_casts(&mut self, a: test_imports::Casts) -> test_imports::Casts {
        a
    }

    fn variant_zeros(&mut self, a: test_imports::Zeros) -> test_imports::Zeros {
        a
    }

    fn variant_typedefs(&mut self, _: Option<u32>, _: bool, _: Result<u32, ()>) {}

    fn variant_enums(
        &mut self,
        a: bool,
        b: Result<(), ()>,
        c: test_imports::MyErrno,
    ) -> (bool, Result<(), ()>, test_imports::MyErrno) {
        assert_eq!(a, true);
        assert_eq!(b, Ok(()));
        assert_eq!(c, test_imports::MyErrno::Success);
        (false, Err(()), test_imports::MyErrno::A)
    }
}

#[test]
fn run() -> Result<()> {
    crate::run_test(
        "variants",
        |linker| Variants::add_to_linker(linker, |x| &mut x.0),
        |store, component, linker| Variants::instantiate(store, component, linker),
        run_test,
    )
}

fn run_test(exports: Variants, store: &mut Store<crate::Wasi<MyImports>>) -> Result<()> {
    use exports::test::variants::test::*;

    exports.call_test_imports(&mut *store)?;
    let exports = exports.test_variants_test();

    assert_eq!(
        exports.call_roundtrip_option(&mut *store, Some(1.0))?,
        Some(1)
    );
    assert_eq!(exports.call_roundtrip_option(&mut *store, None)?, None);
    assert_eq!(
        exports.call_roundtrip_option(&mut *store, Some(2.0))?,
        Some(2)
    );
    assert_eq!(exports.call_roundtrip_result(&mut *store, Ok(2))?, Ok(2.0));
    assert_eq!(exports.call_roundtrip_result(&mut *store, Ok(4))?, Ok(4.0));
    assert_eq!(
        exports.call_roundtrip_result(&mut *store, Err(5.3))?,
        Err(5)
    );

    assert_eq!(exports.call_roundtrip_enum(&mut *store, E1::A)?, E1::A);
    assert_eq!(exports.call_roundtrip_enum(&mut *store, E1::B)?, E1::B);

    assert_eq!(exports.call_invert_bool(&mut *store, true)?, false);
    assert_eq!(exports.call_invert_bool(&mut *store, false)?, true);

    let (a1, a2, a3, a4, a5, a6) = exports.call_variant_casts(
        &mut *store,
        (C1::A(1), C2::A(2), C3::A(3), C4::A(4), C5::A(5), C6::A(6.0)),
    )?;
    assert!(matches!(a1, C1::A(1)));
    assert!(matches!(a2, C2::A(2)));
    assert!(matches!(a3, C3::A(3)));
    assert!(matches!(a4, C4::A(4)));
    assert!(matches!(a5, C5::A(5)));
    assert!(matches!(a6, C6::A(b) if b == 6.0));

    let (a1, a2, a3, a4, a5, a6) = exports.call_variant_casts(
        &mut *store,
        (
            C1::B(1),
            C2::B(2.0),
            C3::B(3.0),
            C4::B(4.0),
            C5::B(5.0),
            C6::B(6.0),
        ),
    )?;
    assert!(matches!(a1, C1::B(1)));
    assert!(matches!(a2, C2::B(b) if b == 2.0));
    assert!(matches!(a3, C3::B(b) if b == 3.0));
    assert!(matches!(a4, C4::B(b) if b == 4.0));
    assert!(matches!(a5, C5::B(b) if b == 5.0));
    assert!(matches!(a6, C6::B(b) if b == 6.0));

    let (a1, a2, a3, a4) =
        exports.call_variant_zeros(&mut *store, (Z1::A(1), Z2::A(2), Z3::A(3.0), Z4::A(4.0)))?;
    assert!(matches!(a1, Z1::A(1)));
    assert!(matches!(a2, Z2::A(2)));
    assert!(matches!(a3, Z3::A(b) if b == 3.0));
    assert!(matches!(a4, Z4::A(b) if b == 4.0));

    exports.call_variant_typedefs(&mut *store, None, false, Err(()))?;

    Ok(())
}
