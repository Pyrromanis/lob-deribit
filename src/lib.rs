use itertools::izip;
use serde::Deserialize;
use serde_json::Result;
use std::collections::HashMap;

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
//the most important struct. The book contains the pairs of values we need to print and the hashmaps
//of all the bids and asks, all other structs are used for the json deserialization
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
//We clear the snapshot in case of packet loss. Primarily was used for testing
pub fn clear_snapshot(book: &mut Book) -> &mut Book {
    println!("Packet Loss! Reconnecting!");
    book.bids.clear();
    book.asks.clear();
    book.change_id = 0;
    book.highest_bid = 0.0;
    book.highest_quantity = 0.0;
    book.lowest_ask = f64::MAX;
    book.lowest_quantity = 0.0;
    book
}
//in case of something going wrong we return an empty snapshots, as we check later if the snapshot
//is empty to know if something went wrong
pub fn empty_snapshot() -> Result<Book> {
    let book: Book = Book {
        change_id: 0,
        highest_bid: 0.0,
        highest_quantity: 0.0,
        lowest_ask: 0.0,
        lowest_quantity: 0.0,
        bids: Default::default(),
        asks: Default::default(),
    };
    Ok(book)
}

pub fn empty_change() -> Result<Notification> {
    let change: Notification = Notification {
        jsonrpc: "".to_string(),
        method: "".to_string(),
        params: ParamsChange {
            channel: "".to_string(),
            data: DataChange {
                _type: "".to_string(),
                timestamp: 0,
                prev_change_id: 0,
                instrument_name: "".to_string(),
                change_id: 0,
                bids: vec![],
                asks: vec![],
            },
        },
    };
    Ok(change)
}
//initialize the Book fill it
pub fn get_snapshot(data: &str) -> Result<Book> {
    let snapshot: FirstNotification = serde_json::from_str(data)?;
    let mut book: Book = Book {
        change_id: snapshot.params.data.change_id,
        highest_bid: 0.0,
        highest_quantity: 0.0,
        lowest_ask: f64::MAX,
        lowest_quantity: 0.0,
        bids: Default::default(),
        asks: HashMap::new(),
    };
    for (i, j) in izip!(&snapshot.params.data.bids, &snapshot.params.data.bids) {
        if let OrderContents::Price(price) = &i[1] {
            if let OrderContents::Price(quantity) = &j[2] {
                if price > &book.highest_bid {
                    book.highest_quantity = *quantity;
                    book.highest_bid = *price;
                }
                book.bids.insert(price.to_string(), quantity.to_string());
            }
        }
    }
    for (i, j) in izip!(&snapshot.params.data.asks, &snapshot.params.data.asks) {
        if let OrderContents::Price(price) = &i[1] {
            if let OrderContents::Price(quantity) = &j[2] {
                if price < &book.lowest_ask {
                    book.lowest_ask = *price;
                    book.lowest_quantity = *quantity;
                }

                book.asks.insert(price.to_string(), quantity.to_string());
            }
        }
    }
    Ok(book)
}
pub fn get_change(data: &str) -> Result<Notification> {
    let change: Notification = serde_json::from_str(data)?;
    Ok(change)
}
//the important function. Check for packet loss and if encountered reconnect. if not iterate the
//bids asks and perform the asked transformations. Check at each new/change if one of the important
//values needs change and use a flag to specify if we have deleted one of them and at the end of
//performing all the changes, so that we pay the O(n) only once at the end. Obviously also perform
// the transformations.
pub fn update_book(mut book: &mut Book, c_data: DataChange) -> Result<u32> {
    //flags if we deleted an importnat value and need to update it at the end of the function
    let mut del_hbid: bool = false;
    let mut del_lask: bool = false;
    if (c_data.prev_change_id) != book.change_id {
        //propagate packet loss
        clear_snapshot(&mut book);
        return Ok(1);
    }
    //izip helps alot for creating an iterator for change price and quantity in parallel
    book.change_id = c_data.change_id;
    for (i, j, k) in izip!(&c_data.bids, &c_data.bids, &c_data.bids) {
        if let OrderContents::Change(action) = &i[0] {
            if let OrderContents::Price(price) = &j[1] {
                match action.as_str() {
                    "delete" => {
                        book.bids.remove_entry(&price.to_string());
                        if book.highest_bid == *price {
                            book.highest_bid = 0.0;
                            book.highest_quantity = 0.0;
                            del_hbid = true;
                        }
                    }
                    //if its not delete, we use insert that changes or creates a value if it doesnt
                    //exist already. We also start iterating for the quantity now since it wasn't
                    //needed for the delete
                    _ => {
                        if let OrderContents::Price(quantity) = &k[2] {
                            if price >= &book.highest_bid {
                                book.highest_bid = *price;
                                book.highest_quantity = *quantity;
                            }
                            book.bids.insert(price.to_string(), quantity.to_string());
                        }
                    }
                }
            }
        }
    }

    for (i, j, k) in izip!(&c_data.asks, &c_data.asks, &c_data.asks) {
        if let OrderContents::Change(action) = &i[0] {
            if let OrderContents::Price(price) = &j[1] {
                match action.as_str() {
                    "delete" => {
                        book.asks.remove_entry(&price.to_string());
                        if book.lowest_ask == *price {
                            book.lowest_ask = f64::MAX;
                            book.lowest_quantity = 0.0;
                            del_lask = true;
                        }
                    }
                    _ => {
                        if let OrderContents::Price(quantity) = &k[2] {
                            if price <= &book.lowest_ask {
                                book.lowest_ask = *price;
                                book.lowest_quantity = *quantity;
                            }
                            book.asks.insert(price.to_string(), quantity.to_string());
                        }
                    }
                }
            }
        }
    }
    //if we deleted traverse the hashmap and find the new pair. As mentioned above we use stings
    //for the values as floats are problematics so we need the double parse here
    if del_hbid {
        for (i, j) in &book.bids {
            let i_val: f64 = i.parse::<f64>().unwrap();
            if i_val > book.highest_bid {
                book.highest_bid = i_val;
                book.highest_quantity = j.parse::<f64>().unwrap();
            }
        }
    }
    if del_lask {
        for (i, j) in &book.asks {
            let i_val: f64 = i.parse::<f64>().unwrap();
            if i_val < book.lowest_ask {
                book.lowest_ask = i_val;
                book.lowest_quantity = j.parse::<f64>().unwrap();
            }
        }
    }
    Ok(0)
}
