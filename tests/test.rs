#[test]
fn crud_entity() {
    let mut ecs = ecs_tiny::ECS::new();
    let entity_key = ecs.insert_entity();

    assert!(ecs.get_entity(entity_key).is_some());
    assert!(ecs.remove_entity(entity_key).is_some());

    assert!(ecs.get_entity(entity_key).is_none());
    assert!(ecs.remove_entity(entity_key).is_none());
}

#[test]
fn crud_comp() {
    let mut ecs = ecs_tiny::ECS::new();
    let entity_key = ecs.insert_entity();
    let comp_key = ecs.insert_comp(entity_key, 42).unwrap();

    assert_eq!(ecs.get_comp::<i32>(comp_key), Some(&42));
    assert_eq!(ecs.get_comp_mut::<i32>(comp_key), Some(&mut 42));
    assert_eq!(ecs.remove_comp::<i32>(comp_key), Some(42));

    assert_eq!(ecs.get_comp::<i32>(comp_key), None);
    assert_eq!(ecs.get_comp_mut::<i32>(comp_key), None);
    assert_eq!(ecs.remove_comp::<i32>(comp_key), None);
}

#[test]
fn insert_comp_with_invalid_entity() {
    let mut ecs = ecs_tiny::ECS::new();
    let entity_key = ecs.insert_entity();
    ecs.remove_entity(entity_key).unwrap();

    assert!(ecs.insert_comp(entity_key, 42).is_none());
}

#[test]
fn remove_comp_with_invalid_type() {
    let mut ecs = ecs_tiny::ECS::new();
    let entity_key = ecs.insert_entity();
    let comp_key = ecs.insert_comp(entity_key, 42).unwrap();

    assert!(ecs.get_comp::<()>(comp_key).is_none());
    assert!(ecs.get_comp_mut::<()>(comp_key).is_none());
    assert!(ecs.remove_comp::<()>(comp_key).is_none());
}

#[test]
fn remove_entity_and_associated_comp() {
    let mut ecs = ecs_tiny::ECS::new();
    let entity_key = ecs.insert_entity();
    let comp_key = ecs.insert_comp(entity_key, 42).unwrap();

    assert!(ecs.remove_entity(entity_key).is_some());

    assert!(ecs.get_entity(entity_key).is_none());
    assert!(ecs.get_comp::<i32>(comp_key).is_none());
    assert!(ecs.get_comp_mut::<i32>(comp_key).is_none());
}

#[test]
fn iter_entity() {
    let mut ecs = ecs_tiny::ECS::new();
    let entity_key0 = ecs.insert_entity();
    let entity_key1 = ecs.insert_entity();
    let mut iter = ecs.iter_entity();

    assert_eq!(iter.next(), Some(entity_key0));
    assert_eq!(iter.next(), Some(entity_key1));
    assert_eq!(iter.next(), None);
}

#[test]
fn iter_comp() {
    let mut ecs = ecs_tiny::ECS::new();
    let entity_key0 = ecs.insert_entity();
    let entity_key1 = ecs.insert_entity();
    ecs.insert_comp(entity_key0, 42).unwrap();
    ecs.insert_comp(entity_key0, 63).unwrap();
    ecs.insert_comp(entity_key1, 42).unwrap();
    ecs.insert_comp(entity_key1, ()).unwrap();
    let mut iter = ecs.iter_comp::<i32>().unwrap();

    assert_eq!(iter.next(), Some(&42));
    assert_eq!(iter.next(), Some(&63));
    assert_eq!(iter.next(), Some(&42));
    assert_eq!(iter.next(), None);

    drop(iter);
    let mut iter = ecs.iter_comp_mut::<i32>().unwrap();

    assert_eq!(iter.next(), Some(&mut 42));
    assert_eq!(iter.next(), Some(&mut 63));
    assert_eq!(iter.next(), Some(&mut 42));
    assert_eq!(iter.next(), None);
}

#[test]
fn iter_comp_with_invalid_type() {
    let mut ecs = ecs_tiny::ECS::new();

    assert!(ecs.iter_comp::<i32>().is_none());
    assert!(ecs.iter_comp_mut::<i32>().is_none());
}

#[test]
fn get_entity_by_comp() {
    let mut ecs = ecs_tiny::ECS::new();
    let entity_key0 = ecs.insert_entity();
    let entity_key1 = ecs.insert_entity();
    let comp_key0 = ecs.insert_comp(entity_key0, 42).unwrap();
    let comp_key1 = ecs.insert_comp(entity_key0, 63).unwrap();
    let comp_key2 = ecs.insert_comp(entity_key1, 42).unwrap();
    let comp_key3 = ecs.insert_comp(entity_key1, ()).unwrap();
    ecs.remove_comp::<i32>(comp_key2).unwrap();

    assert_eq!(ecs.get_entity_by_comp(comp_key0), Some(entity_key0));
    assert_eq!(ecs.get_entity_by_comp(comp_key1), Some(entity_key0));
    assert_eq!(ecs.get_entity_by_comp(comp_key2), None);
    assert_eq!(ecs.get_entity_by_comp(comp_key3), Some(entity_key1));
}

#[test]
fn iter_comp_by_entity() {
    let mut ecs = ecs_tiny::ECS::new();
    let entity_key0 = ecs.insert_entity();
    let entity_key1 = ecs.insert_entity();
    ecs.insert_comp(entity_key0, 42).unwrap();
    ecs.insert_comp(entity_key0, 63).unwrap();
    ecs.insert_comp(entity_key1, 42).unwrap();
    ecs.insert_comp(entity_key1, ()).unwrap();
    let mut iter = ecs.iter_comp_by_entity::<i32>(entity_key0).unwrap();

    assert_eq!(iter.next(), Some(&42));
    assert_eq!(iter.next(), Some(&63));
    assert_eq!(iter.next(), None);

    drop(iter);
    let mut iter = ecs.iter_comp_mut_by_entity::<i32>(entity_key0).unwrap();

    assert_eq!(iter.next(), Some(&mut 42));
    assert_eq!(iter.next(), Some(&mut 63));
    assert_eq!(iter.next(), None);
}

#[test]
fn clear() {
    let mut ecs = ecs_tiny::ECS::new();
    let entity_key0 = ecs.insert_entity();
    let entity_key1 = ecs.insert_entity();
    ecs.insert_comp(entity_key0, 42).unwrap();
    ecs.insert_comp(entity_key0, 63).unwrap();
    ecs.insert_comp(entity_key1, 42).unwrap();
    ecs.insert_comp(entity_key1, ()).unwrap();
    ecs.clear();
}
