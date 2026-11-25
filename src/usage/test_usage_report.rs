//! A usage report in the shape `/v1/zeldoc/usage` returns, shared by the
//! usage tests.

use super::data_transfer_objects::usage_report_dto::UsageReportDTO;

pub fn report() -> UsageReportDTO {
    serde_json::from_str(
        r#"{
            "key_name": "laptop",
            "range": {"start_date": "2026-10-01", "end_date": "2026-10-05"},
            "totals": {"cost_usd": "3.1849", "requests": 162, "input_tokens": 2110000,
                       "output_tokens": 59000, "cache_read_tokens": 1500000, "cache_write_tokens": 12000},
            "models": [
                {"id": "anthropic/claude-sonnet", "cost_usd": "3.1849", "requests": 42, "input_tokens": 910000,
                 "output_tokens": 25000, "cache_read_tokens": 700000, "cache_write_tokens": 12000},
                {"id": "zdev", "cost_usd": "0", "requests": 120, "input_tokens": 1200000,
                 "output_tokens": 34000, "cache_read_tokens": 800000, "cache_write_tokens": 0}
            ],
            "monthly_limit": {"limit_usd": "50.00", "spent_usd": "3.1849"},
            "credits": {"available_usd": "87.66"},
            "zdev": null
        }"#,
    )
    .unwrap()
}
