# ecs-tiny

[![crates.io](https://img.shields.io/crates/v/ecs-tiny)](https://crates.io/crates/ecs-tiny)
[![doc.rs](https://img.shields.io/docsrs/ecs-tiny)](https://docs.rs/ecs-tiny)

A minimal ECS supporting entity and component insertion/removal, association, and single-type iteration.

# Usages

```rust
#[derive(Debug, Clone, PartialEq, Eq, strum_macros::EnumDiscriminants)]
#[strum_discriminants(name(CompKind))]
#[strum_discriminants(derive(Hash))]
enum Comp {
    I32(i32),
    Unit(()),
}

// Create new ecs instance and inserts new entity:

let mut ecs = ecs_tiny::ECS::<Comp, CompKind>::new();

let entity_key0 = ecs.insert_entity();
let entity_key1 = ecs.insert_entity();

// Inserts new component associated with specified entity:

let comp_key0 = ecs.insert_comp(entity_key0, Comp::I32(42)).unwrap();
let comp_key1 = ecs.insert_comp(entity_key0, Comp::I32(63)).unwrap();
let comp_key2 = ecs.insert_comp(entity_key1, Comp::I32(42)).unwrap();
let comp_key3 = ecs.insert_comp(entity_key1, Comp::Unit(())).unwrap();

// Iterates over all components associated with specified entity:

for comp in ecs.iter_comp_mut_by_entity(entity_key0, CompKind::I32).unwrap() {
    if let Comp::I32(comp) = comp {
        *comp += 1;
    }
}

// Iterates over all components of specified type (single type only):

for comp in ecs.iter_comp_mut(CompKind::I32).unwrap() {
    if let Comp::I32(comp) = comp {
        *comp += 1;
    }
}

// Removes specified component:

ecs.remove_comp(comp_key0).unwrap();

// Removes specified entity:

ecs.remove_entity(entity_key1).unwrap();
```
