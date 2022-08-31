use hdk::prelude::{holo_hash::DnaHash, *};

use common::{
    get_latest_linked_entry, sensemaker_cell_id_anchor, sensemaker_cell_id_fns, util,
    SensemakerCellId, SensemakerEntry,
};
use social_sensemaker_core::OWNER_TAG;

entry_defs![
    SensemakerCellId::entry_def(),
    PathEntry::entry_def(),
    SensemakerEntry::entry_def()
];

sensemaker_cell_id_fns! {}
