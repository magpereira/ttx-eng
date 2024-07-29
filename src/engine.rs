use std::collections::HashMap;

use tracing::debug;

use crate::models::client::{Client, ClientReport};
use crate::models::errors::Error;
use crate::models::errors::Error::{
    ClientIdNoMatch, TxIdConflict, TxInvalidAmount, TxNotADeposit, TxNotFound, TxNotUnderDispute,
};
use crate::models::tx::{ClientId, Tx, TxId, TxInput};
use crate::models::tx_type::TxType;

pub struct Engine {
    clients: HashMap<ClientId, Client>,
    transactions: HashMap<TxId, Tx>,
}

impl Engine {
    pub(crate) fn new() -> Self {
        Self {
            clients: HashMap::new(),
            transactions: HashMap::new(),
        }
    }

    pub(crate) fn process_tx(&mut self, tx: &TxInput) {
        if let Err(err) = self.process_tx_inner(tx) {
            debug!("failed to process transaction {}: {}", tx.id, err)
        }
    }

    fn process_tx_inner(&mut self, tx_input: &TxInput) -> Result<(), Error> {
        let client = self
            .clients
            .entry(tx_input.client_id)
            .or_insert(Client::new(tx_input.client_id));

        match tx_input.tx_type {
            TxType::Deposit => {
                if self.transactions.contains_key(&tx_input.id) {
                    return Err(TxIdConflict);
                }

                let amount = match &tx_input.amount {
                    Some(val) => val,
                    None => return Err(TxInvalidAmount),
                };

                self.transactions.insert(tx_input.id, Tx::new(tx_input));
                client.deposit(amount)
            }
            TxType::Withdrawal => {
                if self.transactions.contains_key(&tx_input.id) {
                    return Err(TxIdConflict);
                }

                let amount = match &tx_input.amount {
                    Some(val) => val,
                    None => return Err(TxInvalidAmount),
                };

                self.transactions.insert(tx_input.id, Tx::new(tx_input));
                client.withdraw(amount)
            }
            TxType::Dispute => {
                let tx = self.transactions.get_mut(&tx_input.id);
                match tx {
                    None => Err(TxNotFound),
                    Some(tx) => {
                        if tx.client_id != tx_input.client_id {
                            return Err(ClientIdNoMatch);
                        }

                        if tx.tx_type != TxType::Deposit {
                            return Err(TxNotADeposit);
                        }

                        tx.under_dispute = true;
                        client.dispute(&tx.amount)
                    }
                }
            }
            TxType::Resolve => {
                let tx = self.transactions.get_mut(&tx_input.id);
                match tx {
                    None => Err(TxNotFound),
                    Some(tx) => {
                        if tx.client_id != tx_input.client_id {
                            return Err(ClientIdNoMatch);
                        }

                        if !tx.under_dispute {
                            return Err(TxNotUnderDispute);
                        }

                        tx.under_dispute = false;
                        client.resolve(&tx.amount)
                    }
                }
            }
            TxType::Chargeback => {
                let tx = self.transactions.get_mut(&tx_input.id);
                match tx {
                    None => Err(TxNotFound),
                    Some(tx) => {
                        if tx.client_id != tx_input.client_id {
                            return Err(ClientIdNoMatch);
                        }

                        if !tx.under_dispute {
                            return Err(TxNotUnderDispute);
                        }

                        client.chargeback(&tx.amount)
                    }
                }
            }
        }
    }

    pub(crate) fn report(&self) -> impl Iterator<Item = ClientReport> + '_ {
        return self.clients.values().map(ClientReport::new);
    }
}

#[cfg(test)]
mod tests {
    use rust_decimal_macros::dec;

    use super::*;

