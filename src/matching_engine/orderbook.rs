use std::collections::HashMap;

#[derive(Debug)]
pub enum BidOrAsk {
    Bid,
    Ask,
}
#[derive(Debug)]
pub struct Orderbook {
    asks: HashMap<Price, Limit>,
    bids: HashMap<Price, Limit>,
}

impl Orderbook {
    pub fn new() -> Orderbook {
        Orderbook {
            asks: HashMap::new(),
            bids: HashMap::new(),
        }
    }

    pub fn add_order(&mut self, price: f64, order: Order) {
        let price = Price::new(price);

        match order.bid_or_ask {
            BidOrAsk::Bid => {
                let _limit = self.bids.get_mut(&price);

                match _limit {
                    Some(_limit) => _limit.add_order(order),
                    None => {
                        let mut _limit = Limit::new(price);
                        _limit.add_order(order);
                        self.bids.insert(price, _limit);
                    }
                }
            }
            BidOrAsk::Ask => {
                let _limit = self.asks.get_mut(&price);

                match _limit {
                    Some(_limit) => _limit.add_order(order),
                    None => {
                        let mut _limit = Limit::new(price);
                        _limit.add_order(order);
                        self.asks.insert(price, _limit);
                    }
                }
            }
        }
    }
}

#[derive(Debug, Eq, Hash, PartialEq, Copy, Clone)]
pub struct Price {
    integral: u64,
    fractional: u64,
    scalar: u64,
}

impl Price {
    fn new(price: f64) -> Price {
        let scalar = 100000;
        let integral = price as u64;
        let fractional = ((price % 1.0) * scalar as f64) as u64;
        Price {
            scalar,
            integral,
            fractional,
        }
    }
}

#[derive(Debug)]
pub struct Limit {
    price: Price,
    orders: Vec<Order>,
}

impl Limit {
    fn new(price: Price) -> Limit {
        Limit {
            price: price,
            orders: Vec::new(),
        }
    }

    fn add_order(&mut self, order: Order) {
        self.orders.push(order);
    }
}

#[derive(Debug)]
pub struct Order {
    size: f64,
    bid_or_ask: BidOrAsk,
}

impl Order {
    pub fn new(bid_or_ask: BidOrAsk, size: f64) -> Order {
        Order { bid_or_ask, size }
    }
}
