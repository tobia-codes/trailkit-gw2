//! Gem exchange quotes at the quantities the game's exchange UI offers,
//! used as the amount paid (data-pipeline D31).

use anyhow::{Context, Result, bail};
use serde::Serialize;
use serde_json::value::RawValue;

use crate::{
    compact_json::compact,
    gw2_api::{ApiClient, ApiResponse},
};

/// Gems paid per "Get Gold" quote (`/v2/commerce/exchange/gems`).
const GEMS_PAID: [u64; 4] = [400, 800, 1200, 2000];

/// Gold paid per "Get Gems" quote (`/v2/commerce/exchange/coins`).
const GOLD_PAID: [u64; 5] = [1, 10, 50, 100, 250];

/// Copper coins per gold coin.
const COPPER_PER_GOLD: u64 = 10_000;

/// One quote with the raw API answer (`coins_per_gem`, `quantity`).
#[derive(Debug, Serialize)]
struct Quote {
    /// Amount paid: gems for `gems_to_coins`, copper for `coins_to_gems`.
    paid: u64,
    /// The unmodified API response object.
    response: Box<RawValue>,
}

/// Content of `<date>.gem-exchange.json.gz`.
#[derive(Debug, Serialize)]
struct GemExchangeSnapshot {
    /// Paying gems to get gold ("Get Gold" in game).
    gems_to_coins: Vec<Quote>,
    /// Paying copper to get gems ("Get Gems" in game).
    coins_to_gems: Vec<Quote>,
}

/// Fetches all quotes and returns the snapshot as one compact JSON document.
/// All-or-nothing: 9 small requests with retries each.
///
/// # Errors
/// Returns an error if any quote cannot be fetched or parsed.
pub fn fetch(client: &mut ApiClient) -> Result<String> {
    let mut gems_to_coins = Vec::new();
    for gems in GEMS_PAID {
        gems_to_coins.push(fetch_quote(client, "gems", gems)?);
    }
    let mut coins_to_gems = Vec::new();
    for gold in GOLD_PAID {
        coins_to_gems.push(fetch_quote(client, "coins", gold * COPPER_PER_GOLD)?);
    }
    let snapshot = GemExchangeSnapshot {
        gems_to_coins,
        coins_to_gems,
    };
    Ok(serde_json::to_string(&snapshot)?)
}

/// Fetches one quote from `/v2/commerce/exchange/<currency>?quantity=<paid>`.
///
/// # Errors
/// Returns an error if the request fails or the body is not JSON.
fn fetch_quote(client: &mut ApiClient, currency: &str, paid: u64) -> Result<Quote> {
    let path = format!("/v2/commerce/exchange/{currency}?quantity={paid}");
    let ApiResponse::Found(body) = client.get(&path)? else {
        bail!("GET {path}: 404");
    };
    let response =
        RawValue::from_string(compact(&body)).with_context(|| format!("parsing {path}"))?;
    Ok(Quote { paid, response })
}