    // process_tx_inner
    #[test]
    fn process_tx_inner_success() -> Result<(), Error> {
        let txs = vec![
            TxInput {
                tx_type: TxType::Deposit,
                client_id: 0,
                id: 1,
                amount: Option::from(dec!(10)),
            },
            TxInput {
                tx_type: TxType::Withdrawal,
                client_id: 0,
                id: 2,
                amount: Option::from(dec!(1)),
            },
            TxInput {
                tx_type: TxType::Dispute,
                client_id: 0,
                id: 1,
                amount: None,
            },
            TxInput {
                tx_type: TxType::Resolve,
                client_id: 0,
                id: 1,
                amount: None,
            },
            TxInput {
                tx_type: TxType::Deposit,
                client_id: 0,
                id: 3,
                amount: Option::from(dec!(10)),
            },
            TxInput {
                tx_type: TxType::Dispute,
                client_id: 0,
                id: 3,
                amount: Option::from(dec!(10)),
            },
            TxInput {
                tx_type: TxType::Chargeback,
                client_id: 0,
                id: 3,
                amount: None,
            },
            TxInput {
                tx_type: TxType::Deposit,
                client_id: 1,
                id: 4,
                amount: Option::from(dec!(10)),
            },
            TxInput {
                tx_type: TxType::Dispute,
                client_id: 1,
                id: 4,
                amount: None,
            },
        ];
        let mut e = Engine::new();

        for t in txs.into_iter() {
            match e.process_tx_inner(&t) {
                Ok(_) => continue,
                Err(err) => {
                    return Err(err);
                }
            }
        }

        let c = e.clients.get(&0).expect("client not found");
        let c1 = e.clients.get(&1).expect("client not found");

        assert_ne!(*c, Client::new(0));
        assert_ne!(*c1, Client::new(1));
        assert_eq!(e.transactions.len(), 4);

        let tx1 = e.transactions.get(&1).expect("tx not found");
        assert_eq!(tx1.client_id, 0);
        assert_eq!(tx1.amount, dec!(10));
        assert_eq!(tx1.under_dispute, false);

        let tx2 = e.transactions.get(&2).expect("tx not found");
        assert_eq!(tx2.client_id, 0);
        assert_eq!(tx2.amount, dec!(1));
        assert_eq!(tx2.under_dispute, false);

        let tx3 = e.transactions.get(&3).expect("tx not found");
        assert_eq!(tx3.client_id, 0);
        assert_eq!(tx3.amount, dec!(10));
        assert_eq!(tx3.under_dispute, true);

        let tx4 = e.transactions.get(&4).expect("tx not found");
        assert_eq!(tx4.client_id, 1);
        assert_eq!(tx4.amount, dec!(10));
        assert_eq!(tx4.under_dispute, true);

        Ok(())
    }

    // process_tx_inner fail deposit
    #[test]
    fn process_tx_inner_fail_deposit() {
        let tx1 = TxInput {
            tx_type: TxType::Deposit,
            client_id: 0,
            id: 1,
            amount: Option::from(dec!(-1)),
        };
        let mut e = Engine::new();
        let result = e.process_tx_inner(&tx1);

        assert!(result.is_err())
    }

