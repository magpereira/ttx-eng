use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};

use crate::models::errors::Error;
use crate::models::errors::Error::{AccountLocked, InsufficientFunds, NegativeAmount, Overflow};
use crate::models::tx::ClientId;

const PRECISION: u32 = 4;

#[derive(Debug, PartialEq)]
pub(crate) struct Client {
    id: ClientId,
    available: Decimal,
    held: Decimal,
    locked: bool,
}

impl Client {
    pub(crate) fn new(client_id: ClientId) -> Self {
        Self {
            id: client_id,
            available: dec!(0),
            held: dec!(0),
            locked: false,
        }
    }

    pub(crate) fn deposit(&mut self, amount: &Decimal) -> Result<(), Error> {
        if amount.is_sign_negative() {
            return Err(NegativeAmount);
        }

        if self.locked {
            return Err(AccountLocked);
        }

        match self.available.checked_add(*amount) {
            None => Err(Overflow),
            Some(val) => {
                self.available = val.round_dp(PRECISION);
                Ok(())
            }
        }
    }

    pub(crate) fn withdraw(&mut self, amount: &Decimal) -> Result<(), Error> {
        if amount.is_sign_negative() {
            return Err(NegativeAmount);
        }

        if self.locked {
            return Err(AccountLocked);
        }

        if amount > &self.available {
            return Err(InsufficientFunds);
        }

        match self.available.checked_sub(*amount) {
            None => Err(Overflow),
            Some(val) => {
                self.available = val.round_dp(PRECISION);
                Ok(())
            }
        }
    }

    pub(crate) fn dispute(&mut self, amount: &Decimal) -> Result<(), Error> {
        if amount.is_sign_negative() {
            return Err(NegativeAmount);
        }

        if self.locked {
            return Err(AccountLocked);
        }

        match self.available.checked_sub(*amount) {
            None => return Err(Overflow),
            Some(val) => {
                self.available = val.round_dp(PRECISION);
            }
        };

        match self.held.checked_add(*amount) {
            None => Err(Overflow),
            Some(val) => {
                self.held = val.round_dp(PRECISION);
                Ok(())
            }
        }
    }

    pub(crate) fn resolve(&mut self, amount: &Decimal) -> Result<(), Error> {
        if amount.is_sign_negative() {
            return Err(NegativeAmount);
        }

        if self.locked {
            return Err(AccountLocked);
        }

        match self.available.checked_add(*amount) {
            None => return Err(Overflow),
            Some(val) => {
                self.available = val.round_dp(PRECISION);
            }
        };

        match self.held.checked_sub(*amount) {
            None => Err(Overflow),
            Some(val) => {
                self.held = val.round_dp(PRECISION);
                Ok(())
            }
        }
    }

    pub(crate) fn chargeback(&mut self, amount: &Decimal) -> Result<(), Error> {
        if amount.is_sign_negative() {
            return Err(NegativeAmount);
        }

        if self.locked {
            return Err(AccountLocked);
        }

        self.locked = true;
        match self.held.checked_sub(*amount) {
            None => Err(Overflow),
            Some(val) => {
                self.held = val.round_dp(PRECISION);
                Ok(())
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct ClientReport {
    #[serde(rename = "client")]
    id: ClientId,

    available: Decimal,
    held: Decimal,
    total: Decimal,
    locked: bool,
}

impl ClientReport {
    pub(crate) fn new(c: &Client) -> Self {
        Self {
            id: c.id,
            available: c.available,
            held: c.held,
            total: c.available + c.held,
            locked: c.locked,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    //client new
    #[test]
    fn client_new() {
        let client = Client::new(1);

        assert_eq!(client.id, 1);
        assert_eq!(client.available, dec!(0));
        assert_eq!(client.held, dec!(0));
        assert!(!client.locked)
    }

    // deposit
    #[test]
    fn deposit_success() {
        let val = dec!(1);
        let mut client = Client::new(1);
        client.deposit(&val).expect("failed to deposit");

        assert_eq!(client.id, 1);
        assert_eq!(client.available, val);
        assert_eq!(client.held, dec!(0));
        assert!(!client.locked)
    }

    #[test]
    fn deposit_success_round() {
        let val = dec!(3.12345);
        let mut client = Client::new(1);
        client.deposit(&val).expect("failed to deposit");

        assert_eq!(client.id, 1);
        assert_eq!(client.available.to_string(), "3.1234");
        assert_eq!(client.held, dec!(0));
        assert!(!client.locked)
    }

    #[test]
    fn deposit_fail_locked() {
        let mut client = Client::new(1);
        client.locked = true;
        let result = client.deposit(&dec!(1));

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), AccountLocked);
        assert_eq!(client.id, 1);
        assert_eq!(client.available, dec!(0));
        assert_eq!(client.held, dec!(0));
        assert!(client.locked)
    }

    #[test]
    fn deposit_fail_negative_amount() {
        let mut client = Client::new(1);
        let result = client.deposit(&dec!(-1));

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), NegativeAmount);
        assert_eq!(client.id, 1);
        assert_eq!(client.available, dec!(0));
        assert_eq!(client.held, dec!(0));
        assert!(!client.locked)
    }

    #[test]
    fn deposit_fail_overflow() {
        let mut client = Client::new(1);
        client
            .deposit(&Decimal::MAX)
            .expect("failed to deposit max");
        let result = client.deposit(&Decimal::MAX);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), Overflow);
        assert_eq!(client.id, 1);
        assert_eq!(client.available, Decimal::MAX);
        assert_eq!(client.held, dec!(0));
        assert!(!client.locked)
    }

