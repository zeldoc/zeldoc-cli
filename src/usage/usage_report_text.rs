use std::io::{self, Write};

use rust_decimal::{Decimal, RoundingStrategy};

use super::data_transfer_objects::profile_usage_dto::ProfileUsageDTO;
use super::data_transfer_objects::usage_report_dto::UsageReportDTO;
use super::data_transfer_objects::usage_totals_dto::UsageTotalsDTO;
use super::data_transfer_objects::zdev_allowance_dto::ZDevAllowanceDTO;
use super::request_parameters::usage_period::UsagePeriod;
use crate::number_text::{compact, group_digits};
use crate::text_table::{self, Align};

const COLUMNS: [(&str, Align); 6] = [
    ("MODEL", Align::Left),
    ("REQUESTS", Align::Right),
    ("INPUT", Align::Right),
    ("OUTPUT", Align::Right),
    ("CACHE READ", Align::Right),
    ("COST $", Align::Right),
];

/// The report as text: the key and period, a table with one row per model and
/// a total row (left out when nothing was used), then the monthly limit and
/// credits when the report has them.
pub fn write_report(
    out: &mut impl Write,
    period: UsagePeriod,
    report: &UsageReportDTO,
) -> io::Result<()> {
    let key_name = report.key_name.as_deref().unwrap_or("(no name)");
    writeln!(out, "Key     {key_name}")?;
    writeln!(
        out,
        "Period  {}, {} to {} (UTC)",
        period.label(),
        report.range.start_date,
        report.range.end_date
    )?;

    if !report.models.is_empty() {
        writeln!(out)?;
        let rows: Vec<[String; 6]> = report
            .models
            .iter()
            .map(|model| usage_row(model.id.to_string(), &model.usage))
            .chain(std::iter::once(usage_row(
                "TOTAL".to_string(),
                &report.totals,
            )))
            .collect();
        text_table::write_table(out, &COLUMNS, &rows)?;
    }

    if report.monthly_limit.is_some() || report.credits.is_some() || report.zdev.is_some() {
        writeln!(out)?;
    }
    if let Some(limit) = &report.monthly_limit {
        writeln!(
            out,
            "Monthly limit  ${} of ${} used this month ({})",
            dollars(limit.spent_usd),
            dollars(limit.limit_usd),
            percent(limit.spent_usd, limit.limit_usd)
        )?;
    }
    if let Some(zdev) = &report.zdev {
        writeln!(out, "ZDev           {}", zdev_line(zdev))?;
    }
    if let Some(credits) = &report.credits {
        writeln!(
            out,
            "Credits        ${} available to the organization",
            dollars(credits.available_usd)
        )?;
    }
    Ok(())
}

const PROFILE_COLUMNS: [(&str, Align); 7] = [
    ("PROFILE", Align::Left),
    ("KEY", Align::Left),
    ("REQUESTS", Align::Right),
    ("INPUT", Align::Right),
    ("OUTPUT", Align::Right),
    ("COST $", Align::Right),
    ("MONTHLY LIMIT", Align::Left),
];

