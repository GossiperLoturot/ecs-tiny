#[test]
fn crud_entity() {
    let mut ecs = ecs_tiny::ECS::new();
    let entity_key = ecs.insert_entity();

    assert!(ecs.get_entity(entity_key).is_ok());
    assert!(ecs.remove_entity(entity_key).is_ok());

    assert!(ecs.get_entity(entity_key).is_err());
    assert!(ecs.remove_entity(entity_key).is_err());
}

#[test]
fn crud_comp() {
    let mut ecs = ecs_tiny::ECS::new();
    let entity_key = ecs.insert_entity();

    let comp_key = ecs.insert_comp(entity_key, 42).unwrap();

    assert_eq!(ecs.get_comp::<i32>(comp_key).ok(), Some(&42));
    assert_eq!(ecs.remove_comp::<i32>(comp_key).ok(), Some(42));

    assert_eq!(ecs.get_comp::<i32>(comp_key).ok(), None);
    assert_eq!(ecs.remove_comp::<i32>(comp_key).ok(), None);
}
