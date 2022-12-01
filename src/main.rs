use std::collections::HashMap;

#[derive(Debug)]
enum BidOrAsk {
    Bid,
    Ask,
}
#[derive(Debug)]
struct Orderbook {
    asks: HashMap<Price, Limit>,
    bids: HashMap<Price, Limit>,
}

impl Orderbook {
    fn new() -> Orderbook {
        Orderbook {
            asks: HashMap::new(),
            bids: HashMap::new(),
        }
    }

    fn add_order(&mut self, price: f64, order: Order) {
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
struct Price {
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
struct Limit {
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
struct Order {
    size: f64,
    bid_or_ask: BidOrAsk,
}

impl Order {
    fn new(bid_or_ask: BidOrAsk, size: f64) -> Order {
        Order { bid_or_ask, size }
    }
}

fn main() {
    let buy_order_from_mido = Order::new(BidOrAsk::Bid, 5.5);
    let buy_order_from_mehdi = Order::new(BidOrAsk::Bid, 4.4);
    // let sell_order = Order::new(BidOrAsk::Ask, 2.2);

    let mut orderbook = Orderbook::new();
    orderbook.add_order(50.0, buy_order_from_mido);
    orderbook.add_order(50.0, buy_order_from_mehdi);

    let sell_order = Order::new(BidOrAsk::Ask, 6.5);
    orderbook.add_order(20.0, sell_order);

    println!("{:?}", orderbook);
}
