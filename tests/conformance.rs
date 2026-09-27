//! Executable checks for `docs/spec/07-conformance.md`.
//!
//! The test at the bottom fails if an id exists only in the spec or only here.

use openvet::{
    check_claim, clamp_power, effective_bounds, Baseline, BaselineKind, DealEvent, DealState,
    EnergyBounds, Envelope, ErrorCode, Message, MeterInterval, Name, OfferFlexibility, OfferTerms,
    PartyId, PeerCaps, PoolBook, Price, PricePer, Product, ProtocolError, SettlementClaim,
    Timestamp, WattBounds,
};
use std::collections::BTreeSet;

fn parse(json: &str) -> Result<Envelope, ProtocolError> {
    Envelope::from_slice(json.as_bytes())
}

fn must(json: &str) -> Envelope {
    parse(json).unwrap_or_else(|err| panic!("{err}\n{json}"))
}

fn code_of(json: &str) -> ErrorCode {
    parse(json).expect_err("expected rejection").code
}

fn example(name: &str) -> Envelope {
    let path = format!("{}/examples/{name}", env!("CARGO_MANIFEST_DIR"));
    let text = std::fs::read_to_string(&path).unwrap_or_else(|err| panic!("{path}: {err}"));
    Envelope::from_slice(text.as_bytes()).unwrap_or_else(|err| panic!("{name}: {err}"))
}

fn party(text: &str) -> PartyId {
    PartyId::parse(text).unwrap()
}

fn nm(text: &str) -> Name {
    Name::parse(text).unwrap()
}

fn ts(text: &str) -> Timestamp {
    Timestamp::parse(text).unwrap()
}

fn csms() -> PartyId {
    party("csms:example-municipal")
}

fn agg() -> PartyId {
    party("agg:example-flex")
}

fn terms_at(expires: &str) -> OfferTerms {
    OfferTerms {
        product: Product::ChargeShift,
        pool_id: nm("depot-7"),
        window: openvet::Interval {
            start: ts("2026-09-26T18:00:00Z"),
            end: ts("2026-09-26T20:00:00Z"),
        },
        power_w: WattBounds {
            min: -50_000,
            max: 150_000,
        },
        energy_wh: EnergyBounds {
            min: 0,
            max: 400_000,
        },
        price: Price {
            currency: "EUR".to_string(),
            per: PricePer::Kwh,
            scale: -4,
            amount: 1500,
        },
        baseline: Baseline {
            kind: BaselineKind::UnmanagedCharge,
            ref_id: None,
        },
        executor: csms(),
        expires_at: ts(expires),
    }
}

fn offer(id: &str) -> OfferFlexibility {
    OfferFlexibility {
        offer_id: nm(id),
        revision: 1,
        terms: terms_at("2026-09-26T17:30:00Z"),
    }
}

fn open_new(body: &OfferFlexibility) -> Result<(), ProtocolError> {
    let mut book = PoolBook::new();
    book.open(&csms(), &agg(), body, ts("2026-09-26T16:00:02Z"))
}

fn opened(id: &str) -> PoolBook {
    let mut book = PoolBook::new();
    let body = offer(id);
    book.open(&csms(), &agg(), &body, ts("2026-09-26T16:00:02Z"))
        .unwrap();
    book
}

fn accept(book: &mut PoolBook, id: &str) {
    book.apply(
        &nm(id),
        DealEvent::Accept {
            actor: agg(),
            revision: 1,
            now: ts("2026-09-26T16:05:00Z"),
        },
    )
    .unwrap();
}

fn status(extra: &str) -> String {
    format!(
        r#"{{"v":"0.1","id":"m-1","type":"status.pool","ts":"2026-09-26T16:00:00Z","from":"csms:example-municipal","to":"agg:example-flex"{extra},"body":{{"pool_id":"depot-7","observed_at":"2026-09-26T15:59:00Z","vehicles_connected":1,"power_w":0,"energy_capacity_wh":1000,"energy_stored_wh":100,"import_available_w":1000,"export_available_w":0}}}}"#
    )
}

