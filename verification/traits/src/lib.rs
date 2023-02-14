// Copyright (c) 2023 The MobileCoin Foundation

#![doc = include_str!("../README.md")]
#![deny(missing_docs, missing_debug_implementations, unsafe_code)]

use std::any::Any;
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
    fn as_any(&self) -> &dyn Any;
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
    fn as_any(&self) -> &dyn Any {
        self
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
    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct AlwaysTrue;

impl VerificationStep for AlwaysTrue {
    fn verify(&self) -> Result<()> {
        Ok(())
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct AlwaysFalse;

impl VerificationStep for AlwaysFalse {
    fn verify(&self) -> Result<()> {
        Err(VerificationError::from("AlwaysFalse"))
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;
    use super::*;

    pub struct Node {
        pub succeed: bool,
        pub message: String,
        pub verified_called: Cell<bool>,
    }

    impl Node {
        pub fn new(succeed: bool, message: impl Into<String>) -> Self {
            Self {
                succeed,
                message: message.into(),
                verified_called: Cell::new(false),
            }
        }
    }

    impl VerificationStep for Node {
        fn verify(&self) -> Result<()> {
            self.verified_called.replace(true);
            if self.succeed {
                Ok(())
            } else {
                Err(VerificationError::from(self.message.clone()))
            }
        }
        fn as_any(&self) -> &dyn Any {
            self
        }
    }

    #[test]
    fn and_succeeds() {
        let and = And {
            left: Box::new(AlwaysTrue),
            right: Box::new(AlwaysTrue),
        };
        assert_eq!(and.verify(), Ok(()));
    }

    #[test]
    fn and_short_circuits() {
        let and = And {
            left: Box::new(Node::new(false, "First")),
            right: Box::new(Node::new(true, "Second")),
        };
        assert_eq!(and.verify(), Err(VerificationError::from("First")));
        let right = and.right.as_any().downcast_ref::<Node>().expect("Should be a Node");
        assert_eq!(right.verified_called.get(), false);
    }

    #[test]
    fn and_fails_on_tail() {
        let and = And {
            left: Box::new(Node::new(false, "First")),
            right: Box::new(Node::new(true, "Second")),
        };
        let left = and.left.as_any().downcast_ref::<Node>().expect("Should be a Node");
        assert_eq!(left.verified_called.get(), true);
        assert_eq!(and.verify(), Err(VerificationError::from("Second")));
    }

    #[test]
    fn or_fails_for_both_failing() {
        let or = Or {
            left: Box::new(AlwaysFalse),
            right: Box::new(AlwaysFalse),
        };
        assert_eq!(or.verify(), Err(VerificationError::from("AlwaysFalse")));
    }

    #[test]
    fn or_short_circuits() {
        let or = Or {
            left: Box::new(AlwaysTrue),
            right: Box::new(AlwaysFalse),
        };
        assert_eq!(or.verify(), Ok(()));
    }

    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
