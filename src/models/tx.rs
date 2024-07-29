use crate::models::tx_type::TxType;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};

pub(crate) type ClientId = u16;
pub(crate) type TxId = u32;

#[derive(Serialize, Deserialize, Debug)]
pub struct TxInput {
    #[serde(rename = "type")]
    pub(crate) tx_type: TxType,

    #[serde(rename = "client")]
    pub(crate) client_id: ClientId,

    #[serde(rename = "tx")]
    pub(crate) id: TxId,

    pub(crate) amount: Option<Decimal>,
}

#[derive(Debug, PartialEq)]
pub(crate) struct Tx {
    //pub(crate) id: TxId, //unused
    pub(crate) client_id: ClientId,
    pub(crate) tx_type: TxType,
    pub(crate) amount: Decimal,
    pub(crate) under_dispute: bool,
}

impl Tx {
    pub(crate) fn new(tx_input: &TxInput) -> Self {
        Self {
            client_id: tx_input.client_id,
            tx_type: tx_input.tx_type,
            amount: tx_input.amount.unwrap_or_else(|| dec!(0)),
            under_dispute: false,
        }
    }
}
