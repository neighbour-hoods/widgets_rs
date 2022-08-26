use js_sys::{Array, Object, Reflect};
use wasm_bindgen::{prelude::*, JsCast};

use holochain_client_wrapper::{DeserializeFromJsObj, EntryHashRaw, SerializeToJsObj};
use memez_core::types::Meme;

pub struct Pair<A, B>(A, B);

impl<A, B> Into<(A, B)> for Pair<A, B> {
    fn into(self) -> (A, B) {
        let Pair(a, b) = self;
        (a, b)
    }
}

pub type MemeEhVec = Vec<Pair<EntryHashRaw, Meme>>;

pub trait SerializeToJsObj_ {
    fn serialize_to_js_obj(self) -> JsValue;
}

pub trait DeserializeFromJsObj_ {
    fn deserialize_from_js_obj_(_: JsValue) -> Self;
}

impl SerializeToJsObj_ for Meme {
    fn serialize_to_js_obj(self) -> JsValue {
        let ret = move || -> Result<JsValue, JsValue> {
            let val: JsValue = Object::new().dyn_into()?;
            assert!(Reflect::set(
                &val,
                &JsValue::from_str("filename"),
                &self.filename.serialize_to_js_obj(),
            )?);
            assert!(Reflect::set(
                &val,
                &JsValue::from_str("blob_str"),
                &self.blob_str.serialize_to_js_obj(),
            )?);
            Ok(val)
        };
        ret().expect("operations to succeed")
    }
}

impl<T: DeserializeFromJsObj_> DeserializeFromJsObj_ for Vec<T> {
    fn deserialize_from_js_obj_(v: JsValue) -> Self {
        let arr: Array = v.dyn_into().expect("Array conversion to succeed");
        let len = arr.length();
        let mut ret = Vec::new();
        for idx in 0..len {
            let ele = arr.get(idx);
            ret.push(T::deserialize_from_js_obj_(ele));
        }
        ret
    }
}

impl<A: DeserializeFromJsObj, B: DeserializeFromJsObj_> DeserializeFromJsObj_ for Pair<A, B> {
    fn deserialize_from_js_obj_(v: JsValue) -> Self {
        let arr: Array = v.dyn_into().expect("Array conversion to succeed");
        let a = arr.at(0);
        let b = arr.at(1);
        Pair(
            A::deserialize_from_js_obj(a),
            B::deserialize_from_js_obj_(b),
        )
    }
}

impl<A: DeserializeFromJsObj, B: DeserializeFromJsObj> DeserializeFromJsObj_ for (A, B) {
    fn deserialize_from_js_obj_(v: JsValue) -> Self {
        let arr: Array = v.dyn_into().expect("Array conversion to succeed");
        let a = arr.at(0);
        let b = arr.at(1);
        (A::deserialize_from_js_obj(a), B::deserialize_from_js_obj(b))
    }
}

impl DeserializeFromJsObj_ for Meme {
    fn deserialize_from_js_obj_(v: JsValue) -> Self {
        let filename = String::deserialize_from_js_obj(
            Reflect::get(&v, &JsValue::from_str("filename")).expect("object field get to succeed"),
        );
        let blob_str = String::deserialize_from_js_obj(
            Reflect::get(&v, &JsValue::from_str("blob_str")).expect("object field get to succeed"),
        );
        Self { filename, blob_str }
    }
}
