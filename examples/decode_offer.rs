fn main() {
    let raw = include_str!("offer.flexibility.json");
    let envelope =
        openvet::Envelope::from_slice(raw.as_bytes()).expect("example offer should parse");
    println!("{}", envelope.summary());

    let openvet::Message::OfferFlexibility(offer) = &envelope.msg else {
        panic!("example file is an offer");
    };
    println!(
        "if this revision is accepted, the counterparty pays {} {} per kWh",
        offer.terms.price.format_major(),
        offer.terms.price.currency
    );
}
