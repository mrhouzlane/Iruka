mod matching_engine;
use matching_engine::engine::MatchingEngine;
use matching_engine::orderbook::{BidOrAsk, Order, Orderbook};
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

    let engine = MatchingEngine::new();
}