    //withdraw
    #[test]
    fn withdraw_success() {
        let val = dec!(1);
        let mut client = Client::new(1);
        client.deposit(&val).expect("failed to deposit");
        client.withdraw(&val).expect("failed to withdraw");

        assert_eq!(client.id, 1);
        assert_eq!(client.available, dec!(0));
        assert_eq!(client.held, dec!(0));
        assert!(!client.locked)
    }

    #[test]
    fn withdraw_success_round() {
        let val = dec!(3.12345);
        let mut client = Client::new(1);
        client.deposit(&dec!(4)).expect("failed to deposit");
        client.withdraw(&val).expect("failed to withdraw");

        assert_eq!(client.id, 1);
        assert_eq!(client.available.to_string(), "0.8766");
        assert_eq!(client.held, dec!(0));
        assert!(!client.locked)
    }

    #[test]
    fn withdraw_fail_insufficient_funds() {
        let val = dec!(1);
        let mut client = Client::new(1);
        let result = client.withdraw(&val);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), InsufficientFunds);
        assert_eq!(client.id, 1);
        assert_eq!(client.available, dec!(0));
        assert_eq!(client.held, dec!(0));
        assert!(!client.locked)
    }

    #[test]
    fn withdraw_fail_locked() {
        let mut client = Client::new(1);
        client.locked = true;
        let result = client.withdraw(&dec!(1));

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), AccountLocked);
        assert_eq!(client.id, 1);
        assert_eq!(client.available, dec!(0));
        assert_eq!(client.held, dec!(0));
        assert!(client.locked)
    }

    #[test]
    fn withdraw_fail_negative_amount() {
        let mut client = Client::new(1);
        client.deposit(&dec!(1)).expect("failed to deposit max");
        let result = client.withdraw(&Decimal::MIN);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), NegativeAmount);
        assert_eq!(client.id, 1);
        assert_eq!(client.available, dec!(1));
        assert_eq!(client.held, dec!(0));
        assert!(!client.locked)
    }

    //dispute
    #[test]
    fn dispute_success() {
        let val = dec!(1);
        let mut client = Client::new(1);
        client.deposit(&val).expect("failed to deposit");
        client.dispute(&val).expect("failed to dispute");

        assert_eq!(client.id, 1);
        assert_eq!(client.available, dec!(0));
        assert_eq!(client.held, val);
        assert!(!client.locked)
    }

    #[test]
    fn dispute_success_round() {
        let val = dec!(3.12345);
        let mut client = Client::new(1);
        client.deposit(&dec!(4)).expect("failed to deposit");
        client.dispute(&val).expect("failed to dispute");

        assert_eq!(client.id, 1);
        assert_eq!(client.available.to_string(), "0.8766");
        assert_eq!(client.held.to_string(), "3.1234");
        assert!(!client.locked);
    }

    #[test]
    fn dispute_fail_locked() {
        let mut client = Client::new(1);
        client.locked = true;
        let result = client.dispute(&dec!(1));

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), AccountLocked);
        assert_eq!(client.id, 1);
        assert_eq!(client.available, dec!(0));
        assert_eq!(client.held, dec!(0));
        assert!(client.locked)
    }

    #[test]
    fn dispute_fail_negative_amount() {
        let mut client = Client::new(1);
        let result = client.dispute(&dec!(-1));

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), NegativeAmount);
        assert_eq!(client.id, 1);
        assert_eq!(client.available, dec!(0));
        assert_eq!(client.held, dec!(0));
        assert!(!client.locked)
    }

    #[test]
    fn dispute_fail_overflow() {
        let mut client = Client::new(1);
        client.deposit(&dec!(1)).expect("failed to deposit");

        client.dispute(&Decimal::MAX).expect("failed dispute");

        assert_eq!(client.id, 1);
        assert_eq!(client.available, dec!(-79228162514264337593543950334));
        assert_eq!(client.held, Decimal::MAX);
        assert!(!client.locked);

        //available overflow
        let result = client.dispute(&Decimal::MAX);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), Overflow);
        assert_eq!(client.id, 1);
        assert_eq!(client.available, dec!(-79228162514264337593543950334));
        assert_eq!(client.held, Decimal::MAX);
        assert!(!client.locked);

        //held overflow
        client.deposit(&Decimal::MAX).expect("failed to deposit");
        let result = client.dispute(&Decimal::MAX);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), Overflow);
        assert_eq!(client.id, 1);
        assert_eq!(client.available, dec!(-79228162514264337593543950334));
        assert_eq!(client.held, Decimal::MAX);
        assert!(!client.locked)
    }

    //resolve
    #[test]
    fn resolve_success() {
        let val = dec!(1);
        let mut client = Client::new(1);
        client.deposit(&val).expect("failed to deposit");
        client.dispute(&val).expect("failed to dispute");
        client.resolve(&val).expect("failed to dispute");

        assert_eq!(client.id, 1);
        assert_eq!(client.available, val);
        assert_eq!(client.held, dec!(0));
        assert!(!client.locked)
    }

    #[test]
    fn resolve_success_round() {
        let val = dec!(3.12345);
        let mut client = Client::new(1);
        client.deposit(&dec!(4)).expect("failed to deposit");
        client.dispute(&val).expect("failed to dispute");
        client.resolve(&dec!(1.23456)).expect("failed to dispute");

        assert_eq!(client.id, 1);
        assert_eq!(client.available.to_string(), "2.1112");
        assert_eq!(client.held.to_string(), "1.8888");
        assert!(!client.locked);
    }

    #[test]
    fn resolve_fail_locked() {
        let mut client = Client::new(1);
        client.locked = true;
        let result = client.resolve(&dec!(1));

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), AccountLocked);
        assert_eq!(client.id, 1);
        assert_eq!(client.available, dec!(0));
        assert_eq!(client.held, dec!(0));
        assert!(client.locked)
    }

    #[test]
    fn resolve_fail_negative_amount() {
        let mut client = Client::new(1);
        let result = client.resolve(&dec!(-1));

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), NegativeAmount);
        assert_eq!(client.id, 1);
        assert_eq!(client.available, dec!(0));
        assert_eq!(client.held, dec!(0));
        assert!(!client.locked)
    }

    #[test]
    fn resolve_fail_overflow() {
        let mut client = Client::new(1);
        client.deposit(&dec!(100)).expect("failed to deposit");

        //available overflow
        let result = client.resolve(&Decimal::MAX);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), Overflow);
        assert_eq!(client.id, 1);
        assert_eq!(client.available, dec!(100));
        assert_eq!(client.held, dec!(0));
        assert!(!client.locked);

        //held overflow
        client.resolve(&dec!(200)).expect("failed to resolve");
        client.withdraw(&dec!(300)).expect("failed to withdraw");
        let result = client.resolve(&Decimal::MAX);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), Overflow);
        assert_eq!(client.id, 1);
        assert_eq!(client.available, Decimal::MAX);
        assert_eq!(client.held, dec!(-200));
        assert!(!client.locked)
    }

    //chargeback
    #[test]
    fn chargeback_success() {
        let val = dec!(1);
        let mut client = Client::new(1);
        client.chargeback(&val).expect("failed to chargeback");

        assert_eq!(client.id, 1);
        assert_eq!(client.available, dec!(0));
        assert_eq!(client.held, dec!(-1));
        assert!(client.locked)
    }

    #[test]
    fn chargeback_success_round() {
        let val = dec!(3.12345);
        let mut client = Client::new(1);
        client.chargeback(&val).expect("failed to deposit");

        assert_eq!(client.id, 1);
        assert_eq!(client.available.to_string(), "0");
        assert_eq!(client.held.to_string(), "-3.1234");
        assert!(client.locked)
    }

    #[test]
    fn chargeback_fail_locked() {
        let mut client = Client::new(1);
        client.locked = true;
        let result = client.chargeback(&dec!(1));

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), AccountLocked);
        assert_eq!(client.id, 1);
        assert_eq!(client.available, dec!(0));
        assert_eq!(client.held, dec!(0));
        assert!(client.locked)
    }

    #[test]
    fn chargeback_fail_negative_amount() {
        let mut client = Client::new(1);
        let result = client.chargeback(&dec!(-1));

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), NegativeAmount);
        assert_eq!(client.id, 1);
        assert_eq!(client.available, dec!(0));
        assert_eq!(client.held, dec!(0));
        assert!(!client.locked)
    }

    #[test]
    fn chargeback_fail_overflow() {
        let mut client = Client::new(1);
        client.resolve(&dec!(1)).expect("failed to deposit max");
        let result = client.chargeback(&Decimal::MAX);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), Overflow);
        assert_eq!(client.id, 1);
        assert_eq!(client.available, dec!(1));
        assert_eq!(client.held, dec!(-1));
        assert!(client.locked)
    }

    //client report new
    #[test]
    fn client_report_new() {
        let mut client = Client::new(1);
        client.deposit(&dec!(1)).expect("failed to deposit");
        let client_report = ClientReport::new(&client);

        assert_eq!(client_report.id, 1);
        assert_eq!(client_report.available, dec!(1));
        assert_eq!(client_report.held, dec!(0));
        assert_eq!(client_report.total, dec!(1));
        assert!(!client_report.locked)
    }
}
