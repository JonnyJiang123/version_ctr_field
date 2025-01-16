use core_macro::VersionCtr;

fn main() {
    let person = Person {
        name: "John".to_string(),
        email: "john@example.com".to_string(),
    };
    println!("{}", person.into_json(20.0_f32));
}
#[derive(VersionCtr)]
struct Person {
    #[version(since = "1.0", until = "15.0")]
    name: String,
    #[version]
    email: String,
}

// impl Display for Person {
//     fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//         write!(f,"{}",BTreeMap::new())
//     }
// }
