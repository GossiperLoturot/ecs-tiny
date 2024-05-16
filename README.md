# ecs-tiny

[![crates.io](https://img.shields.io/crates/v/ecs-tiny)](https://crates.io/crates/ecs-tiny)
[![doc.rs](https://img.shields.io/docsrs/ecs-tiny)](https://docs.rs/ecs-tiny)

A minimal ECS supporting entity and component insertion/removal, association, and single-type iteration.

# Usages

Create new ecs instance and inserts new entity:

```rust
let mut ecs = ecs_tiny::ECS::new();

let entity_key0 = ecs.insert_entity();
let entity_key1 = ecs.insert_entity();
```

Inserts new component associated with specified entity:

```rust
let comp_key0 = ecs.insert_component(entity_key, 42);
let comp_key1 = ecs.insert_component(entity_key, 63);
let comp_key2 = ecs.insert_component(entity_key, 42);
let comp_key3 = ecs.insert_component(entity_key, ());
```

Iterates over all components associated with specified entity:

```rust
for comp in ecs.iter_comp_mut_by_entity::<i32>(entity_key0) {
    *comp += 1;
}
```

Iterates over all components of specified type:

```rust
for comp in ecs.iter_comp_mut::<i32>() {
    *comp += 1;
}
```

Removes specified component:

```rust
ecs.remove_comp::<i32>(comp_key0);
```

Removes specified entity:

```rust
ecs.remove_entity(entity_key1);
```
