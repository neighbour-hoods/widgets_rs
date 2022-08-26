use hdk::prelude::*;

#[hdk_entry]
#[derive(Clone)]
pub struct Meme {
    // must include extension
    pub filename: String,
    // encoded file bytes payload
    pub blob_str: String,
}