/// Every saved profile's usage, one row each, with a total of requests,
/// tokens and cost. A profile whose report failed shows `error`; the reason
/// is printed separately.
pub fn write_profile_usages(
    out: &mut impl Write,
    period: UsagePeriod,
    usages: &[ProfileUsageDTO],
) -> io::Result<()> {
    if let Some(range) = usages
        .iter()
        .find_map(|usage| usage.report.as_ref())
        .map(|report| &report.range)
    {
        writeln!(
            out,
            "Period  {}, {} to {} (UTC)",
            period.label(),
            range.start_date,
            range.end_date
        )?;
        writeln!(out)?;
    }

    let reports = || usages.iter().filter_map(|usage| usage.report.as_ref());
    let total_row = [
        "TOTAL".to_string(),
        String::new(),
        compact(reports().map(|report| report.totals.requests).sum::<u64>()),
        compact(
            reports()
                .map(|report| report.totals.input_tokens)
                .sum::<u64>(),
        ),
        compact(
            reports()
                .map(|report| report.totals.output_tokens)
                .sum::<u64>(),
        ),
        dollars(reports().map(|report| report.totals.cost_usd).sum()),
        String::new(),
    ];
    let rows: Vec<[String; 7]> = usages
        .iter()
        .map(|usage| match &usage.report {
            Some(report) => [
                usage.profile.to_string(),
                report
                    .key_name
                    .clone()
                    .unwrap_or_else(|| "(no name)".to_string()),
                compact(report.totals.requests),
                compact(report.totals.input_tokens),
                compact(report.totals.output_tokens),
                dollars(report.totals.cost_usd),
                report.monthly_limit.as_ref().map_or_else(
                    || "-".to_string(),
                    |limit| {
                        format!(
                            "{} of ${}",
                            percent(limit.spent_usd, limit.limit_usd),
                            dollars(limit.limit_usd)
                        )
                    },
                ),
            ],
            None => [
                usage.profile.to_string(),
                "error".to_string(),
                String::new(),
                String::new(),
                String::new(),
                String::new(),
                String::new(),
            ],
        })
        .chain(std::iter::once(total_row))
        .collect();
    text_table::write_table(out, &PROFILE_COLUMNS, &rows)
}

/// The ZDev allowance in a line: how much of the plan's tokens are used, and
/// what happens once they are.
fn zdev_line(zdev: &ZDevAllowanceDTO) -> String {
    let mut line = match zdev.included_tokens {
        Some(included) => format!(
            "{} plan: {} of {} tokens used this month ({})",
            zdev.plan,
            compact(zdev.used_tokens),
            compact(included),
            percent(Decimal::from(zdev.used_tokens), Decimal::from(included))
        ),
        None => format!(
            "{} plan: {} tokens used this month (fair use)",
            zdev.plan,
            compact(zdev.used_tokens)
        ),
    };
    let used_up = zdev
        .included_tokens
        .is_some_and(|included| zdev.used_tokens >= included);
    if zdev.paused {
        line.push_str(", paused until the 1st");
    } else if used_up && zdev.overage {
        line.push_str(", overage billed");
    }
    line
}

fn usage_row(name: String, usage: &UsageTotalsDTO) -> [String; 6] {
    [
        name,
        compact(usage.requests),
        compact(usage.input_tokens),
        compact(usage.output_tokens),
        compact(usage.cache_read_tokens),
        dollars(usage.cost_usd),
    ]
}

/// An amount in whole cents, half a cent rounding up, thousands grouped
/// (`1,234.50`). Above zero but below half a cent shows as `<0.01`, so a
/// little usage does not read as free.
fn dollars(amount: Decimal) -> String {
    let cents = amount.round_dp_with_strategy(2, RoundingStrategy::MidpointAwayFromZero);
    if cents.is_zero() && amount > Decimal::ZERO {
        return "<0.01".to_string();
    }
    let text = format!("{:.2}", cents.abs());
    let (whole, fraction) = text.split_once('.').unwrap_or((&text, "00"));
    let sign = if cents.is_sign_negative() && !cents.is_zero() {
        "-"
    } else {
        ""
    };
    format!("{sign}{}.{fraction}", group_digits(whole))
}

