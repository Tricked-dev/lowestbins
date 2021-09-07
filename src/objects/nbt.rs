use std::collections::HashMap;

#[derive(Deserialize)]
pub struct PartialNbt {
    pub i: Vec<PartialNbtElement>,
}

#[derive(Deserialize)]
pub struct PartialNbtElement {
    #[serde(rename = "Count")]
    pub count: i8,
    pub tag: PartialTag,
}

#[derive(Deserialize)]
pub struct PartialTag {
    #[serde(rename = "ExtraAttributes")]
    pub extra_attributes: PartialExtraAttr,
    pub display: DisplayInfo,
}

#[derive(Deserialize)]
pub struct PartialExtraAttr {
    pub id: String,
    pub enchantments: Option<HashMap<String, i32>>,
    pub anvil_uses: Option<i8>,
}

#[derive(Deserialize)]
pub struct DisplayInfo {
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "Lore")]
    pub lore: Vec<String>,
}
