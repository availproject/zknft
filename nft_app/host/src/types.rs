use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, Default)]
pub struct ReceiptQuery {
    pub key: String,
}
