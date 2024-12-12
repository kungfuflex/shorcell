use anyhow::{Result};
use metashrew_support::utils::{is_empty, consensus_encode, consensus_decode};
use bitcoin::{OutPoint, TxOut};
use std::io::{Cursor};

#[derive(Debug, Clone)]
pub struct Payment {
  pub spendable: OutPoint,
  pub output: TxOut,
}

impl Payment {
  pub fn serialize(&self) -> Result<Vec<u8>> {
    let mut result: Vec<u8> = vec![];
    let spendable: Vec<u8> = consensus_encode::<OutPoint>(&self.spendable)?;
    let output: Vec<u8> = consensus_encode::<TxOut>(&self.output)?;
    result.extend(&spendable);
    result.extend(&output);
    Ok(result)
  }
}

pub fn deserialize_payments(v: &Vec<u8>) -> Result<Vec<Payment>> {
  let mut payments: Vec<Payment> = vec![];
  let mut cursor: Cursor<Vec<u8>> = Cursor::new(v.clone());
  while !is_empty(&mut cursor) {
    let (spendable, output) = (consensus_decode::<OutPoint>(&mut cursor)?, consensus_decode::<TxOut>(&mut cursor)?);
    payments.push(Payment {
      spendable,
      output
    });
  }
  Ok(payments)
}