    #[test]
    fn process_tx_inner_fail_deposit_conflict() {
        let tx1 = TxInput {
            tx_type: TxType::Deposit,
            client_id: 0,
            id: 1,
            amount: Option::from(dec!(10)),
        };
        let tx2 = TxInput {
            tx_type: TxType::Deposit,
            client_id: 1,
            id: 1,
            amount: Option::from(dec!(20)),
        };
        let mut e = Engine::new();
        e.process_tx_inner(&tx1).expect("process tx failed");
        let result = e.process_tx_inner(&tx2);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), TxIdConflict)
    }

    #[test]
    fn process_tx_inner_fail_deposit_tx_invalid_amount() {
        let tx1 = TxInput {
            tx_type: TxType::Deposit,
            client_id: 0,
            id: 1,
            amount: None,
        };
        let mut e = Engine::new();
        let result = e.process_tx_inner(&tx1);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), TxInvalidAmount)
    }

    // process_tx_inner fail withdrawal
    #[test]
    fn process_tx_inner_fail_withdrawal() {
        let tx1 = TxInput {
            tx_type: TxType::Withdrawal,
            client_id: 0,
            id: 1,
            amount: Option::from(dec!(-1)),
        };
        let mut e = Engine::new();
        let result = e.process_tx_inner(&tx1);

        assert!(result.is_err())
    }

    #[test]
    fn process_tx_inner_fail_withdrawal_conflict() {
        let tx1 = TxInput {
            tx_type: TxType::Deposit,
            client_id: 0,
            id: 1,
            amount: Option::from(dec!(11)),
        };
        let tx2 = TxInput {
            tx_type: TxType::Withdrawal,
            client_id: 1,
            id: 1,
            amount: Option::from(dec!(20)),
        };

        let mut e = Engine::new();
        e.process_tx_inner(&tx1).expect("process tx failed");
        let result = e.process_tx_inner(&tx2);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), TxIdConflict)
    }

    #[test]
    fn process_tx_inner_fail_withdrawal_tx_invalid_amount() {
        let tx1 = TxInput {
            tx_type: TxType::Withdrawal,
            client_id: 0,
            id: 1,
            amount: None,
        };
        let mut e = Engine::new();
        let result = e.process_tx_inner(&tx1);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), TxInvalidAmount)
    }

    // process_tx_inner fail dispute
    #[test]
    fn process_tx_inner_fail_dispute() {
        let tx1 = TxInput {
            tx_type: TxType::Deposit,
            client_id: 0,
            id: 1,
            amount: Option::from(dec!(1)),
        };
        let tx2 = TxInput {
            tx_type: TxType::Dispute,
            client_id: 0,
            id: 1,
            amount: None,
        };
        let tx3 = TxInput {
            tx_type: TxType::Chargeback,
            client_id: 0,
            id: 1,
            amount: None,
        };
        let tx4 = TxInput {
            tx_type: TxType::Dispute,
            client_id: 0,
            id: 1,
            amount: None,
        };
        let mut e = Engine::new();
        e.process_tx_inner(&tx1).expect("process tx failed");
        e.process_tx_inner(&tx2).expect("process tx failed");
        e.process_tx_inner(&tx3).expect("process tx failed");
        let result = e.process_tx_inner(&tx4);

        assert!(result.is_err())
    }

    #[test]
    fn process_tx_inner_fail_dispute_tx_not_found() {
        let tx = TxInput {
            tx_type: TxType::Dispute,
            client_id: 0,
            id: 1,
            amount: None,
        };

        let mut e = Engine::new();
        let result = e.process_tx_inner(&tx);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), TxNotFound)
    }

    #[test]
    fn process_tx_inner_fail_dispute_client_id_no_match() {
        let tx1 = TxInput {
            tx_type: TxType::Deposit,
            client_id: 1,
            id: 1,
            amount: Option::from(dec!(0)),
        };
        let tx2 = TxInput {
            tx_type: TxType::Dispute,
            client_id: 0,
            id: 1,
            amount: None,
        };

        let mut e = Engine::new();
        e.process_tx_inner(&tx1).expect("process tx failed");
        let result = e.process_tx_inner(&tx2);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), ClientIdNoMatch)
    }

    #[test]
    fn process_tx_inner_fail_dispute_tx_not_a_deposit() {
        let tx1 = TxInput {
            tx_type: TxType::Withdrawal,
            client_id: 0,
            id: 1,
            amount: Option::from(dec!(0)),
        };
        let tx2 = TxInput {
            tx_type: TxType::Dispute,
            client_id: 0,
            id: 1,
            amount: None,
        };

        let mut e = Engine::new();
        e.process_tx_inner(&tx1).expect("process tx failed");
        let result = e.process_tx_inner(&tx2);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), TxNotADeposit)
    }

    // process_tx_inner fail resolve
    #[test]
    fn process_tx_inner_fail_resolve() {
        let tx1 = TxInput {
            tx_type: TxType::Deposit,
            client_id: 0,
            id: 1,
            amount: Option::from(dec!(1)),
        };
        let tx2 = TxInput {
            tx_type: TxType::Dispute,
            client_id: 0,
            id: 1,
            amount: None,
        };
        let tx3 = TxInput {
            tx_type: TxType::Chargeback,
            client_id: 0,
            id: 1,
            amount: None,
        };
        let tx4 = TxInput {
            tx_type: TxType::Resolve,
            client_id: 0,
            id: 1,
            amount: None,
        };
        let mut e = Engine::new();
        e.process_tx_inner(&tx1).expect("process tx failed");
        e.process_tx_inner(&tx2).expect("process tx failed");
        e.process_tx_inner(&tx3).expect("process tx failed");
        let result = e.process_tx_inner(&tx4);

        assert!(result.is_err())
    }

    #[test]
    fn process_tx_inner_fail_resolve_tx_not_found() {
        let tx = TxInput {
            tx_type: TxType::Resolve,
            client_id: 0,
            id: 1,
            amount: None,
        };

        let mut e = Engine::new();
        let result = e.process_tx_inner(&tx);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), TxNotFound)
    }

    #[test]
    fn process_tx_inner_fail_resolve_client_id_no_match() {
        let tx1 = TxInput {
            tx_type: TxType::Deposit,
            client_id: 1,
            id: 1,
            amount: Option::from(dec!(0)),
        };
        let tx2 = TxInput {
            tx_type: TxType::Resolve,
            client_id: 0,
            id: 1,
            amount: None,
        };

        let mut e = Engine::new();
        e.process_tx_inner(&tx1).expect("process tx failed");
        let result = e.process_tx_inner(&tx2);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), ClientIdNoMatch)
    }

    #[test]
    fn process_tx_inner_fail_resolve_tx_not_under_dispute() {
        let tx1 = TxInput {
            tx_type: TxType::Deposit,
            client_id: 0,
            id: 1,
            amount: Option::from(dec!(0)),
        };
        let tx2 = TxInput {
            tx_type: TxType::Resolve,
            client_id: 0,
            id: 1,
            amount: None,
        };

        let mut e = Engine::new();
        e.process_tx_inner(&tx1).expect("process tx failed");
        let result = e.process_tx_inner(&tx2);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), TxNotUnderDispute)
    }

    // process_tx_inner fail chargeback
    #[test]
    fn process_tx_inner_fail_chargeback() {
        let tx1 = TxInput {
            tx_type: TxType::Deposit,
            client_id: 0,
            id: 1,
            amount: Option::from(dec!(1)),
        };
        let tx2 = TxInput {
            tx_type: TxType::Dispute,
            client_id: 0,
            id: 1,
            amount: None,
        };
        let tx3 = TxInput {
            tx_type: TxType::Chargeback,
            client_id: 0,
            id: 1,
            amount: None,
        };
        let tx4 = TxInput {
            tx_type: TxType::Chargeback,
            client_id: 0,
            id: 1,
            amount: None,
        };
        let mut e = Engine::new();
        e.process_tx_inner(&tx1).expect("process tx failed");
        e.process_tx_inner(&tx2).expect("process tx failed");
        e.process_tx_inner(&tx3).expect("process tx failed");
        let result = e.process_tx_inner(&tx4);

        assert!(result.is_err())
    }

    #[test]
    fn process_tx_inner_fail_chargeback_tx_not_found() {
        let tx = TxInput {
            tx_type: TxType::Chargeback,
            client_id: 0,
            id: 1,
            amount: None,
        };

        let mut e = Engine::new();
        let result = e.process_tx_inner(&tx);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), TxNotFound)
    }

    #[test]
    fn process_tx_inner_fail_chargeback_client_id_no_match() {
        let tx1 = TxInput {
            tx_type: TxType::Deposit,
            client_id: 1,
            id: 1,
            amount: Option::from(dec!(0)),
        };
        let tx2 = TxInput {
            tx_type: TxType::Chargeback,
            client_id: 0,
            id: 1,
            amount: None,
        };

        let mut e = Engine::new();
        e.process_tx_inner(&tx1).expect("process tx failed");
        let result = e.process_tx_inner(&tx2);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), ClientIdNoMatch)
    }

    #[test]
    fn process_tx_inner_fail_chargeback_tx_not_under_dispute() {
        let tx1 = TxInput {
            tx_type: TxType::Deposit,
            client_id: 0,
            id: 1,
            amount: Option::from(dec!(0)),
        };
        let tx2 = TxInput {
            tx_type: TxType::Chargeback,
            client_id: 0,
            id: 1,
            amount: None,
        };

        let mut e = Engine::new();
        e.process_tx_inner(&tx1).expect("process tx failed");
        let result = e.process_tx_inner(&tx2);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), TxNotUnderDispute)
    }
}