#[test]
fn envelope_round_trip_and_garbage() {
    // OV-ENV-01
    let env = example("offer.flexibility.json");
    let again = Envelope::from_slice(&env.to_vec().unwrap()).unwrap();
    assert_eq!(env, again);
    assert_eq!(code_of("{"), ErrorCode::Malformed);
    assert_eq!(code_of("[]"), ErrorCode::Malformed);
    assert_eq!(code_of(""), ErrorCode::Malformed);

    let mut raw = std::fs::read_to_string(format!(
        "{}/examples/offer.flexibility.json",
        env!("CARGO_MANIFEST_DIR")
    ))
    .unwrap();
    raw = raw.replacen("\"offer_id\"", "\"note\": true, \"offer_id\"", 1);
    assert_eq!(
        Envelope::from_slice(raw.as_bytes()).unwrap_err().code,
        ErrorCode::Malformed
    );
}

#[test]
fn timestamps_are_canonical() {
    // OV-ENV-02
    assert!(parse(&status("")).is_ok());
    for bad in [
        "2026-09-26T16:00:00+00:00",
        "2026-09-26T18:00:00+02:00",
        "2026-09-26T16:00:00.000Z",
        "2026-02-29T00:00:00Z",
        "2100-02-29T00:00:00Z",
    ] {
        let json = status("").replacen("2026-09-26T16:00:00Z", bad, 1);
        assert_eq!(code_of(&json), ErrorCode::Malformed, "{bad}");
    }
}

#[test]
fn version_is_the_draft_string() {
    // OV-ENV-03
    for version in ["0.2", "0.1.0", "1.0"] {
        let json = status("").replacen("\"0.1\"", &format!("\"{version}\""), 1);
        assert_eq!(code_of(&json), ErrorCode::UnsupportedVersion, "{version}");
    }
}

