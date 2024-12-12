use alkanes_runtime::{auth::AuthenticatedResponder, token::Token};
#[allow(unused_imports)]
use alkanes_runtime::{
    println,
    stdio::{stdout, Write},
};
use anyhow::{anyhow, Result};
use alkanes_runtime::{runtime::AlkaneResponder, storage::StoragePointer};
use alkanes_support::{id::AlkaneId, utils::{shift_or_err}};
use alkanes_support::{context::Context, parcel::AlkaneTransfer, response::CallResponse};
use metashrew_support::{utils::{consensus_decode}, compat::{to_arraybuffer_layout, to_passback_ptr}};
use metashrew_support::index_pointer::KeyValuePointer;
use protorune_support::{network::{to_address_str, NetworkParams, set_network},protostone::{Protostone}};
use ordinals::{Runestone, Artifact};
use bitcoin::{Script, OutPoint, Amount, TxOut, Transaction};
use bitcoin::hashes::{Hash};
use shorcell_support::{Payment};
use std::sync::Arc;

#[derive(Default)]
pub struct Shorcell(());

#[cfg(all(
    not(feature = "mainnet"),
    not(feature = "testnet"),
    not(feature = "luckycoin"),
    not(feature = "dogecoin"),
    not(feature = "bellscoin")
))]
pub fn configure_network() {
    set_network(NetworkParams {
        bech32_prefix: String::from("bcrt"),
        p2pkh_prefix: 0x64,
        p2sh_prefix: 0xc4,
    });
}
#[cfg(feature = "mainnet")]
pub fn configure_network() {
    set_network(NetworkParams {
        bech32_prefix: String::from("bc"),
        p2sh_prefix: 0x05,
        p2pkh_prefix: 0x00,
    });
}
#[cfg(feature = "testnet")]
pub fn configure_network() {
    set_network(NetworkParams {
        bech32_prefix: String::from("tb"),
        p2pkh_prefix: 0x6f,
        p2sh_prefix: 0xc4,
    });
}
#[cfg(feature = "luckycoin")]
pub fn configure_network() {
    set_network(NetworkParams {
        bech32_prefix: String::from("lky"),
        p2pkh_hash: 0x2f,
        p2sh_hash: 0x05,
    });
}

#[cfg(feature = "dogecoin")]
pub fn configure_network() {
    set_network(NetworkParams {
        bech32_prefix: String::from("dc"),
        p2pkh_hash: 0x1e,
        p2sh_hash: 0x16,
    });
}
#[cfg(feature = "bellscoin")]
pub fn configure_network() {
    set_network(NetworkParams {
        bech32_prefix: String::from("bel"),
        p2pkh_hash: 0x19,
        p2sh_hash: 0x1e,
    });
}

pub trait MintableToken: Token {
    fn mint(&self, context: &Context, value: u128) -> AlkaneTransfer {
        AlkaneTransfer {
            id: context.myself.clone(),
            value,
        }
    }
}

impl Token for Shorcell {
    fn name(&self) -> String {
        String::from("SUBFROST BTC")
    }
    fn symbol(&self) -> String {
        String::from("frBTC")
    }
}
impl MintableToken for Shorcell {}

impl AuthenticatedResponder for Shorcell {}

