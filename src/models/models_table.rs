use std::io::{self, Write};

use super::data_transfer_objects::model_capabilities_dto::ModelCapabilitiesDTO;
use super::data_transfer_objects::model_dto::ModelDTO;
use crate::number_text::compact;
use crate::text_table::{self, Align};

const COLUMNS: [(&str, Align); 7] = [
    ("MODEL", Align::Left),
    ("MODE", Align::Left),
    ("CONTEXT", Align::Right),
    ("OUTPUT", Align::Right),
    ("IN $/1M", Align::Right),
    ("OUT $/1M", Align::Right),
    ("CAPABILITIES", Align::Left),
];

/// The models as an aligned table, one row per model, token limits shortened
/// (`1M`); `-` where the catalog has no value.
pub fn write_table(out: &mut impl Write, models: &[ModelDTO]) -> io::Result<()> {
    let rows: Vec<[String; 7]> = models
        .iter()
        .map(|model| {
            [
                model.id.to_string(),
                or_dash(model.mode.as_ref()),
                or_dash(model.limits.context.map(compact)),
                or_dash(model.limits.output.map(compact)),
                or_dash(model.pricing.input),
                or_dash(model.pricing.output),
                capability_names(&model.capabilities).join(", "),
            ]
        })
        .collect();
    text_table::write_table(out, &COLUMNS, &rows)
}

fn capability_names(capabilities: &ModelCapabilitiesDTO) -> Vec<&'static str> {
    [
        (capabilities.tool_calls, "tools"),
        (capabilities.reasoning, "reasoning"),
        (capabilities.vision, "vision"),
        (capabilities.pdf_input, "pdf"),
        (capabilities.prompt_caching, "caching"),
    ]
    .into_iter()
    .filter_map(|(supported, name)| supported.then_some(name))
    .collect()
}

fn or_dash(value: Option<impl ToString>) -> String {
    value.map_or_else(|| "-".to_string(), |value| value.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::test_catalog;

    #[test]
    fn table_aligns_columns_and_marks_missing_values() {
        let mut out = Vec::new();
        write_table(&mut out, &test_catalog::models()).unwrap();
        assert_eq!(
            String::from_utf8(out).unwrap(),
            "\
MODEL                          MODE       CONTEXT  OUTPUT  IN $/1M  OUT $/1M  CAPABILITIES
zdev                           chat            1M    131K        0         0  tools, reasoning, vision
openai/text-embedding-3-large  embedding     8.2K       -     0.13      0.13
jev-latest                     -                -       -        -         -
"
        );
    }
}
