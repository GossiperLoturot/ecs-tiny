# ecs-tiny

[![crates.io](https://img.shields.io/crates/v/ecs-tiny)](https://crates.io/crates/ecs-tiny)
[![doc.rs](https://img.shields.io/docsrs/ecs-tiny)](https://docs.rs/ecs-tiny)

A minimal ECS supporting entity and component insertion/removal, association, and single-type iteration.

# Usages

```rust
// Create new ecs instance and inserts new entity:

let mut ecs = ecs_tiny::ECS::new();

let entity_key0 = ecs.insert_entity();
let entity_key1 = ecs.insert_entity();

// Inserts new component associated with specified entity:

let comp_key0 = ecs.insert_comp(entity_key0, 42).unwrap();
let comp_key1 = ecs.insert_comp(entity_key0, 63).unwrap();
let comp_key2 = ecs.insert_comp(entity_key1, 42).unwrap();
let comp_key3 = ecs.insert_comp(entity_key1, ()).unwrap();

// Iterates over all components associated with specified entity:

for comp in ecs.iter_comp_mut_by_entity::<i32>(entity_key0).unwrap() {
    *comp += 1;
}

// Iterates over all components of specified type (single type only):

for comp in ecs.iter_comp_mut::<i32>().unwrap() {
    *comp += 1;
}

// Removes specified component:

ecs.remove_comp::<i32>(comp_key0).unwrap();

// Removes specified entity:

ecs.remove_entity(entity_key1).unwrap();
```
