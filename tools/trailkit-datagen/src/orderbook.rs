//! Order book condensation (data-pipeline D17): per item and side the best 5
//! price tiers plus market depth buckets, instead of the full order book
//! (~19.5 MB/day gzip → ~1.9 MB/day).

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

/// Number of best price tiers kept per side.
const BEST_TIER_COUNT: usize = 5;

/// One price tier of `/v2/commerce/listings` (also stored unchanged).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tier {
    /// Number of individual listings at this price.
    pub listings: u32,
    /// Price per unit in copper.
    pub unit_price: u32,
    /// Total units offered at this price.
    pub quantity: u32,
}

/// One item of `/v2/commerce/listings`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Listing {
    /// Item id.
    pub id: u32,
    /// Buy orders.
    pub buys: Vec<Tier>,
    /// Sell offers.
    pub sells: Vec<Tier>,
}

/// Which side of the order book a tier list belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Side {
    /// Buy orders: the best price is the highest.
    Buy,
    /// Sell offers: the best price is the lowest.
    Sell,
}

/// Total quantity offered within a distance from the best price.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct Depth {
    /// Within 1 % of the best price.
    pub within_1_percent: u64,
    /// Within 5 % of the best price.
    pub within_5_percent: u64,
    /// Within 10 % of the best price.
    pub within_10_percent: u64,
}

/// Condensed view of one side of an item's order book.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CondensedSide {
    /// Up to [`BEST_TIER_COUNT`] best tiers, best first.
    pub best_tiers: Vec<Tier>,
    /// Number of price tiers on this side.
    pub tier_count: u32,
    /// Sum of all listings on this side.
    pub total_listings: u64,
    /// Sum of all quantities on this side.
    pub total_quantity: u64,
    /// Quantity near the best price.
    pub depth: Depth,
}

/// One line of `<date>.orderbook.ndjson.gz`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CondensedListing {
    /// Item id.
    pub id: u32,
    /// Condensed buy orders.
    pub buys: CondensedSide,
    /// Condensed sell offers.
    pub sells: CondensedSide,
}

impl Listing {
    /// Condenses both sides of this item's order book.
    #[must_use]
    pub fn condense(&self) -> CondensedListing {
        CondensedListing {
            id: self.id,
            buys: condense_side(&self.buys, Side::Buy),
            sells: condense_side(&self.sells, Side::Sell),
        }
    }
}

/// Parses one `/v2/commerce/listings` response body and returns one compact
/// JSON line per item.
///
/// # Errors
/// Returns an error if the body is not a valid listings array.
pub fn condense_response(body: &str) -> Result<Vec<String>> {
    let listings: Vec<Listing> =
        serde_json::from_str(body).context("parsing /v2/commerce/listings")?;
    listings
        .iter()
        .map(|listing| serde_json::to_string(&listing.condense()).map_err(Into::into))
        .collect()
}

/// Condenses one side; does not rely on the API's sort order.
fn condense_side(tiers: &[Tier], side: Side) -> CondensedSide {
    let mut sorted = tiers.to_vec();
    match side {
        Side::Buy => sorted.sort_by_key(|tier| std::cmp::Reverse(tier.unit_price)),
        Side::Sell => sorted.sort_by_key(|tier| tier.unit_price),
    }
    let best_price = sorted.first().map(|tier| tier.unit_price);
    let depth_within = |percent: u64| -> u64 {
        best_price.map_or(0, |best_price| {
            sorted
                .iter()
                .filter(|tier| is_within(tier.unit_price, best_price, percent, side))
                .map(|tier| u64::from(tier.quantity))
                .sum()
        })
    };
    CondensedSide {
        best_tiers: sorted.iter().take(BEST_TIER_COUNT).copied().collect(),
        tier_count: u32::try_from(sorted.len()).unwrap_or(u32::MAX),
        total_listings: sorted.iter().map(|tier| u64::from(tier.listings)).sum(),
        total_quantity: sorted.iter().map(|tier| u64::from(tier.quantity)).sum(),
        depth: Depth {
            within_1_percent: depth_within(1),
            within_5_percent: depth_within(5),
            within_10_percent: depth_within(10),
        },
    }
}

/// Whether `price` is at most `percent` % worse than `best_price` (integer math).
fn is_within(price: u32, best_price: u32, percent: u64, side: Side) -> bool {
    let price = u64::from(price) * 100;
    let best_price = u64::from(best_price);
    match side {
        Side::Buy => price >= best_price * (100 - percent),
        Side::Sell => price <= best_price * (100 + percent),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Parses the committed fixture.
    fn sample() -> Result<Vec<Listing>> {
        Ok(serde_json::from_str(include_str!(
            "../fixtures/listings-sample.json"
        ))?)
    }

    #[test]
    fn condenses_buy_side() -> Result<()> {
        let buys = sample()?[0].condense().buys;
        let prices: Vec<u32> = buys.best_tiers.iter().map(|tier| tier.unit_price).collect();
        assert_eq!(prices, [1000, 995, 990, 950, 900]);
        assert_eq!(buys.tier_count, 6);
        assert_eq!(buys.total_listings, 14);
        assert_eq!(buys.total_quantity, 1250);
        assert_eq!(
            buys.depth,
            Depth {
                within_1_percent: 130,
                within_5_percent: 170,
                within_10_percent: 250
            }
        );
        Ok(())
    }

    #[test]
    fn condenses_sell_side() -> Result<()> {
        let sells = sample()?[0].condense().sells;
        let prices: Vec<u32> = sells
            .best_tiers
            .iter()
            .map(|tier| tier.unit_price)
            .collect();
        assert_eq!(prices, [1100, 1110, 1155, 1210, 1300]);
        assert_eq!(sells.total_quantity, 83);
        assert_eq!(
            sells.depth,
            Depth {
                within_1_percent: 55,
                within_5_percent: 62,
                within_10_percent: 71
            }
        );
        Ok(())
    }

    #[test]
    fn empty_side_has_zero_depth() -> Result<()> {
        let buys = sample()?[1].condense().buys;
        assert!(buys.best_tiers.is_empty());
        assert_eq!(buys.total_quantity, 0);
        assert_eq!(buys.depth.within_10_percent, 0);
        Ok(())
    }

    #[test]
    fn response_becomes_one_line_per_item() -> Result<()> {
        let lines = condense_response(include_str!("../fixtures/listings-sample.json"))?;
        assert_eq!(lines.len(), 2);
        assert!(lines.iter().all(|line| !line.contains('\n')));
        Ok(())
    }
}
