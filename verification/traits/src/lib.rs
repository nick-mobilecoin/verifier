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

/// A verification step. These can chained together using the [`Or`] and [`And`]
/// types.
pub trait VerificationStep {
    fn verify(&self) -> Result<()>;
    fn as_any(&self) -> &dyn Any;
}

impl Debug for dyn VerificationStep {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VerificationStep").finish()
    }
}

/// Will perform a logical and operation for the [`VerificationStep::verify()`]
/// operation.
///
/// This is will be a short circuiting operation. If the `left` side fails
/// the `right` side will *not* be exercised.
#[derive(Debug)]
pub struct And {
    left: Box<dyn VerificationStep>,
    right: Box<dyn VerificationStep>,
}

impl And {
    pub fn new(left: Box<dyn VerificationStep>, right: Box<dyn VerificationStep>) -> Self {
        Self{ left, right }
    }
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

/// Will perform a logical or operation for the [`VerificationStep::verify()`]
/// operation.
///
/// This is will be a short circuiting operation. If the `left` side succeeds
/// the `right` side will *not* be exercised.
#[derive(Debug)]
pub struct Or {
    left: Box<dyn VerificationStep>,
    right: Box<dyn VerificationStep>,
}

impl Or {
    pub fn new(left: Box<dyn VerificationStep>, right: Box<dyn VerificationStep>) -> Self {
        Self{ left, right }
    }
}

impl VerificationStep for Or {
    fn verify(&self) -> Result<()> {
        self.left.verify().or_else(|_| self.right.verify())
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Will always succeed for the [`VerificationStep::verify()`] operation.
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

/// Will always fail for the [`VerificationStep::verify()`] operation.
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
    use super::*;
    use std::cell::Cell;

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
        let right = and
            .right
            .as_any()
            .downcast_ref::<Node>()
            .expect("Should be a Node");
        assert!(!right.verified_called.get());
    }

    #[test]
    fn and_fails_on_tail() {
        let and = And {
            left: Box::new(Node::new(true, "First")),
            right: Box::new(Node::new(false, "Second")),
        };
        assert_eq!(and.verify(), Err(VerificationError::from("Second")));
        let left = and
            .left
            .as_any()
            .downcast_ref::<Node>()
            .expect("Should be a Node");
        assert!(left.verified_called.get());
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
            left: Box::new(Node::new(true, "First")),
            right: Box::new(Node::new(false, "Second")),
        };
        assert_eq!(or.verify(), Ok(()));
        let right = or
            .right
            .as_any()
            .downcast_ref::<Node>()
            .expect("Should be a Node");
        assert!(!right.verified_called.get());
    }

    #[test]
    fn or_is_true_when_tail_is_true() {
        let or = Or {
            left: Box::new(Node::new(false, "First")),
            right: Box::new(Node::new(true, "Second")),
        };
        assert_eq!(or.verify(), Ok(()));
        let left = or
            .left
            .as_any()
            .downcast_ref::<Node>()
            .expect("Should be a Node");
        assert!(left.verified_called.get());
    }

    #[test]
    fn composing_or_and_and() {
        let or = Or {
            left: Box::new(And {
                left: Box::new(Node::new(true, "First")),
                right: Box::new(Node::new(false, "Second")),
            }),
            right: Box::new(Node::new(true, "Third")),
        };
        assert_eq!(or.verify(), Ok(()));
    }

    #[test]
    fn composing_and_and_or() {
        let and = And {
            left: Box::new(Or {
                left: Box::new(Node::new(true, "First")),
                right: Box::new(Node::new(false, "Second")),
            }),
            right: Box::new(Node::new(true, "Third")),
        };
        assert_eq!(and.verify(), Ok(()));
    }
}
