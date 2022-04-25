use serde::Deserialize;
pub use std::collections::HashMap;

#[derive(Debug, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum OrderContents {
    Change(String),
    Price(f64),
    //price and quantity have the same data type so they both deserialize as price
    //Quantity(f64),
}

#[derive(Deserialize)]
pub struct FirstNotification {
    pub jsonrpc: String,
    pub method: String,
    pub params: ParamsSnapshot,
}
#[derive(Deserialize)]
pub struct Notification {
    pub jsonrpc: String,
    pub method: String,
    pub params: ParamsChange,
}
#[derive(Debug)]
pub struct Book {
    pub change_id: u64,
    pub highest_bid: f64,
    pub highest_quantity: f64,
    pub lowest_ask: f64,
    pub lowest_quantity: f64,
    //f64 does not implement either Eq or Hash traits so we use strings instead instead of using
    //a struct that holds the integer and the floating point
    pub bids: HashMap<String, String>,
    pub asks: HashMap<String, String>,
}

#[derive(Deserialize)]
pub struct ParamsSnapshot {
    pub channel: String,
    pub data: DataSnapshot,
}
//because of prev_change_id and because there isn't a skip_deserialization_if we just make another struct identical to the above+ the extra field
#[derive(Deserialize)]
pub struct ParamsChange {
    pub channel: String,
    pub data: DataChange,
}

#[derive(Deserialize)]
pub struct DataSnapshot {
    #[serde(rename = "type")]
    pub _type: String,
    pub timestamp: u64,
    pub instrument_name: String,
    pub change_id: u64,
    pub bids: Vec<Vec<OrderContents>>,
    pub asks: Vec<Vec<OrderContents>>,
}
#[derive(Deserialize)]
pub struct DataChange {
    #[serde(rename = "type")]
    pub _type: String,
    pub timestamp: u64,
    pub prev_change_id: u64,
    pub instrument_name: String,
    pub change_id: u64,
    pub bids: Vec<Vec<OrderContents>>,
    pub asks: Vec<Vec<OrderContents>>,
}