#[test]
fn correlation_is_required_only_where_the_spec_says() {
    // OV-ENV-04
    assert_eq!(
        code_of(&status(r#","corr":"m-earlier""#)),
        ErrorCode::Malformed
    );
    let accept = r#"{"v":"0.1","id":"m-accept","type":"offer.accept","ts":"2026-09-26T16:05:00Z","from":"agg:example-flex","to":"csms:example-municipal","body":{"offer_id":"off-1001","revision":1}}"#;
    assert_eq!(code_of(accept), ErrorCode::Malformed);
    let accept_ok = accept.replacen("\"body\"", "\"corr\":\"m-offer\",\"body\"", 1);
    assert!(parse(&accept_ok).is_ok());
    let err = r#"{"v":"0.1","id":"m-err","type":"error","ts":"2026-09-26T16:05:00Z","from":"agg:example-flex","to":"csms:example-municipal","body":{"code":"profile_unsupported","detail":"peer has basic only"}}"#;
    assert_eq!(code_of(err), ErrorCode::Malformed);
    assert!(parse(&err.replacen("\"body\"", "\"corr\":\"m-offer\",\"body\"", 1)).is_ok());
}

#[test]
fn rejects_oversized_payloads() {
    // OV-ENV-05
    let huge = vec![b' '; openvet::limits::MAX_MESSAGE_BYTES + 1];
    assert_eq!(
        Envelope::from_slice(&huge).unwrap_err().code,
        ErrorCode::Limit
    );
}

#[test]
fn ext_is_a_small_vendor_hatch() {
    // OV-ENV-06
    let ok = {
        let mut text = status("");
        assert!(text.ends_with('}'));
        text.pop();
        text.push_str(r#","ext":{"acme.trace":"abc"}}"#);
        text
    };
    assert!(parse(&ok).is_ok(), "{ok}");

    let mut too_many = status("");
    too_many.pop();
    too_many.push_str(r#","ext":{"#);
    for i in 0..9 {
        if i > 0 {
            too_many.push(',');
        }
        too_many.push_str(&format!(r#""k{i}.token":1"#));
    }
    too_many.push_str("}}");
    assert_eq!(code_of(&too_many), ErrorCode::Limit);

    let mut bad_key = status("");
    bad_key.pop();
    bad_key.push_str(r#","ext":{"NoDot":1}}"#);
    assert_eq!(code_of(&bad_key), ErrorCode::Malformed);
}

#[test]
fn party_ids_are_lowercase_roles() {
    // OV-ID-01
    assert!(PartyId::parse("csms:example-municipal").is_ok());
    assert!(PartyId::parse("ev:depot-7").is_err());
    assert!(PartyId::parse("CSMS:example-municipal").is_err());
    assert!(PartyId::parse("csms:550E8400-E29B-41D4-A716-446655440000").is_err());
    let json = status("").replacen("csms:example-municipal", "ev:depot-7", 1);
    assert_eq!(code_of(&json), ErrorCode::Malformed);
}

#[test]
fn money_is_decimal_not_binary_float() {
    // OV-UNIT-01
    let price = Price {
        currency: "EUR".to_string(),
        per: PricePer::Kwh,
        scale: -4,
        amount: 1500,
    };
    assert_eq!(price.format_major(), "0.1500");
    let negative = Price {
        amount: -25,
        ..price.clone()
    };
    assert_eq!(negative.format_major(), "-0.0025");
    let whole = Price {
        scale: 0,
        amount: 2,
        ..price
    };
    assert_eq!(whole.format_major(), "2");

    let fractional = include_price().replacen("3200", "1.5", 1);
    assert_eq!(code_of(&fractional), ErrorCode::Malformed);
}

#[test]
fn negative_prices_are_real() {
    // OV-UNIT-02
    let json = include_price().replacen("3200", "-25", 1);
    let env = must(&json);
    let Message::SignalPrice(signal) = env.msg else {
        panic!("price");
    };
    assert_eq!(signal.points[0].import_amount, -25);
}

#[test]
fn power_cap_is_ten_gigawatts() {
    // OV-UNIT-03
    let json = status("").replacen("\"power_w\":0", "\"power_w\":10000000001", 1);
    assert_eq!(code_of(&json), ErrorCode::Limit);
}

#[test]
fn price_points_stay_inside_the_valid_window() {
    // OV-PRICE-01
    example("signal.price.json");
    let overlapping = r#"{"v":"0.1","id":"m-price","type":"signal.price","ts":"2026-09-26T16:00:01Z","from":"agg:example-flex","to":"csms:example-municipal","body":{"signal_id":"px-overlap","valid":{"start":"2026-09-26T16:00:00Z","end":"2026-09-27T16:00:00Z"},"currency":"EUR","per":"kWh","scale":-4,"points":[{"start":"2026-09-26T16:00:00Z","end":"2026-09-26T20:00:00Z","import_amount":3200,"export_amount":900},{"start":"2026-09-26T19:00:00Z","end":"2026-09-26T21:00:00Z","import_amount":1,"export_amount":1}]}}"#;
    assert_eq!(code_of(overlapping), ErrorCode::Malformed);
}

#[test]
fn stale_or_future_status_is_rejected() {
    // OV-STAT-01
    let stale = status("").replacen("2026-09-26T15:59:00Z", "2026-09-26T15:00:00Z", 1);
    assert_eq!(code_of(&stale), ErrorCode::Limit);
    let future = status("").replacen("2026-09-26T15:59:00Z", "2026-09-26T16:00:01Z", 1);
    assert_eq!(code_of(&future), ErrorCode::Malformed);
}

#[test]
fn example_offer_is_revision_one() {
    // OV-OFFER-01
    let env = example("offer.flexibility.json");
    let Message::OfferFlexibility(body) = &env.msg else {
        panic!("offer");
    };
    assert_eq!(body.revision, 1);
    assert_eq!(body.terms.product, Product::ChargeShift);
    assert_eq!(body.terms.price.format_major(), "0.1500");

    let mut export_offer = offer("off-export");
    export_offer.terms.product = Product::ExportCap;
    export_offer.terms.power_w = WattBounds {
        min: 0,
        max: 10_000,
    };
    let err = open_new(&export_offer).unwrap_err();
    assert_eq!(err.code, ErrorCode::Malformed);
}

#[test]
fn baseline_ref_is_required_only_for_external() {
    // OV-BASE-01
    let mut body = offer("off-base");
    body.terms.baseline.kind = BaselineKind::ExternalRef;
    let err = open_new(&body).unwrap_err();
    assert_eq!(err.code, ErrorCode::Inconsistent);

    body.terms.baseline.ref_id = Some(nm("tariff-2026"));
    open_new(&body).unwrap();

    let mut unmanaged = offer("off-base-2");
    unmanaged.terms.baseline.ref_id = Some(nm("tariff-2026"));
    let err = open_new(&unmanaged).unwrap_err();
    assert_eq!(err.code, ErrorCode::Inconsistent);
}

#[test]
fn sample_trace_completes_without_a_claim() {
    // OV-DEAL-01
    let path = format!("{}/examples/deal-happy.ndjson", env!("CARGO_MANIFEST_DIR"));
    let text = std::fs::read_to_string(path).unwrap();
    let mut book = PoolBook::new();
    let mut lines = 0;
    for (index, line) in text.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        lines += 1;
        let env = Envelope::from_slice(line.as_bytes())
            .unwrap_or_else(|err| panic!("line {}: {err}", index + 1));
        let from = env.from.clone();
        let to = env.to.clone();
        match env.msg {
            Message::OfferFlexibility(body) => book.open(&from, &to, &body, env.ts).unwrap(),
            Message::OfferAccept(body) => book
                .apply(
                    &body.offer_id,
                    DealEvent::Accept {
                        actor: from,
                        revision: body.revision,
                        now: env.ts,
                    },
                )
                .unwrap(),
            Message::ContractActivate(body) => book
                .apply(
                    &body.offer_id,
                    DealEvent::Activate {
                        actor: from,
                        contract_id: body.contract_id,
                        revision: body.revision,
                        now: env.ts,
                    },
                )
                .unwrap(),
            Message::DispatchSetpoint(body) => {
                book.get(&body.offer_id)
                    .unwrap()
                    .check_setpoint(&from, &body, env.ts)
                    .unwrap();
            }
            Message::ContractComplete(body) => book
                .apply(
                    &body.offer_id,
                    DealEvent::Complete {
                        actor: from,
                        contract_id: body.contract_id,
                    },
                )
                .unwrap(),
            Message::StatusPool(_)
            | Message::SignalPrice(_)
            | Message::DispatchAck(_)
            | Message::MeterInterval(_) => {}
            other => panic!("unexpected message in the sample trace: {other:?}"),
        }
    }
    assert_eq!(lines, 9);
    let deal = book.get(&nm("off-1001")).unwrap();
    assert_eq!(deal.state, DealState::Completed);
    assert_eq!(deal.contract_id.as_ref().unwrap().as_str(), "ctr-1001");
}

#[test]
fn offerer_cannot_accept() {
    // OV-DEAL-02
    let mut book = opened("off-self");
    let err = book
        .apply(
            &nm("off-self"),
            DealEvent::Accept {
                actor: csms(),
                revision: 1,
                now: ts("2026-09-26T16:05:00Z"),
            },
        )
        .unwrap_err();
    assert_eq!(err.code, ErrorCode::WrongParty);
    assert_eq!(book.get(&nm("off-self")).unwrap().state, DealState::Offered);
}

#[test]
fn counter_swaps_roles_and_keeps_the_executor() {
    // OV-DEAL-03
    let mut book = opened("off-counter");
    let id = nm("off-counter");
    let err = book
        .apply(
            &id,
            DealEvent::Counter {
                actor: agg(),
                revision: 3,
                terms: terms_at("2026-09-26T17:40:00Z"),
                now: ts("2026-09-26T16:10:00Z"),
            },
        )
        .unwrap_err();
    assert_eq!(err.code, ErrorCode::BadRevision);

    let mut stolen = terms_at("2026-09-26T17:40:00Z");
    stolen.executor = agg();
    let err = book
        .apply(
            &id,
            DealEvent::Counter {
                actor: agg(),
                revision: 2,
                terms: stolen,
                now: ts("2026-09-26T16:10:00Z"),
            },
        )
        .unwrap_err();
    assert_eq!(err.code, ErrorCode::Conflict);

    let mut revised = terms_at("2026-09-26T17:40:00Z");
    revised.price.amount = 900;
    book.apply(
        &id,
        DealEvent::Counter {
            actor: agg(),
            revision: 2,
            terms: revised,
            now: ts("2026-09-26T16:10:00Z"),
        },
    )
    .unwrap();
    let deal = book.get(&id).unwrap();
    assert_eq!(deal.state, DealState::Countered);
    assert_eq!(deal.revision, 2);
    assert_eq!(deal.offerer, agg());
    assert_eq!(deal.counterparty, csms());
    assert_eq!(deal.executor, csms());
    assert_eq!(deal.terms.price.amount, 900);
}

#[test]
fn only_the_executor_activates() {
    // OV-DEAL-04
    let mut book = opened("off-exec");
    accept(&mut book, "off-exec");
    let err = book
        .apply(
            &nm("off-exec"),
            DealEvent::Activate {
                actor: agg(),
                contract_id: nm("ctr-exec"),
                revision: 1,
                now: ts("2026-09-26T16:10:00Z"),
            },
        )
        .unwrap_err();
    assert_eq!(err.code, ErrorCode::WrongParty);
    book.apply(
        &nm("off-exec"),
        DealEvent::Activate {
            actor: csms(),
            contract_id: nm("ctr-exec"),
            revision: 1,
            now: ts("2026-09-26T16:10:00Z"),
        },
    )
    .unwrap();
    assert_eq!(book.get(&nm("off-exec")).unwrap().state, DealState::Active);
}

#[test]
fn cancel_stops_at_activation() {
    // OV-DEAL-05
    let mut book = opened("off-cancel");
    accept(&mut book, "off-cancel");
    book.apply(
        &nm("off-cancel"),
        DealEvent::Cancel {
            actor: csms(),
            revision: 1,
        },
    )
    .unwrap();
    assert_eq!(
        book.get(&nm("off-cancel")).unwrap().state,
        DealState::Cancelled
    );

    let mut live = opened("off-live");
    accept(&mut live, "off-live");
    live.apply(
        &nm("off-live"),
        DealEvent::Activate {
            actor: csms(),
            contract_id: nm("ctr-live"),
            revision: 1,
            now: ts("2026-09-26T16:10:00Z"),
        },
    )
    .unwrap();
    let err = live
        .apply(
            &nm("off-live"),
            DealEvent::Cancel {
                actor: agg(),
                revision: 1,
            },
        )
        .unwrap_err();
    assert_eq!(err.code, ErrorCode::WrongState);
}

#[test]
fn a_pool_is_not_sold_twice() {
    // OV-DEAL-06
    let mut book = opened("off-a");
    let second = offer("off-b");
    book.open(&csms(), &agg(), &second, ts("2026-09-26T16:00:02Z"))
        .unwrap();
    accept(&mut book, "off-a");
    let err = book
        .apply(
            &nm("off-b"),
            DealEvent::Accept {
                actor: agg(),
                revision: 1,
                now: ts("2026-09-26T16:06:00Z"),
            },
        )
        .unwrap_err();
    assert_eq!(err.code, ErrorCode::Overlap);
    assert_eq!(book.get(&nm("off-b")).unwrap().state, DealState::Offered);
}

#[test]
fn expired_offers_stay_expired() {
    // OV-DEAL-07
    let mut book = opened("off-exp");
    let err = book
        .apply(
            &nm("off-exp"),
            DealEvent::Accept {
                actor: agg(),
                revision: 1,
                now: ts("2026-09-26T17:30:01Z"),
            },
        )
        .unwrap_err();
    assert_eq!(err.code, ErrorCode::Expired);

    let mut on_time = opened("off-ontime");
    on_time
        .apply(
            &nm("off-ontime"),
            DealEvent::Accept {
                actor: agg(),
                revision: 1,
                now: ts("2026-09-26T17:30:00Z"),
            },
        )
        .unwrap();
    assert_eq!(
        on_time.get(&nm("off-ontime")).unwrap().state,
        DealState::Accepted
    );
}

#[test]
fn commercial_power_is_clamped_to_the_constraint() {
    // OV-DISP-01
    example("signal.constraint.json");
    let contract = WattBounds {
        min: -50_000,
        max: 150_000,
    };
    let feeder = WattBounds {
        min: -10_000,
        max: 80_000,
    };
    let bounds = effective_bounds(contract, Some(feeder)).unwrap();
    assert_eq!(clamp_power(-20_000, bounds), -10_000);
    assert_eq!(clamp_power(100_000, bounds), 80_000);
    assert_eq!(clamp_power(10_000, bounds), 10_000);
}

#[test]
fn acks_have_to_tell_the_truth() {
    // OV-DISP-02
    let env = example("dispatch.ack.json");
    let Message::DispatchAck(ack) = &env.msg else {
        panic!("ack");
    };
    assert_eq!(ack.applied_w, -10_000);
    assert!(ack.limited_by.is_some());

    let silent = r#"{"v":"0.1","id":"m-ack","type":"dispatch.ack","ts":"2026-09-26T18:00:05Z","from":"csms:example-municipal","to":"agg:example-flex","corr":"m-setpoint","body":{"contract_id":"ctr-1001","requested_w":-20000,"applied_w":-10000,"kind":"clamped"}}"#;
    assert_eq!(code_of(silent), ErrorCode::Inconsistent);
    let lied = silent.replacen("\"clamped\"", "\"applied\"", 1);
    assert_eq!(code_of(&lied), ErrorCode::Inconsistent);
}

#[test]
fn an_empty_intersection_is_not_a_compromise() {
    // OV-DISP-03
    let contract = WattBounds {
        min: 0,
        max: 10_000,
    };
    let must_export = WattBounds {
        min: -5_000,
        max: -1_000,
    };
    assert!(effective_bounds(contract, Some(must_export)).is_none());
}

#[test]
fn meters_keep_directions_apart() {
    // OV-MTR-01
    let env = example("meter.interval.json");
    let Message::MeterInterval(meter) = &env.msg else {
        panic!("meter");
    };
    assert_eq!(meter.energy_in_wh, 0);
    assert_eq!(meter.energy_out_wh, 5000);
    assert_ne!(meter.energy_in_wh, meter.energy_out_wh);

    let future = std::fs::read_to_string(format!(
        "{}/examples/meter.interval.json",
        env!("CARGO_MANIFEST_DIR")
    ))
    .unwrap()
    .replacen("2026-09-26T18:15:05Z", "2026-09-26T18:14:00Z", 1);
    assert_eq!(code_of(&future), ErrorCode::Malformed);
    let negative = std::fs::read_to_string(format!(
        "{}/examples/meter.interval.json",
        env!("CARGO_MANIFEST_DIR")
    ))
    .unwrap()
    .replacen("\"energy_out_wh\": 5000", "\"energy_out_wh\": -1", 1);
    assert_eq!(code_of(&negative), ErrorCode::Malformed);
}

#[test]
fn operational_meters_cannot_carry_a_claim() {
    // OV-SET-01
    let env = example("meter.interval.json");
    let Message::MeterInterval(meter) = env.msg else {
        panic!("meter");
    };
    let claim = SettlementClaim {
        contract_id: meter.contract_id.clone(),
        offer_id: meter.offer_id.clone(),
        energy_in_wh: meter.energy_in_wh,
        energy_out_wh: meter.energy_out_wh,
    };
    assert_eq!(
        check_claim(&claim, &[meter]).unwrap_err().code,
        ErrorCode::NotFiscal
    );
}

#[test]
fn a_claim_is_the_sum_of_its_intervals() {
    // OV-SET-02
    let intervals = vec![fiscal(2_000, 400), fiscal(1_000, 600)];
    let short = SettlementClaim {
        contract_id: nm("ctr-1001"),
        offer_id: nm("off-1001"),
        energy_in_wh: 3_000,
        energy_out_wh: 999,
    };
    assert_eq!(
        check_claim(&short, &intervals).unwrap_err().code,
        ErrorCode::Inconsistent
    );
    let exact = SettlementClaim {
        energy_out_wh: 1_000,
        ..short
    };
    check_claim(&exact, &intervals).unwrap();
}

#[test]
fn profiles_are_a_prefix() {
    // OV-PROF-01
    let pool = must(&status(""));
    PeerCaps::basic().check(&pool.msg).unwrap();
    let offer = example("offer.flexibility.json");
    assert_eq!(
        PeerCaps::basic().check(&offer.msg).unwrap_err().code,
        ErrorCode::ProfileUnsupported
    );
    PeerCaps::contract().check(&offer.msg).unwrap();
    let ack = example("dispatch.ack.json");
    assert_eq!(
        PeerCaps::contract().check(&ack.msg).unwrap_err().code,
        ErrorCode::ProfileUnsupported
    );
    PeerCaps::full().check(&ack.msg).unwrap();
}

#[test]
fn conformance_ids_match_the_spec() {
    let spec = include_str!("../docs/spec/07-conformance.md");
    let tests = include_str!("conformance.rs");
    let spec_ids = conformance_ids(spec);
    let test_ids = conformance_ids(tests);
    let missing_in_tests: Vec<_> = spec_ids.difference(&test_ids).collect();
    let missing_in_spec: Vec<_> = test_ids.difference(&spec_ids).collect();
    assert!(
        missing_in_tests.is_empty() && missing_in_spec.is_empty(),
        "missing in tests: {missing_in_tests:?}\nmissing in spec: {missing_in_spec:?}"
    );
}

fn fiscal(energy_in: u64, energy_out: u64) -> MeterInterval {
    MeterInterval {
        contract_id: nm("ctr-1001"),
        offer_id: nm("off-1001"),
        pool_id: nm("depot-7"),
        interval: openvet::Interval {
            start: ts("2026-09-26T18:00:00Z"),
            end: ts("2026-09-26T18:15:00Z"),
        },
        energy_in_wh: energy_in,
        energy_out_wh: energy_out,
        quality: openvet::MeterQuality::Fiscal,
        source: "csms_aggregate".to_string(),
    }
}

fn include_price() -> String {
    std::fs::read_to_string(format!(
        "{}/examples/signal.price.json",
        env!("CARGO_MANIFEST_DIR")
    ))
    .unwrap()
}

fn conformance_ids(text: &str) -> BTreeSet<String> {
    let mut found = BTreeSet::new();
    let bytes = text.as_bytes();
    let mut index = 0;
    while index + 3 < bytes.len() {
        if bytes[index..].starts_with(b"OV-") {
            let start = index;
            index += 3;
            while index < bytes.len() {
                let byte = bytes[index];
                if byte.is_ascii_uppercase() || byte.is_ascii_digit() || byte == b'-' {
                    index += 1;
                } else {
                    break;
                }
            }
            let mut end = index;
            while end > start && bytes[end - 1] == b'-' {
                end -= 1;
            }
            if end - start > 3 {
                found.insert(String::from_utf8_lossy(&bytes[start..end]).into_owned());
            }
        } else {
            index += 1;
        }
    }
    found
}
