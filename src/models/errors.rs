use core::fmt;
use std::fmt::Formatter;

#[derive(Clone, Debug, PartialEq)]
pub enum Error {
    /// insufficient available funds
    InsufficientFunds,
    /// calculation overflow
    Overflow,
    /// Negative amount
    NegativeAmount,
    /// Tx not found, partner error
    TxNotFound,
    /// Tx not under dispute, partner error
    TxNotUnderDispute,
    /// Account locked
    AccountLocked,
    /// Client id doesn't match
    ClientIdNoMatch,
    /// Tx id conflict
    TxIdConflict,
    /// Tx is not a deposit
    TxNotADeposit,
    /// Tx invalid amount
    TxInvalidAmount,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::InsufficientFunds => {
                write!(f, "insufficient available funds")
            }
            Error::Overflow => {
                write!(f, "calculation overflow")
            }
            Error::NegativeAmount => {
                write!(f, "negative amount")
            }
            Error::TxNotFound => {
                write!(f, "tx not found, partner error")
            }
            Error::TxNotUnderDispute => {
                write!(f, "tx not under dispute, partner error")
            }
            Error::AccountLocked => {
                write!(f, "account locked")
            }
            Error::ClientIdNoMatch => {
                write!(f, "client id doesn't match")
            }
            Error::TxIdConflict => {
                write!(f, "tx id conflict")
            }
            Error::TxNotADeposit => {
                write!(f, "tx is not a deposit")
            }
            Error::TxInvalidAmount => {
                write!(f, "tx invalid amount")
            }
        }
    }
}
