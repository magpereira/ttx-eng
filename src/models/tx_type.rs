use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq, Copy, Clone)]
#[serde(rename_all = "lowercase")]
pub(crate) enum TxType {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
}