impl Shorcell {
  fn signer_pointer(&self) -> StoragePointer {
    StoragePointer::from_keyword("/signer")
  }
  fn signer(&self) -> Vec<u8> {
    self.signer_pointer().get().as_ref().clone()
  }
  fn set_signer(&self, context: &Context, _vout: u32) -> Result<()> {
    let vout = _vout as usize;
    let tx = consensus_decode::<Transaction>(&mut std::io::Cursor::new(self.transaction()))?;
    if let Some(Artifact::Runestone(ref runestone)) = Runestone::decipher(&tx) {
      let protostones = Protostone::from_runestone(runestone)?;
      let message = &protostones[(context.vout as usize) - tx.output.len() - 1];
      if message.edicts.len() != 0 {
        return Err(anyhow!("message cannot contain edicts, only a pointer"));
      }
      let pointer = message
        .pointer
        .ok_or("")
        .map_err(|_| anyhow!("no pointer in message"))?;
      if pointer as usize >= tx.output.len() {
        return Err(anyhow!("pointer cannot be a protomessage"));
      }
      if pointer as usize == vout {
        return Err(anyhow!("pointer cannot be equal to output spendable by synthetic"));
      }
      self.signer_pointer().set(Arc::new(tx.output[vout as usize].script_pubkey.as_bytes().to_vec()));
      Ok(())
    } else {
      Err(anyhow!("unexpected condition: execution occurred with no Protostone present"))
    }
  }
  fn observe_transaction(&self, tx: &Transaction) -> Result<()> {
    let mut ptr = StoragePointer::from_keyword("/seen/").select(&tx.compute_txid().as_byte_array().to_vec());
    if ptr.get().len() != 0 { 
      Err(anyhow!("transaction already processed"))
    } else {
      ptr.set_value::<u8>(0x01);
      Ok(())
    }
  }
  fn compute_output(&self, tx: &Transaction) -> u128 {
    let signer = self.signer();
    tx.output.iter().fold(0, |r: u128, v: &TxOut| -> u128 {
      if v.script_pubkey.as_bytes().to_vec() == signer {
        r + <u64 as Into<u128>>::into(v.value.to_sat())
      } else {
        r
      }
    })
  }
  fn burn_input(&self, context: &Context) -> Result<u64> {
    Ok(context.incoming_alkanes.0.iter().find(|v| context.myself == v.id).ok_or("").map_err(|_| anyhow!("must spend synthetics into message"))?.value.try_into()?)
  }
  fn burn(&self, context: &Context, vout: usize) -> Result<u64> {
    let tx = consensus_decode::<Transaction>(&mut std::io::Cursor::new(self.transaction()))?;

    if let Some(Artifact::Runestone(ref runestone)) = Runestone::decipher(&tx) {
      let protostones = Protostone::from_runestone(runestone)?;
      let message = &protostones[(context.vout as usize) - tx.output.len() - 1];
      if message.edicts.len() != 0 {
        return Err(anyhow!("message cannot contain edicts, only a pointer"));
      }
      let pointer = message
        .pointer
        .ok_or("")
        .map_err(|_| anyhow!("no pointer in message"))?;
      if pointer as usize >= tx.output.len() {
        return Err(anyhow!("pointer cannot be a protomessage"));
      }
      if pointer as usize == vout {
        return Err(anyhow!("pointer cannot be equal to output spendable by synthetic"));
      }
      let signer = self.signer();
      if signer != tx.output[vout].script_pubkey.as_bytes().to_vec() {
        return Err(anyhow!("signer pubkey must be targeted with supplementary output"));
      }
      let txid = tx.compute_txid();
      let value = self.burn_input(context)?;
      StoragePointer::from_keyword("/payments/byheight/").select_value(self.height()).append(Arc::<Vec<u8>>::new((Payment {
        output: TxOut {
          script_pubkey: tx.output[pointer as usize].script_pubkey.clone(),
          value: Amount::from_sat(value)
        },
        spendable: OutPoint {
          txid,
          vout: vout.try_into()?
        }
      }).serialize()?));
      Ok(value)
    } else {
      Err(anyhow!("execution triggered unexpectedly -- no protostone"))
    }
  }
  fn exchange(&self, context: &Context) -> Result<AlkaneTransfer> {
    let tx = consensus_decode::<Transaction>(&mut std::io::Cursor::new(self.transaction()))?;
    self.observe_transaction(&tx)?;
    let payout = self.compute_output(&tx);
    Ok(self.mint(&context, payout))
  }
}

impl AlkaneResponder for Shorcell {
    fn execute(&self) -> Result<CallResponse> {
        configure_network();
        let context = self.context()?;
        let mut inputs = context.inputs.clone();
        let mut response: CallResponse = CallResponse::forward(&context.incoming_alkanes.clone());
        match shift_or_err(&mut inputs)? {
            /* initialize(u128, u128) */
            0 => {
                let mut pointer = StoragePointer::from_keyword("/initialized");
                if pointer.get().len() == 0 {
                    let auth_token_units = shift_or_err(&mut inputs)?;
                    response
                        .alkanes
                        .0
                        .push(self.deploy_auth_token(auth_token_units)?);
                    pointer.set(Arc::new(vec![0x01]));
                    Ok(response)
                } else {
                    return Err(anyhow!("already initialized"));
                }
            },
            1 => {
                self.only_owner()?;
                self.set_signer(&context, shift_or_err(&mut inputs)?.try_into()?)?;
                response.data = self.signer();
                Ok(response)
            }
            /* get_signer() -> String */
            100001 => {
              response.data = to_address_str(Script::from_bytes(self.signer_pointer().get().as_ref())).ok_or("").map_err(|_| anyhow!("invalid script"))?.as_bytes().to_vec();
              Ok(response)
            }
            /* mint(u128) */
            77 => {
                response.alkanes.0.push(self.exchange(&context)?);
                Ok(response)
            }
            78 => {
                if context.caller.clone() != (AlkaneId { tx: 0, block: 0 }) {
                  return Err(anyhow!("must be called by EOA"));
                }
                if context.incoming_alkanes.0.len() != 1 {
                  return Err(anyhow!("must only send synthetics as input alkanes"));
                }
                let burn_value = self.burn(&context, shift_or_err(&mut inputs)?.try_into()?)?;
                let mut burn_response = CallResponse::default();
                burn_response.data = burn_value.to_le_bytes().to_vec();
                Ok(burn_response)
            }
            /* name() */
            99 => {
                response.data = self.name().into_bytes().to_vec();
                Ok(response)
            }
            /* symbol() */
            100 => {
                response.data = self.symbol().into_bytes().to_vec();
                Ok(response)
            }
            /* payments_at_height */
            1001 => {
                let mut payments = CallResponse::forward(&context.incoming_alkanes);
                payments.data = StoragePointer::from_keyword("/payments/byheight/").select_value(self.height()).get_list().into_iter().fold(Vec::<u8>::new(), |r, v| {
                  let mut result = Vec::<u8>::with_capacity(r.len() + v.len());
                  result.extend(&r);
                  result.extend(v.as_ref());
                  result
                });
                Ok(payments)
            }
            _ => {
                panic!("unrecognized opcode");
            }
        }
    }
}

#[no_mangle]
pub extern "C" fn __execute() -> i32 {
    let mut response = to_arraybuffer_layout(&Shorcell::default().run());
    to_passback_ptr(&mut response)
}
