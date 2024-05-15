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

    let e0 = plugin.insert_entity();

    let c0 = plugin
        .insert_comp(e0, CompA::new("Hello".to_string()))
        .unwrap();
    let c1 = plugin
        .insert_comp(e0, CompA::new("World".to_string()))
        .unwrap();

    for c in plugin.iter_comp_mut_by_entity::<CompA>(e0).unwrap() {
        c.content += "!";
    }

    for c in plugin.iter_comp_by_entity::<CompA>(e0).unwrap() {
        println!("{}", c.content);
    }

    plugin.remove_comp::<CompA>(c0).unwrap();
    plugin.remove_comp::<CompA>(c1).unwrap();

    plugin.remove_entity(e0).unwrap();
}
