use hdk::prelude::{holo_hash::DnaHash, *};

use common::{
    compose_entry_hash_path, compose_paths, create_sensemaker_entry_parse, get_latest_linked_entry,
    mk_application_se, remote_get_sensemaker_entry_by_path,
    remote_get_sensemaker_entry_by_path_with_hh, remote_initialize_sm_data,
    remote_set_sensemaker_entry_parse_rl_expr, remote_step_sm, remote_step_sm_path,
    sensemaker_cell_id_anchor, sensemaker_cell_id_fns, util, CreateSensemakerEntryInputParse,
    SensemakerCellId, SensemakerEntry,
};
use rep_lang_runtime::eval::{FlatValue, Value};
use social_sensemaker_core::{OWNER_TAG, SM_COMP_TAG, SM_DATA_TAG, SM_INIT_TAG};

use memez_core::{types::Meme, MEMEZ_PATH, MEME_TAG};
use paperz_core::AGENT_PATH;

entry_defs![
    Meme::entry_def(),
    SensemakerCellId::entry_def(),
    PathEntry::entry_def(),
    SensemakerEntry::entry_def()
];

sensemaker_cell_id_fns! {}

fn meme_anchor() -> ExternResult<EntryHash> {
    anchor("memez".into(), "".into())
}

#[hdk_extern]
fn upload_meme(meme: Meme) -> ExternResult<(EntryHash, HeaderHash)> {
    debug!(
        "upload_meme: received input of length {}",
        meme.blob_str.len()
    );

    let meme_hh = create_entry(&meme)?;
    let meme_eh = hash_entry(&meme)?;
    create_link(
        meme_anchor()?,
        meme_eh.clone(),
        LinkType(0),
        LinkTag::new(MEME_TAG),
    )?;

    // init SM data for meme
    // this requires the SM_INIT to have been already set...
    let cell_id = get_sensemaker_cell_id(())?;
    let payload = (MEMEZ_PATH.to_string(), meme_eh.clone());
    remote_initialize_sm_data(cell_id, None, payload)?;

    Ok((meme_eh, meme_hh))
}

#[hdk_extern]
fn clap_for_meme(meme_eh: EntryHash) -> ExternResult<()> {
    step_sm_remote((MEMEZ_PATH.into(), meme_eh, "1".into()))
}

#[hdk_extern]
fn meme_clap_count(meme_eh: EntryHash) -> ExternResult<Option<i64>> {
    let opt_eh_hh_se = get_sm_data(meme_eh)?;
    Ok(
        opt_eh_hh_se.and_then(|(_eh, _hh, se)| match se.output_flat_value {
            FlatValue(Value::VInt(x)) => Some(x),
            _ => None,
        }),
    )
}

#[hdk_extern]
fn get_all_memez(
    (feed_score_comp, agent_pk): (String, AgentPubKey),
) -> ExternResult<Vec<(EntryHash, Meme, i64)>> {
    let meme_entry_links = get_links(meme_anchor()?, Some(LinkTag::new(MEME_TAG)))?;
    let mut memez: Vec<(EntryHash, Meme, i64)> = Vec::new();
    let mut opt_err = None;
    for lnk in meme_entry_links {
        let res: ExternResult<(EntryHash, Meme, i64)> = {
            let meme_eh = lnk.target.into_entry_hash().expect("should be an Entry.");
            let (meme, _hh) =
                util::try_get_and_convert_with_hh(meme_eh.clone(), GetOptions::content())?;
            let meme_score_eh_hh_se = match get_sm_data(meme_eh.clone())? {
                Some(x) => x,
                None => panic!("impossible"),
            };
            let agent_score_eh_hh_se = match get_paperz_sm_data(agent_pk.clone())? {
                Some(x) => x,
                None => panic!("impossible"),
            };
            let score_comp_hh_se =
                create_sensemaker_entry_parse(CreateSensemakerEntryInputParse {
                    expr: feed_score_comp.clone(),
                    args: vec![],
                })?;
            let application_se = mk_application_se(vec![
                score_comp_hh_se.0,
                meme_score_eh_hh_se.1,
                agent_score_eh_hh_se.1,
            ])?;

            let meme_score = match application_se.output_flat_value {
                FlatValue(Value::VInt(x)) => {
                    debug!("x: {}", x);
                    x
                }
                _ => {
                    debug!("score is None!");
                    0
                }
            };
            Ok((meme_eh, meme, meme_score))
        };

        match res {
            Ok(tup) => memez.push(tup),
            Err(err) => {
                debug!("err in fetching meme: {}", err);
                opt_err = Some(err);
            }
        }
    }
    match opt_err {
        None => Ok(memez),
        Some(err) => Err(WasmError::Guest(format!("get_all_memez: {:?}", err))),
    }
}