fn percent(part: Decimal, whole: Decimal) -> String {
    if whole <= Decimal::ZERO {
        return "-".to_string();
    }
    format!("{}%", (part * Decimal::ONE_HUNDRED / whole).round())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::usage::test_usage_report;

    fn rendered(report: &UsageReportDTO) -> String {
        let mut out = Vec::new();
        write_report(&mut out, UsagePeriod::Month, report).unwrap();
        String::from_utf8(out).unwrap()
    }

    #[test]
    fn report_lists_models_with_a_total_then_limit_and_credits() {
        assert_eq!(
            rendered(&test_usage_report::report()),
            "\
Key     laptop
Period  this month, 2026-10-01 to 2026-10-05 (UTC)

MODEL                    REQUESTS  INPUT  OUTPUT  CACHE READ  COST $
anthropic/claude-sonnet        42   910K     25K        700K    3.18
zdev                          120   1.2M     34K        800K    0.00
TOTAL                         162   2.1M     59K        1.5M    3.18

Monthly limit  $3.18 of $50.00 used this month (6%)
Credits        $87.66 available to the organization
"
        );
    }

    #[test]
    fn report_without_usage_limit_or_credits_is_just_key_and_period() {
        let mut report = test_usage_report::report();
        report.key_name = None;
        report.models.clear();
        report.monthly_limit = None;
        report.credits = None;
        assert_eq!(
            rendered(&report),
            "\
Key     (no name)
Period  this month, 2026-10-01 to 2026-10-05 (UTC)
"
        );
    }

    #[test]
    fn zdev_allowance_says_how_much_is_used_and_what_happens_after() {
        let mut report = test_usage_report::report();
        report.monthly_limit = None;
        report.credits = None;
        report.models.clear();
        report.zdev = Some(
            serde_json::from_str(
                r#"{"plan":"go","included_tokens":100000000,"used_tokens":40000000,"paused":false,"overage":false}"#,
            )
            .unwrap(),
        );
        assert!(
            rendered(&report)
                .ends_with("\nZDev           Go plan: 40M of 100M tokens used this month (40%)\n")
        );

        let zdev = report.zdev.as_mut().unwrap();
        zdev.used_tokens = 100_000_000;
        zdev.paused = true;
        assert!(rendered(&report).ends_with("(100%), paused until the 1st\n"));

        let zdev = report.zdev.as_mut().unwrap();
        zdev.paused = false;
        zdev.overage = true;
        assert!(rendered(&report).ends_with("(100%), overage billed\n"));

        let zdev = report.zdev.as_mut().unwrap();
        zdev.plan = serde_json::from_str("\"max\"").unwrap();
        zdev.included_tokens = None;
        assert!(rendered(&report).ends_with("Max plan: 100M tokens used this month (fair use)\n"));
    }

    #[test]
    fn profile_usages_have_one_row_each_and_a_total() {
        let mut globex = test_usage_report::report();
        globex.key_name = None;
        globex.monthly_limit = None;
        let usages = [
            ProfileUsageDTO {
                profile: "acme".parse().unwrap(),
                report: Some(test_usage_report::report()),
                error: None,
            },
            ProfileUsageDTO {
                profile: "globex".parse().unwrap(),
                report: Some(globex),
                error: None,
            },
            ProfileUsageDTO {
                profile: "initech".parse().unwrap(),
                report: None,
                error: Some("Zeldoc.ai returned 401 Unauthorized".to_string()),
            },
        ];
        let mut out = Vec::new();
        write_profile_usages(&mut out, UsagePeriod::Month, &usages).unwrap();
        assert_eq!(
            String::from_utf8(out).unwrap(),
            "\
Period  this month, 2026-10-01 to 2026-10-05 (UTC)

PROFILE  KEY        REQUESTS  INPUT  OUTPUT  COST $  MONTHLY LIMIT
acme     laptop          162   2.1M     59K    3.18  6% of $50.00
globex   (no name)       162   2.1M     59K    3.18  -
initech  error
TOTAL                    324   4.2M    118K    6.37
"
        );
    }

    #[test]
    fn dollars_round_to_cents_but_never_hide_usage() {
        assert_eq!(dollars(Decimal::ZERO), "0.00");
        assert_eq!(dollars(Decimal::new(4, 3)), "<0.01");
        assert_eq!(dollars(Decimal::new(5, 3)), "0.01");
        assert_eq!(dollars(Decimal::new(31849, 4)), "3.18");
        assert_eq!(dollars(Decimal::new(50, 0)), "50.00");
        assert_eq!(dollars(Decimal::new(123456789, 2)), "1,234,567.89");
        assert_eq!(dollars(Decimal::new(-123450, 2)), "-1,234.50");
    }
}
