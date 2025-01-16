use core_macro::VersionCtr;

#[derive(VersionCtr)]
struct Person {
    #[version(since = 1.0_f32, until = 1.0_f32)]
    name: String,
    #[version]
    age: u32,
    #[version]
    email: String,
}

#[test]
fn test() {
    let p = Person {
        name: "John".to_string(),
        age: 30,
        email: "qq.com".to_string(),
    };
    let json = p.into_json(1.0_f32);
    println!("{}", json);
}
