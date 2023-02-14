// Copyright (c) 2023 The MobileCoin Foundation

#![doc = include_str!("../README.md")]
#![deny(missing_docs, missing_debug_implementations, unsafe_code)]

use std::fmt::{Debug, Formatter};

type Result<T> = std::result::Result<T, VerificationError>;

#[derive(Debug, Eq, PartialEq)]
/// Failed to verify: {0}.
pub struct VerificationError(String);

impl<S: Into<String>> From<S> for VerificationError {
    fn from(message: S) -> Self {
        Self(message.into())
    }
}

trait VerificationStep {
    fn verify(&self) -> Result<()>;
}

impl Debug for dyn VerificationStep {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug)]
pub struct And {
    left: Box<dyn VerificationStep>,
    right: Box<dyn VerificationStep>,
}

impl VerificationStep for And {
    fn verify(&self) -> Result<()> {
        self.left.verify()?;
        self.right.verify()
    }
}

#[derive(Debug)]
pub struct Or {
    left: Box<dyn VerificationStep>,
    right: Box<dyn VerificationStep>,
}

impl VerificationStep for Or {
    fn verify(&self) -> Result<()> {
        self.left.verify().or(self.right.verify())
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct AlwaysTrue;

impl VerificationStep for AlwaysTrue {
    fn verify(&self) -> Result<()> {
        Ok(())
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct AlwaysFalse;

impl VerificationStep for AlwaysFalse {
    fn verify(&self) -> Result<()> {
        Err(VerificationError::from("AlwaysFalse"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn and_succeeds() {
        let and = And {
            left: Box::new(AlwaysTrue),
            right: Box::new(AlwaysTrue),
        };
        assert_eq!(and.verify(), Ok(()));
    }

    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
