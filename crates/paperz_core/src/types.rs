use hdk::prelude::*;

#[hdk_entry]
pub struct Paper {
    // must include extension
    pub filename: String,
    // encoded file bytes payload
    pub blob_str: String,
}

#[hdk_entry]
pub struct Annotation {
    pub paper_ref: EntryHash, // this should probably be a HeaderHash
    pub page_num: u64,
    pub paragraph_num: u64,
    pub what_it_says: String,
    pub what_it_should_say: String,
}
