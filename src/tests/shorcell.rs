use alkanes::message::AlkaneMessageContext;
use alkanes_support::id::AlkaneId;
use anyhow::Result;
use bitcoin::address::{NetworkChecked};
use bitcoin::{OutPoint, Txid, WitnessVersion, WitnessProgram, Witness, Sequence,  Amount, ScriptBuf, Script, Address, TxIn, TxOut, Transaction};
use bitcoin::hashes::{Hash};
use protorune_support::protostone::Protostone;
use protorune::protostone::Protostones;
#[allow(unused_imports)]
use hex;
use metashrew_support::index_pointer::KeyValuePointer;
#[allow(unused_imports)]
use metashrew_support::{utils::{format_key}};
use protorune::{test_helpers::{get_address}, balance_sheet::load_sheet, message::MessageContext, tables::RuneTable};
use shorcell_support::{address::Payload, constants::SHORCELL_FACTORY_ID};

use protorune_support::utils::consensus_encode;

use alkanes::indexer::index_block;
use alkanes_support::envelope::{RawEnvelope};
use ordinals::{Runestone, Artifact};
use alkanes::tests::helpers as alkane_helpers;
use alkanes::precompiled::{alkanes_std_auth_token_build};
use alkanes_support::{cellpack::Cellpack, constants::AUTH_TOKEN_FACTORY_ID};
use crate::tests::std::shorcell_contract_build;
#[allow(unused_imports)]
use metashrew::{get_cache, index_pointer::IndexPointer, println, stdio::stdout};
use alkane_helpers::{clear};
use std::fmt::Write;
use wasm_bindgen_test::wasm_bindgen_test;
use sphincsplus::{keypair, sign};

pub const ADDRESS1: &'static str = "bcrt1qzr9vhs60g6qlmk7x3dd7g3ja30wyts48sxuemv";

fn lock(inputs: Vec<OutPoint>, keys: &sphincsplus::Keypair) -> Transaction {
  let protostone: Protostone = Protostone {
    burn: None,
    edicts: vec![],
    pointer: Some(0),
    refund: Some(2),
    from: None,
    protocol_tag: AlkaneMessageContext::protocol_tag(),
    message: (Cellpack {
      target: AlkaneId {
        block: 6,
        tx: SHORCELL_FACTORY_ID
      },
      inputs: vec![0, 0]
    }).encipher(),
  };
  let runestone: ScriptBuf = (Runestone {
    etching: None,
    pointer: Some(0), // points to the OP_RETURN, so therefore targets the protoburn
    edicts: Vec::new(),
    mint: None,
    protocol: vec![protostone].encipher().ok(),
  }).encipher();
  let op_return = TxOut {
    value: Amount::from_sat(0),
    script_pubkey: runestone,
  };
  let address: Address<NetworkChecked> = get_address(ADDRESS1);
  let _script_pubkey = address.script_pubkey();
  Transaction {
    version: bitcoin::blockdata::transaction::Version::ONE,
    lock_time: bitcoin::absolute::LockTime::ZERO,
    input: inputs.into_iter().map(|v| TxIn {
      previous_output: v,
      witness: Witness::new(),
      script_sig: ScriptBuf::new(),
      sequence: Sequence::MAX
    }).collect::<Vec<TxIn>>(),
    output: vec![
      TxOut {
        value: Amount::from_sat(546),
        script_pubkey: Payload::WitnessProgram(WitnessProgram::new(WitnessVersion::V1, &keys.public).unwrap()).script_pubkey()
      },
      op_return
    ]
  }
}

fn burn(keys: &sphincsplus::Keypair) -> Transaction {
  let protostone: Protostone = Protostone {
    burn: None,
    edicts: vec![],
    pointer: Some(0),
    refund: Some(2),
    from: None,
    protocol_tag: AlkaneMessageContext::protocol_tag(),
    message: (Cellpack {
      target: AlkaneId {
        block: 4,
        tx: SHORCELL_FACTORY_ID
      },
      inputs: vec![78]
    }).encipher(),
  };
  let runestone: ScriptBuf = (Runestone {
    etching: None,
    pointer: Some(0), // points to the OP_RETURN, so therefore targets the protoburn
    edicts: Vec::new(),
    mint: None,
    protocol: vec![protostone].encipher().ok(),
  }).encipher();
  let op_return = TxOut {
    value: Amount::from_sat(0),
    script_pubkey: runestone,
  };
  let address: Address<NetworkChecked> = get_address(ADDRESS1);
  let _script_pubkey = address.script_pubkey();
  let mut tx = Transaction {
    version: bitcoin::blockdata::transaction::Version::ONE,
    lock_time: bitcoin::absolute::LockTime::ZERO,
    input: vec![TxIn {
      previous_output: OutPoint {
        txid: Txid::all_zeros(),
        vout: 0
      },
      witness: Witness::new(),
      script_sig: ScriptBuf::new(),
      sequence: Sequence::MAX
    }],
    output: vec![
      TxOut {
        value: Amount::from_sat(546),
        script_pubkey: get_address(ADDRESS1).script_pubkey()
      },
      op_return
    ]
  };
  let sig = sphincsplus::sign(&consensus_encode::<Transaction>(&tx).unwrap(), keys);
  tx.input[0].witness = RawEnvelope::from(sig.to_vec()).to_gzipped_witness();
  tx
}

#[wasm_bindgen_test]
fn test_shorcell() -> Result<()> {
    clear();
    let mut block_height = 850_000;
    let cellpacks: Vec<Cellpack> = [
        //auth token factory init
        Cellpack {
            target: AlkaneId { block: 3, tx: AUTH_TOKEN_FACTORY_ID },
            inputs: vec![100]
        }
    ]
    .into();
    let mut test_block = alkane_helpers::init_with_multiple_cellpacks_with_tx(
        [shorcell_contract_build::get_bytes()].into(),
        cellpacks,
    );
    let keys = sphincsplus::keypair();
    test_block.txdata.push(lock(vec![], &keys));
    test_block.txdata.push(burn(&keys));
    let len = test_block.txdata.len();
    let outpoint = OutPoint {
        txid: test_block.txdata[len - 1].compute_txid(),
        vout: 0
    };
    index_block(&test_block, block_height)?;
    let ptr = RuneTable::for_protocol(AlkaneMessageContext::protocol_tag())
        .OUTPOINT_TO_RUNES
        .select(&consensus_encode(&outpoint)?);
    let sheet = load_sheet(&ptr);
    println!("balances at end {:?}", sheet);
    Ok(())
}
