//! Fast check of the owning native candle-use classifier over real block facts.
#[path = "../../crates/gameplay/src/block_use/candle.rs"] mod candle;
fn main() {
    let records = assets::read_registry_for_protocol(assets::pinned_block_registry_bytes(), assets::active_content_registry_protocol()).unwrap();
    let facts = assets::facts::BlockFacts::new(&records).unwrap();
    let fact = |name| facts.get(records.iter().find(|r|r.name.as_ref()==name).unwrap().sequential_id);
    let cake = fact("minecraft:cake");
    let zero = r#"{"bite_counter":{"type":"int","value":0}}"#;
    let bitten = r#"{"bite_counter":{"type":"int","value":1}}"#;
    let names: std::collections::BTreeSet<_> = records.iter().filter(|r| r.name.as_ref()=="minecraft:candle" || r.name.ends_with("_candle")).map(|r|r.name.as_ref()).collect();
    assert_eq!(names.len(),17);
    for name in names {
        let held = fact(name);
        assert!(candle::uses_cake(cake,held,Some(zero),true,true),"{name}");
        for canonical in [None, Some(bitten), Some("{}"), Some("broken")] {
            assert!(!candle::uses_cake(cake,held,canonical,true,true));
        }
        assert!(!candle::uses_cake(cake,held,Some(zero),false,true));
        assert!(!candle::uses_cake(cake,held,Some(zero),true,false));
    }
    assert!(!candle::uses_cake(cake,fact("minecraft:stone"),Some(zero),true,true));
    println!("PASS: all 17 candle items use an intact cake; bitten/unreadable cake, empty stacks and denied building do not");
}
