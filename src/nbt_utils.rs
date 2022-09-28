use nbt::from_gzip_reader;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, io, result::Result as StdResult};

#[derive(Deserialize)]
pub struct PartialNbt {
    pub i: Vec<PartialNbtElement>,
}

#[derive(Deserialize)]
pub struct PartialNbtElement {
    pub tag: PartialTag,
}

#[derive(Deserialize)]
pub struct PartialTag {
    #[serde(rename = "ExtraAttributes")]
    pub extra_attributes: PartialExtraAttr,
}

#[derive(Serialize, Deserialize)]
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

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct Item {
    #[serde(rename = "item_name")]
    pub name: String,
    /// The count of items in the stack
    #[serde(rename = "item_count", default = "one")]
    pub count: u8,
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
fn one() -> u8 {
    1
}
impl Item {
    pub fn to_nbt(&self) -> Result<PartialNbt, Box<dyn std::error::Error>> {
        let bytes: StdResult<Vec<u8>, _> = self.bytes.clone().into();
        let nbt: PartialNbt = from_gzip_reader(io::Cursor::new(bytes?))?;
        Ok(nbt)
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
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
impl From<ItemBytes> for Result<Vec<u8>, Box<dyn std::error::Error>> {
    fn from(s: ItemBytes) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let b64: String = s.into();
        Ok(base64::decode(&b64)?)
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
#[serde(tag = "type", content = "data")]
pub enum ItemBytesT0 {
    #[serde(rename = "0")]
    Data(String),
}
