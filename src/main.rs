struct CompA {
    content: String,
}

impl CompA {
    fn new(content: String) -> Self {
        Self { content }
    }
}

fn main() {
    let mut plugin = ecs_tiny::ECS::new();

    let e0 = plugin.insert_entity().unwrap();

    let c0 = plugin
        .insert_comp(e0, CompA::new("Hello".to_string()))
        .unwrap();
    let c1 = plugin
        .insert_comp(e0, CompA::new("World".to_string()))
        .unwrap();

    let comp = plugin.get_comp::<CompA>(c1).unwrap();
    println!("{}", comp.content);

    plugin.remove_comp::<CompA>(c0).unwrap();

    plugin.remove_entity(e0).unwrap();
}
