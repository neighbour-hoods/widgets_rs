use wasm_bindgen::prelude::*;

#[wasm_bindgen(module = "/src/myclass.js")]
extern "C" {
    pub fn name() -> String;

    pub type MyClass;

    #[wasm_bindgen(constructor)]
    pub fn new() -> MyClass;
    #[wasm_bindgen(method, getter)]
    pub fn number(this: &MyClass) -> u32;
    #[wasm_bindgen(method, setter)]
    pub fn set_number(this: &MyClass, number: u32) -> MyClass;
    #[wasm_bindgen(method)]
    pub fn render(this: &MyClass) -> String;
}
