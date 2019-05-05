#[cfg(not(feature = "wasm_bindgen_test"))]
extern crate noop_attr;
#[cfg(feature = "wasm_bindgen_test")]
extern crate wasm_bindgen_test;
#[cfg(not(feature = "wasm_bindgen_test"))]
pub use self::noop_attr::noop as test;
#[cfg(feature = "wasm_bindgen_test")]
pub use self::wasm_bindgen_test::wasm_bindgen_test as test;
#[cfg(feature = "wasm_bindgen_test")]
self::wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);
