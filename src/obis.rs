use phf::phf_map;
use serde::{Deserialize, Serialize};
use serde::{Deserializer, Serializer};
use sml_rs::parser::OctetStr;

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, strum::AsRefStr)]
#[serde(rename_all = "lowercase")]
pub enum Unit {
    #[strum(to_string = "W")]
    W,
    #[strum(to_string = "Wh")]
    Wh,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ObisKeyFigure {
    pub exact: OctetStr<'static>,
    pub simplified: &'static str,
    pub unit: Unit,
    /// expected DLMS/COSEM unit according to IEC 62056-6-2 <https://www.nema.org/docs/default-source/standards-document-library/c12-iec62056-6-2-ed3-contents-and-scope.pdf?sfvrsn=8cf9aa7e_0>
    pub unit_id: u8,
}

// Static instances for each OBIS key figure
pub(crate) static OBIS_1_8_0: ObisKeyFigure = ObisKeyFigure {
    exact: &[1, 0, 1, 8, 0, 255],
    simplified: "1.8.0",
    unit: Unit::Wh,
    unit_id: 30,
};

pub(crate) static OBIS_2_8_0: ObisKeyFigure = ObisKeyFigure {
    exact: &[1, 0, 2, 8, 0, 255],
    simplified: "2.8.0",
    unit: Unit::Wh,
    unit_id: 30,
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
