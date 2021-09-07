use crate::objects::items::Item;
use crate::objects::profile::PartialProfile;

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct Claim {
    pub claimed: bool,
    // pub claimed_bidders: Vec<String>, // TODO
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
#[serde(transparent)]
pub struct PartialAuction(pub String);

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct Auction {
    /// Hypixel's auction UUID, this can be utilized with the in-game command
    /// `/viewauction <uuid>`.
    pub uuid: PartialAuction,
    /// A partial Skyblock profile, it only contains the UUID but can be upgraded through API calls. (TODO)
    pub auctioneer: PartialProfile,
    /// A list of co-op members in the auctioneer's Skyblock profile
    pub coop: Vec<PartialProfile>,
    /// Unix time (in milliseconds) of when the auction commenced.
    pub start: i64,
    /// Unix time (in milliseconds) of when the auction is currently projected to end.
    /// This is only an estimate as it does not account for last-2-minute bids extending the end time.
    pub end: i64,
    /// Fields pertaining to the item itself.
    #[serde(flatten)]
    pub item: Item,
    /// Fields pertaining to the bids on the item
    #[serde(flatten)]
    pub bids: Bids,
    /// Fields related to whether the auction has been claimed
    /// This does not appear to function correctly.
    #[serde(flatten)]
    pub claim: Claim,
}

/// A collection of bidding data.
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct Bids {
    /// The current highest price on the auction item
    #[serde(rename = "highest_bid_amount")]
    pub highest: i64,
    /// The starting price of the auction item
    #[serde(rename = "starting_bid")]
    pub starting: i64,
    /// The list of bids on the item
    pub bids: Vec<Bid>,
}

/// A discrete bid on an item.
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct Bid {
    /// The auction which this bid belongs to.
    pub auction_id: PartialAuction,
    /// The profile-member that had placed a bid on the auction.
    pub bidder: PartialProfile,
    /// The amount of coins in the bid
    pub amount: i64,
    /// The time of the bid
    pub timestamp: i64,
}

/// A page of auctions retrieved by the `skyblock/auctions` endpoint.
#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone)]
pub struct GlobalAuctions {
    /// The current page fetched
    pub page: usize,
    /// The total number of pages in the system
    #[serde(rename = "totalPages")]
    pub total_pages: usize,
    /// The total number of auctions in the system
    #[serde(rename = "totalAuctions")]
    pub total_auctions: usize,
    /// The timestamp of the last update that has occurred in the dataset
    #[serde(rename = "lastUpdated")]
    pub last_update: u64,
    /// The list of auctions retrieved
    pub auctions: Vec<Auction>,
}

/// An auction that had been searched by UUID.
#[cfg(test)]
#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone)]
pub struct SearchedAuctions {
    pub auctions: Vec<Auction>,
}
