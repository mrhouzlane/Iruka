use super::orderbook::Orderbook;
use std::collections::HashMap;

// BTCUSD
// BTC => BASE
// USD => QUOTE

#[derive(Eq, Hash, PartialEq, Clone)]
pub struct TradingPair {
    base: String,
    quote: String,
}

impl TradingPair {
    pub fn new(base: String, quote: String) -> TradingPair {
        TradingPair { base, quote }
    }
}

pub struct MatchingEngine {
    orderbooks: HashMap<TradingPair, Orderbook>,
}

impl MatchingEngine {
    pub fn new() -> MatchingEngine {
        MatchingEngine {
            orderbooks: HashMap::new(),
        }
    }

    pub fn add_new_market(&mut self, pair: TradingPair) {
        self.orderbooks.insert(pair, Orderbook::new());
        println!("opening new orderbook")
    }
}
