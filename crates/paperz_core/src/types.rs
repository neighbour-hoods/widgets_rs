use hdk::prelude::*;
use js_sys::{Object, Reflect};
use wasm_bindgen::{prelude::*, JsCast};

use holochain_client_wrapper::{DeserializeFromJsObj, EntryHashRaw, SerializeToJsObj};

pub type PaperEhVec = Vec<(EntryHashRaw, Paper)>;

#[hdk_entry]
#[derive(Clone)]
pub struct Paper {
    // must include extension
    pub filename: String,
    // encoded file bytes payload
    pub blob_str: String,
}

impl SerializeToJsObj for Paper {
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

impl DeserializeFromJsObj for Paper {
    fn deserialize_from_js_obj(v: JsValue) -> Self {
        let filename = String::deserialize_from_js_obj(
            Reflect::get(&v, &JsValue::from_str("filename")).expect("object field get to succeed"),
        );
        let blob_str = String::deserialize_from_js_obj(
            Reflect::get(&v, &JsValue::from_str("blob_str")).expect("object field get to succeed"),
        );
        Self { filename, blob_str }
    }
}

#[hdk_entry]
pub struct Annotation {
    pub paper_ref: EntryHash, // this should probably be a HeaderHash
    pub page_num: u64,
    pub paragraph_num: u64,
    pub what_it_says: String,
    pub what_it_should_say: String,
}
