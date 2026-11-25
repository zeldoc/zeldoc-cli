use serde::{Deserialize, Serialize};

use super::credits_dto::CreditsDTO;
use super::model_usage_dto::ModelUsageDTO;
use super::monthly_limit_dto::MonthlyLimitDTO;
use super::usage_date_range_dto::UsageDateRangeDTO;
use super::usage_totals_dto::UsageTotalsDTO;
use super::zdev_allowance_dto::ZDevAllowanceDTO;

/// The usage endpoint's response: what the API key itself used over a period.
/// Other keys of the organization are not included.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UsageReportDTO {
    /// The key's name, when it has one.
    pub key_name: Option<String>,
    pub range: UsageDateRangeDTO,
    pub totals: UsageTotalsDTO,
    /// One entry per model the key called in the period, most expensive first.
    pub models: Vec<ModelUsageDTO>,
    /// `None` when the key has no monthly limit.
    pub monthly_limit: Option<MonthlyLimitDTO>,
    /// `None` when the organization is invoiced instead of using prepaid
    /// credits.
    pub credits: Option<CreditsDTO>,
    /// `None` unless the key belongs to a person with a ZDev seat.
    #[serde(default)]
    pub zdev: Option<ZDevAllowanceDTO>,
}
