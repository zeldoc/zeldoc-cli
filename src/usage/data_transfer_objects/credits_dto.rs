use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// The prepaid credits the key's organization has left. Shared by every key
/// of the organization.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CreditsDTO {
    pub available_usd: Decimal,
}
