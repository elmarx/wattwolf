use phf::phf_map;
use serde::{Deserialize, Serialize};
use serde::{Deserializer, Serializer};
use sml_rs::parser::OctetStr;

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Unit {
    Kw,
    Kwh,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ObisKeyFigure {
    pub exact: OctetStr<'static>,
    pub simplified: &'static str,
    pub unit: Unit,
}

// Static instances for each OBIS key figure
pub(crate) static OBIS_1_8_0: ObisKeyFigure = ObisKeyFigure {
    exact: &[1, 0, 1, 8, 0, 255],
    simplified: "1.8.0",
    unit: Unit::Kwh,
};

pub(crate) static OBIS_2_8_0: ObisKeyFigure = ObisKeyFigure {
    exact: &[1, 0, 2, 8, 0, 255],
    simplified: "2.8.0",
    unit: Unit::Kwh,
};

// Map with multiple notations pointing to the same instances
pub static OBIS_KEY_FIGURES: phf::Map<&'static str, &'static ObisKeyFigure> = phf_map! {
    // Energy consumed (1.8.0) - all notations
    "1.8.0" => &OBIS_1_8_0,
    "1-0:1.8.0" => &OBIS_1_8_0,
    "1-0:1.8.0*255" => &OBIS_1_8_0,

    // Energy produced (2.8.0) - all notations
    "2.8.0" => &OBIS_2_8_0,
    "1-0:2.8.0" => &OBIS_2_8_0,
    "1-0:2.8.0*255" => &OBIS_2_8_0,
};

pub fn serialize<S>(value: &ObisKeyFigure, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(value.simplified)
}

pub fn deserialize<'de, D>(deserializer: D) -> Result<&'static ObisKeyFigure, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    OBIS_KEY_FIGURES
        .get(&s as &str)
        .copied()
        .ok_or_else(|| serde::de::Error::custom(format!("Unknown OBIS code: {s}")))
}
