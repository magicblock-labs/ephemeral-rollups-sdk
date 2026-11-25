use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BuilderError {
    PayerNotSet,
    AuthorityNotSet,
    PermissionedAccountNotSet,
    PermissionNotSet,
    MagicProgramNotSet,
    MagicContextNotSet,
    SerializationError(String),
}

impl fmt::Display for BuilderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BuilderError::PayerNotSet => write!(f, "payer is not set"),
            BuilderError::AuthorityNotSet => write!(f, "authority is not set"),
            BuilderError::PermissionedAccountNotSet => write!(f, "permissioned_account is not set"),
            BuilderError::PermissionNotSet => write!(f, "permission is not set"),
            BuilderError::MagicProgramNotSet => write!(f, "magic_program is not set"),
            BuilderError::MagicContextNotSet => write!(f, "magic_context is not set"),
            BuilderError::SerializationError(msg) => write!(f, "serialization error: {}", msg),
        }
    }
}

impl std::error::Error for BuilderError {}
