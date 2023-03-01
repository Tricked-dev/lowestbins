use nbt::from_gzip_reader;
use serde::Deserialize;

use std::{collections::HashMap, io, result::Result as StdResult};

use crate::error::Result;

#[derive(Deserialize)]
pub struct PartialNbt {
    pub i: Vec<PartialNbtElement>,
}

#[derive(Deserialize)]
pub struct PartialNbtElement {
    #[serde(rename = "Count")]
    pub count: u8,
    pub tag: PartialTag,
}

#[derive(Deserialize)]
pub struct PartialTag {
    #[serde(rename = "ExtraAttributes")]
    pub extra_attributes: PartialExtraAttr,
}

#[derive(Deserialize)]
pub struct Pet {
    #[serde(rename = "type")]
    pub pet_type: String,

    #[serde(rename = "tier")]
    pub tier: String,
}

#[derive(Deserialize)]
pub struct PartialExtraAttr {
    pub id: String,
    #[serde(rename = "petInfo")]
    pub pet: Option<String>,
    pub enchantments: Option<HashMap<String, u8>>,
    pub potion: Option<String>,
    pub potion_level: Option<u8>,
    #[serde(default = "bool_false")]
    pub enhanced: bool,
    pub runes: Option<HashMap<String, u8>>,
    pub attributes: Option<HashMap<String, u8>>,
}

#[derive(Deserialize)]
pub struct DisplayInfo {
    #[serde(rename = "Name")]
    pub name: String,
}

#[derive(Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct Item {
    #[serde(rename = "item_name")]
    pub name: String,
    /// The item's gzipped NBT representation
    #[serde(rename = "item_bytes")]
    pub bytes: ItemBytes,
    #[serde(rename = "starting_bid")]
    pub starting_bid: u64,
    #[serde(rename = "bin", default = "bool_false")]
    pub bin: bool,
}
fn bool_false() -> bool {
    false
}
impl Item {
    pub fn to_nbt(&self) -> Result<PartialNbt> {
        let bytes: StdResult<Vec<u8>, _> = self.bytes.clone().into();
        let nbt: PartialNbt = from_gzip_reader(io::Cursor::new(bytes?))?;
        Ok(nbt)
    }
}

#[derive(Deserialize, Debug, PartialEq, Eq, Clone)]
#[serde(untagged)]
pub enum ItemBytes {
    T0(ItemBytesT0),
    Data(String),
}

impl From<ItemBytes> for String {
    fn from(s: ItemBytes) -> String {
        match s {
            ItemBytes::T0(ibt0) => {
                let ItemBytesT0::Data(x) = ibt0;
                x
            }
            ItemBytes::Data(x) => x,
        }
    }
}
impl From<ItemBytes> for Result<Vec<u8>> {
    fn from(s: ItemBytes) -> Result<Vec<u8>> {
        let b64: String = s.into();
        Ok(base64::decode(b64)?)
    }
}

#[derive(Deserialize, Debug, PartialEq, Eq, Clone)]
#[serde(tag = "type", content = "data")]
pub enum ItemBytesT0 {
    #[serde(rename = "0")]
    Data(String),
}