fn get_paperz_sm_data(
    agent_pk: AgentPubKey,
) -> ExternResult<Option<(EntryHash, HeaderHash, SensemakerEntry)>> {
    let agent_b64: String = base64::encode(agent_pk.clone().into_inner());
    let path_string = compose_paths(&AGENT_PATH.into(), &agent_b64);
    get_sm_generic_with_hh(path_string, SM_DATA_TAG.to_string())
}

#[hdk_extern]
fn get_sm_data(
    target_eh: EntryHash,
) -> ExternResult<Option<(EntryHash, HeaderHash, SensemakerEntry)>> {
    let path_string = compose_entry_hash_path(&MEMEZ_PATH.into(), target_eh);
    get_sm_generic_with_hh(path_string, SM_DATA_TAG.to_string())
}

#[hdk_extern]
fn get_sm_init(path_string: String) -> ExternResult<Option<(EntryHash, SensemakerEntry)>> {
    get_sm_generic(path_string, SM_INIT_TAG.into())
}

#[hdk_extern]
fn get_sm_comp(path_string: String) -> ExternResult<Option<(EntryHash, SensemakerEntry)>> {
    get_sm_generic(path_string, SM_COMP_TAG.into())
}

fn get_sm_generic(
    path_string: String,
    link_tag_string: String,
) -> ExternResult<Option<(EntryHash, SensemakerEntry)>> {
    let cell_id = get_sensemaker_cell_id(())?;
    remote_get_sensemaker_entry_by_path(cell_id, None, (path_string, link_tag_string))
}

fn get_sm_generic_with_hh(
    path_string: String,
    link_tag_string: String,
) -> ExternResult<Option<(EntryHash, HeaderHash, SensemakerEntry)>> {
    let cell_id = get_sensemaker_cell_id(())?;
    remote_get_sensemaker_entry_by_path_with_hh(cell_id, None, (path_string, link_tag_string))
}

#[hdk_extern]
/// set the sm_init state for the path_string to the `rep_lang` interpretation of `expr_str`
pub fn set_sm_init((path_string, expr_str): (String, String)) -> ExternResult<bool> {
    set_sensemaker_entry(path_string, SM_INIT_TAG.into(), expr_str)
}

#[hdk_extern]
/// set the sm_comp state for the path_string to the `rep_lang` interpretation of `expr_str`
pub fn set_sm_comp((path_string, expr_str): (String, String)) -> ExternResult<bool> {
    set_sensemaker_entry(path_string, SM_COMP_TAG.into(), expr_str)
}

fn set_sensemaker_entry(
    path_string: String,
    link_tag_string: String,
    expr_str: String,
) -> ExternResult<bool> {
    let cell_id = get_sensemaker_cell_id(())?;
    remote_set_sensemaker_entry_parse_rl_expr(
        cell_id,
        None,
        (path_string, link_tag_string, expr_str),
    )?;
    Ok(true)
}

#[hdk_extern]
fn step_sm_remote((path_string, entry_hash, act): (String, EntryHash, String)) -> ExternResult<()> {
    let cell_id = get_sensemaker_cell_id(())?;
    remote_step_sm(cell_id, None, (path_string, entry_hash, act))
}

// TODO figure out how to automate / streamline all these high-indirection methods
#[hdk_extern]
fn step_sm_path_remote(payload: (String, String, String)) -> ExternResult<()> {
    let cell_id = get_sensemaker_cell_id(())?;
    remote_step_sm_path(cell_id, None, payload)
}
