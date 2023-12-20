use nvim_oxi::{Object, ObjectKind};
use std::fmt::Debug;
use thiserror::Error;

use crate::widgets::support::{
    BoxConstraints, Margin, Padding, ParseBoxConstraintsError,
    ParsePaddingError,
};

#[derive(Debug, Default, Clone)]
pub struct BoxOptions {
    pub constraints: BoxConstraints,
    pub padding: Padding,
    pub margin: Margin,
}

#[derive(Debug, Error)]
pub enum ParseBoxOptionsError {
    #[error("invalid type, expected object, got {0:?}")]
    Type(ObjectKind),
    #[error("invalid field")]
    Constraits(#[from] ParseBoxConstraintsError),
    #[error("invalid padding/margin")]
    Field(#[from] ParsePaddingError),
}

impl TryFrom<Object> for BoxOptions {
    type Error = ParseBoxOptionsError;

    fn try_from(obj: Object) -> Result<Self, Self::Error> {
        match obj.kind() {
            ObjectKind::Nil => Ok(Self::default()),
            ObjectKind::Dictionary => {
                let mut constraints = BoxConstraints::default();
                let mut padding = Padding::default();
                let mut margin = Margin::default();

                let dict = unsafe { obj.into_dict_unchecked() };

                for (key, value) in dict.into_iter() {
                    match &*key.to_string_lossy() {
                        "constraints" => {
                            constraints = value.try_into()?;
                        }
                        "padding" => {
                            padding = value.try_into()?;
                        }
                        "margin" => {
                            margin = value.try_into()?;
                        }
                        _ => {}
                    }
                }

                Ok(Self {
                    constraints,
                    padding,
                    margin,
                })
            }
            _ => Err(ParseBoxOptionsError::Type(obj.kind())),
        }
    }
}
