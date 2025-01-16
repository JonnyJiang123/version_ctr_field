pub trait IntoJson {
    fn into_json(self, version: f32) -> String;
}

#[test]
fn test() {
    let _ = String::new();
    use std::collections::BTreeMap;
    let mut map = BTreeMap::new();
    map.insert("a", 1);
    map.insert("b", 2);
    println!("{:?}", map);
}
