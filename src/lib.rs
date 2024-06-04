//! # ecs-tiny
//!
//! A minimal ECS supporting entity and component insertion/removal, association, and single-type iteration.
//!
//! # Usages
//!
//! ```
//! #[derive(Debug, Clone, PartialEq, Eq, strum_macros::EnumDiscriminants)]
//! #[strum_discriminants(name(CompKind))]
//! #[strum_discriminants(derive(Hash))]
//! enum Comp {
//!     I32(i32),
//!     Unit(()),
//! }
//!
//! // Create new ecs instance and inserts new entity:
//!
//! let mut ecs = ecs_tiny::ECS::<Comp, CompKind>::new();
//!
//! let entity_key0 = ecs.insert_entity();
//! let entity_key1 = ecs.insert_entity();
//!
//! // Inserts new component associated with specified entity:
//!
//! let comp_key0 = ecs.insert_comp(entity_key0, Comp::I32(42)).unwrap();
//! let comp_key1 = ecs.insert_comp(entity_key0, Comp::I32(63)).unwrap();
//! let comp_key2 = ecs.insert_comp(entity_key1, Comp::I32(42)).unwrap();
//! let comp_key3 = ecs.insert_comp(entity_key1, Comp::Unit(())).unwrap();
//!
//! // Iterates over all components associated with specified entity:
//!
//! for comp in ecs.iter_comp_mut_by_entity(entity_key0, CompKind::I32).unwrap() {
//!     if let Comp::I32(comp) = comp {
//!         *comp += 1;
//!     }
//! }
//!
//! // Iterates over all components of specified type (single type only):
//!
//! for comp in ecs.iter_comp_mut(CompKind::I32).unwrap() {
//!     if let Comp::I32(comp) = comp {
//!         *comp += 1;
//!     }
//! }
//!
//! // Removes specified component:
//!
//! ecs.remove_comp(comp_key0).unwrap();
//!
//! // Removes specified entity:
//!
//! ecs.remove_entity(entity_key1).unwrap();
//! ```

type EntityKey = u32;

type CompKey<K> = (K, u32);

#[derive(Debug, Clone)]
struct CompMeta<T> {
    inner: T,
    entity_key: u32,
    relation_0: u32,
    relation_1: u32,
}

/// A minimal ECS supporting entity and component insertion/removal, association, and single-type iteration.
///
/// # Examples
///
/// ```
/// #[derive(Debug, Clone, PartialEq, Eq, strum_macros::EnumDiscriminants)]
/// #[strum_discriminants(name(CompKind))]
/// #[strum_discriminants(derive(Hash))]
/// enum Comp {
///     I32(i32),
///     Unit(()),
/// }
///
/// let mut ecs = ecs_tiny::ECS::<Comp, CompKind>::new();
///
/// let entity_key = ecs.insert_entity();
///
/// let comp_key0 = ecs.insert_comp(entity_key, Comp::I32(42)).unwrap();
/// let comp_key1 = ecs.insert_comp(entity_key, Comp::I32(63)).unwrap();
///
/// for comp in ecs.iter_comp_mut(CompKind::I32).unwrap() {
///     if let Comp::I32(comp) = comp {
///         *comp += 1;
///     }
/// }
/// ```
#[derive(Debug, Clone)]
pub struct ECS<C, K> {
    entities: slab::Slab<()>,
    comp_metas: ahash::AHashMap<K, slab::Slab<CompMeta<C>>>,
    relation_0: ahash::AHashMap<EntityKey, slab::Slab<(K, u32)>>,
    relation_1: ahash::AHashMap<(EntityKey, K), slab::Slab<u32>>,
}

impl<C, K> Default for ECS<C, K> {
    fn default() -> Self {
        Self {
            entities: Default::default(),
            comp_metas: Default::default(),
            relation_0: Default::default(),
            relation_1: Default::default(),
        }
    }
}

impl<C, K> ECS<C, K>
where
    for<'a> K: From<&'a C>,
    K: Copy + Eq + std::hash::Hash,
{
    /// Create a new ECS instance.
    ///
    /// # Examples
    ///
    /// ```
    /// #[derive(Debug, Clone, PartialEq, Eq, strum_macros::EnumDiscriminants)]
    /// #[strum_discriminants(name(CompKind))]
    /// #[strum_discriminants(derive(Hash))]
    /// enum Comp {
    ///     I32(i32),
    ///     Unit(()),
    /// }
    ///
    /// let mut ecs = ecs_tiny::ECS::<Comp, CompKind>::new();
    /// ```
    pub fn new() -> Self {
        Default::default()
    }

    /// Insert a new entity and return the corresponding entity key.
    ///
    /// # Examples
    ///
    /// ```
    /// #[derive(Debug, Clone, PartialEq, Eq, strum_macros::EnumDiscriminants)]
    /// #[strum_discriminants(name(CompKind))]
    /// #[strum_discriminants(derive(Hash))]
    /// enum Comp {
    ///     I32(i32),
    ///     Unit(()),
    /// }
    ///
    /// let mut ecs = ecs_tiny::ECS::<Comp, CompKind>::new();
    /// let entity_key = ecs.insert_entity();
    /// ```
    pub fn insert_entity(&mut self) -> EntityKey {
        self.entities.insert(()) as u32
    }

    /// Remove an entity with the corresponding entity key.
    /// If the entity corresponding to the entity key is not found, return an `None`.
    /// Otherwise, return an `Some(())`.
    ///
    /// # Examples
    ///
    /// ```
    /// #[derive(Debug, Clone, PartialEq, Eq, strum_macros::EnumDiscriminants)]
    /// #[strum_discriminants(name(CompKind))]
    /// #[strum_discriminants(derive(Hash))]
    /// enum Comp {
    ///     I32(i32),
    ///     Unit(()),
    /// }
    ///
    /// let mut ecs = ecs_tiny::ECS::<Comp, CompKind>::new();
    /// let entity_key = ecs.insert_entity();
    /// ecs.remove_entity(entity_key).unwrap();
    /// ```
    pub fn remove_entity(&mut self, entity_key: EntityKey) -> Option<()> {
        self.entities.try_remove(entity_key as usize)?;

        if let Some(relation_0) = self.relation_0.remove(&entity_key) {
            for (_, (kind_key, slab_key)) in relation_0 {
                let comp_meta = self
                    .comp_metas
                    .get_mut(&kind_key)
                    .check()
                    .try_remove(slab_key as usize)
                    .check();

                self.relation_1
                    .get_mut(&(entity_key, kind_key))
                    .check()
                    .try_remove(comp_meta.relation_1 as usize)
                    .check();
            }
        }

        Some(())
    }

    /// Return entity with the corresponding entity key.
    /// If the entity corresponding to the entity key is not found, return an `None`.
    /// Otherwise, return an `Some(())`.
    ///
    /// # Examples
    ///
    /// ```
    /// #[derive(Debug, Clone, PartialEq, Eq, strum_macros::EnumDiscriminants)]
    /// #[strum_discriminants(name(CompKind))]
    /// #[strum_discriminants(derive(Hash))]
    /// enum Comp {
    ///     I32(i32),
    ///     Unit(()),
    /// }
    ///
    /// let mut ecs = ecs_tiny::ECS::<Comp, CompKind>::new();
    /// let entity_key = ecs.insert_entity();
    /// ecs.get_entity(entity_key).unwrap();
    /// ```
    pub fn get_entity(&self, entity_key: EntityKey) -> Option<()> {
        self.entities.get(entity_key as usize)?;
        Some(())
    }

    /// Return an iterator over all entity keys.
    ///
    /// # Examples
    ///
    /// ```
    /// #[derive(Debug, Clone, PartialEq, Eq, strum_macros::EnumDiscriminants)]
    /// #[strum_discriminants(name(CompKind))]
    /// #[strum_discriminants(derive(Hash))]
    /// enum Comp {
    ///     I32(i32),
    ///     Unit(()),
    /// }
    ///
    /// let mut ecs = ecs_tiny::ECS::<Comp, CompKind>::new();
    /// let entity_key0 = ecs.insert_entity();
    /// let entity_key1 = ecs.insert_entity();
    /// let entity_key2 = ecs.insert_entity();
    /// let mut iter = ecs.iter_entity();
    ///
    /// assert_eq!(iter.next(), Some(entity_key0));
    /// assert_eq!(iter.next(), Some(entity_key1));
    /// assert_eq!(iter.next(), Some(entity_key2));
    /// assert_eq!(iter.next(), None);
    /// ```
    pub fn iter_entity(&self) -> impl Iterator<Item = EntityKey> + '_ {
        self.entities.iter().map(|(key, _)| key as u32)
    }

    /// Insert a new component with the corresponding entity key and return the corresponding component key.
    /// If the entity corresponding to the entity key is not found, return an `None`.
    /// Otherwise, return an `Some(CompKey)`.
    ///
    /// # Examples
    ///
    /// ```
    /// #[derive(Debug, Clone, PartialEq, Eq, strum_macros::EnumDiscriminants)]
    /// #[strum_discriminants(name(CompKind))]
    /// #[strum_discriminants(derive(Hash))]
    /// enum Comp {
    ///     I32(i32),
    ///     Unit(()),
    /// }
    ///
    /// let mut ecs = ecs_tiny::ECS::<Comp, CompKind>::new();
    /// let entity_key = ecs.insert_entity();
    /// let comp_key = ecs.insert_comp(entity_key, Comp::I32(42)).unwrap();
    /// ```
    pub fn insert_comp(&mut self, entity_key: EntityKey, comp: C) -> Option<CompKey<K>> {
        self.entities.get(entity_key as usize)?;

        let kind_key = K::from(&comp);

        let comp_metas = self.comp_metas.entry(kind_key).or_default();

        let slab_key = comp_metas.vacant_key() as u32;

        let relation_0 = self
            .relation_0
            .entry(entity_key)
            .or_default()
            .insert((kind_key, slab_key)) as u32;

        let relation_1 = self
            .relation_1
            .entry((entity_key, kind_key))
            .or_default()
            .insert(slab_key) as u32;

        let comp_meta = CompMeta {
            inner: comp,
            entity_key,
            relation_0,
            relation_1,
        };
        comp_metas.insert(comp_meta);

        Some((kind_key, slab_key))
    }

    /// Remove a component with the corresponding component key and type, and return the component.
    /// If the component corresponding to the component key and type is not found, return an `None`.
    /// Otherwise, return an `Some(T)`.
    ///
    /// # Examples
    ///
    /// ```
    /// #[derive(Debug, Clone, PartialEq, Eq, strum_macros::EnumDiscriminants)]
    /// #[strum_discriminants(name(CompKind))]
    /// #[strum_discriminants(derive(Hash))]
    /// enum Comp {
    ///     I32(i32),
    ///     Unit(()),
    /// }
    ///
    /// let mut ecs = ecs_tiny::ECS::<Comp, CompKind>::new();
    /// let entity_key = ecs.insert_entity();
    /// let comp_key = ecs.insert_comp(entity_key, Comp::I32(42)).unwrap();
    /// let comp = ecs.remove_comp(comp_key).unwrap();
    ///
    /// assert_eq!(comp, Comp::I32(42));
    /// ```
    pub fn remove_comp(&mut self, comp_key: CompKey<K>) -> Option<C> {
        let (kind_key, slab_key) = comp_key;

        let comp_metas = self.comp_metas.get_mut(&kind_key)?;
        let comp_meta = comp_metas.try_remove(slab_key as usize)?;

        self.relation_0
            .get_mut(&comp_meta.entity_key)
            .check()
            .try_remove(comp_meta.relation_0 as usize)
            .check();

        self.relation_1
            .get_mut(&(comp_meta.entity_key, kind_key))
            .check()
            .try_remove(comp_meta.relation_1 as usize)
            .check();

        Some(comp_meta.inner)
    }

    /// Return a component with the corresponding component key and type.
    /// If the component corresponding to the component key and type is not found, return an `None`.
    /// Otherwise, return an `Some(&T)`.
    ///
    /// # Examples
    ///
    /// ```
    /// #[derive(Debug, Clone, PartialEq, Eq, strum_macros::EnumDiscriminants)]
    /// #[strum_discriminants(name(CompKind))]
    /// #[strum_discriminants(derive(Hash))]
    /// enum Comp {
    ///     I32(i32),
    ///     Unit(()),
    /// }
    ///
    /// let mut ecs = ecs_tiny::ECS::<Comp, CompKind>::new();
    /// let entity_key = ecs.insert_entity();
    /// let comp_key = ecs.insert_comp(entity_key, Comp::I32(42)).unwrap();
    /// let comp = ecs.get_comp(comp_key).unwrap();
    ///
    /// assert_eq!(comp, &Comp::I32(42));
    /// ```
    pub fn get_comp(&self, comp_key: CompKey<K>) -> Option<&C> {
        let (kind_key, slab_key) = comp_key;

        let comps = self.comp_metas.get(&kind_key)?;
        let comp_meta = comps.get(slab_key as usize)?;

        Some(&comp_meta.inner)
    }

    /// Return a mutable component with the corresponding component key and type.
    /// If the component corresponding to the component key and type is not found, return an `None`.
    /// Otherwise, return an `Some(&mut T)`.
    ///
    /// # Examples
    ///
    /// ```
    /// #[derive(Debug, Clone, PartialEq, Eq, strum_macros::EnumDiscriminants)]
    /// #[strum_discriminants(name(CompKind))]
    /// #[strum_discriminants(derive(Hash))]
    /// enum Comp {
    ///     I32(i32),
    ///     Unit(()),
    /// }
    ///
    /// let mut ecs = ecs_tiny::ECS::<Comp, CompKind>::new();
    /// let entity_key = ecs.insert_entity();
    /// let comp_key = ecs.insert_comp(entity_key, Comp::I32(42)).unwrap();
    /// let comp = ecs.get_comp_mut(comp_key).unwrap();
    ///
    /// assert_eq!(comp, &mut Comp::I32(42));
    /// ```
    pub fn get_comp_mut(&mut self, comp_key: CompKey<K>) -> Option<&mut C> {
        let (kind_key, slab_key) = comp_key;

        let comp_metas = self.comp_metas.get_mut(&kind_key)?;
        let comp_meta = comp_metas.get_mut(slab_key as usize)?;

        Some(&mut comp_meta.inner)
    }

    /// Return an iterator over all components of the corresponding type.
    /// If the component type is not found, return an `None`.
    /// Otherwise, return an `Some(impl Iterator<Item = &T>)`.
    ///
    /// # Examples
    ///
    /// ```
    /// #[derive(Debug, Clone, PartialEq, Eq, strum_macros::EnumDiscriminants)]
    /// #[strum_discriminants(name(CompKind))]
    /// #[strum_discriminants(derive(Hash))]
    /// enum Comp {
    ///     I32(i32),
    ///     Unit(()),
    /// }
    ///
    /// let mut ecs = ecs_tiny::ECS::<Comp, CompKind>::new();
    /// let entity_key0 = ecs.insert_entity();
    /// let entity_key1 = ecs.insert_entity();
    /// ecs.insert_comp(entity_key0, Comp::I32(42)).unwrap();
    /// ecs.insert_comp(entity_key0, Comp::I32(63)).unwrap();
    /// ecs.insert_comp(entity_key1, Comp::I32(42)).unwrap();
    /// let mut iter = ecs.iter_comp(CompKind::I32).unwrap();
    ///
    /// assert_eq!(iter.next(), Some(&Comp::I32(42)));
    /// assert_eq!(iter.next(), Some(&Comp::I32(63)));
    /// assert_eq!(iter.next(), Some(&Comp::I32(42)));
    /// assert_eq!(iter.next(), None);
    /// ```
    pub fn iter_comp(&self, kind_key: K) -> Option<impl Iterator<Item = &C>> {
        let comps = self.comp_metas.get(&kind_key)?;
        let iter = comps.iter().map(|(_, comp)| &comp.inner);

        Some(iter)
    }

    /// Return a mutable iterator over all components of the corresponding type.
    /// If the component type is not found, return an `None`.
    /// Otherwise, return an `Some(impl Iterator<Item = &mut T>)`.
    ///
    /// # Examples
    ///
    /// ```
    /// #[derive(Debug, Clone, PartialEq, Eq, strum_macros::EnumDiscriminants)]
    /// #[strum_discriminants(name(CompKind))]
    /// #[strum_discriminants(derive(Hash))]
    /// enum Comp {
    ///     I32(i32),
    ///     Unit(()),
    /// }
    ///
    /// let mut ecs = ecs_tiny::ECS::<Comp, CompKind>::new();
    /// let entity_key0 = ecs.insert_entity();
    /// let entity_key1 = ecs.insert_entity();
    /// ecs.insert_comp(entity_key0, Comp::I32(42)).unwrap();
    /// ecs.insert_comp(entity_key0, Comp::I32(63)).unwrap();
    /// ecs.insert_comp(entity_key1, Comp::I32(42)).unwrap();
    /// let mut iter = ecs.iter_comp_mut(CompKind::I32).unwrap();
    ///
    /// assert_eq!(iter.next(), Some(&mut Comp::I32(42)));
    /// assert_eq!(iter.next(), Some(&mut Comp::I32(63)));
    /// assert_eq!(iter.next(), Some(&mut Comp::I32(42)));
    /// assert_eq!(iter.next(), None);
    /// ```
    pub fn iter_comp_mut(&mut self, kind_key: K) -> Option<impl Iterator<Item = &mut C>> {
        let comps = self.comp_metas.get_mut(&kind_key)?;
        let iter = comps.iter_mut().map(|(_, comp)| &mut comp.inner);

        Some(iter)
    }

    /// Return an entity key with the corresponding component key.
    /// If the component corresponding to the component key is not found, return an `None`.
    /// Otherwise, return an `Some(EntityKey)`.
    ///
    /// # Examples
    ///
    /// ```
    /// #[derive(Debug, Clone, PartialEq, Eq, strum_macros::EnumDiscriminants)]
    /// #[strum_discriminants(name(CompKind))]
    /// #[strum_discriminants(derive(Hash))]
    /// enum Comp {
    ///     I32(i32),
    ///     Unit(()),
    /// }
    ///
    /// let mut ecs = ecs_tiny::ECS::<Comp, CompKind>::new();
    /// let entity_key0 = ecs.insert_entity();
    /// let entity_key1 = ecs.insert_entity();
    /// let comp_key0 = ecs.insert_comp(entity_key0, Comp::I32(42)).unwrap();
    /// let comp_key1 = ecs.insert_comp(entity_key0, Comp::I32(63)).unwrap();
    /// let comp_key2 = ecs.insert_comp(entity_key1, Comp::I32(42)).unwrap();
    /// let entity_key = ecs.get_entity_by_comp(comp_key0).unwrap();
    ///
    /// assert_eq!(entity_key, entity_key0);
    /// ```
    pub fn get_entity_by_comp(&self, comp_key: CompKey<K>) -> Option<EntityKey> {
        let (kind_key, slab_key) = comp_key;

        let comp_metas = self.comp_metas.get(&kind_key)?;
        let comp_meta = comp_metas.get(slab_key as usize)?;

        Some(comp_meta.entity_key)
    }

    /// Return an iterator over all components with the corresponding entity key and type.
    /// If the entity corresponding to the entity key and type is not found, return an `None`.
    /// Otherwise, return an `Some(impl Iterator<Item = &T>)`.
    ///
    /// # Examples
    ///
    /// ```
    /// #[derive(Debug, Clone, PartialEq, Eq, strum_macros::EnumDiscriminants)]
    /// #[strum_discriminants(name(CompKind))]
    /// #[strum_discriminants(derive(Hash))]
    /// enum Comp {
    ///     I32(i32),
    ///     Unit(()),
    /// }
    ///
    /// let mut ecs = ecs_tiny::ECS::<Comp, CompKind>::new();
    /// let entity_key0 = ecs.insert_entity();
    /// let entity_key1 = ecs.insert_entity();
    /// ecs.insert_comp(entity_key0, Comp::I32(42)).unwrap();
    /// ecs.insert_comp(entity_key0, Comp::I32(63)).unwrap();
    /// ecs.insert_comp(entity_key1, Comp::I32(42)).unwrap();
    /// let mut iter = ecs.iter_comp_by_entity(entity_key0, CompKind::I32).unwrap();
    ///
    /// assert_eq!(iter.next(), Some(&Comp::I32(42)));
    /// assert_eq!(iter.next(), Some(&Comp::I32(63)));
    /// assert_eq!(iter.next(), None);
    /// ```
    pub fn iter_comp_by_entity(
        &self,
        entity_key: EntityKey,
        kind_key: K,
    ) -> Option<impl Iterator<Item = &C>> {
        let comp_metas = self.comp_metas.get(&kind_key)?;

        let relation_1 = self.relation_1.get(&(entity_key, kind_key))?;

        let iter = relation_1
            .iter()
            .map(|(_, slab_key)| &comp_metas.get(*slab_key as usize).check().inner);

        Some(iter)
    }

    /// Return a mutable iterator over all components with the corresponding entity key and type.
    /// If the entity corresponding to the entity key and type is not found, return an `None`.
    /// Otherwise, return an `None(impl Iterator<Item = &mut T>)`.
    ///
    /// # Examples
    ///
    /// ```
    /// #[derive(Debug, Clone, PartialEq, Eq, strum_macros::EnumDiscriminants)]
    /// #[strum_discriminants(name(CompKind))]
    /// #[strum_discriminants(derive(Hash))]
    /// enum Comp {
    ///     I32(i32),
    ///     Unit(()),
    /// }
    ///
    /// let mut ecs = ecs_tiny::ECS::<Comp, CompKind>::new();
    /// let entity_key0 = ecs.insert_entity();
    /// let entity_key1 = ecs.insert_entity();
    /// ecs.insert_comp(entity_key0, Comp::I32(42)).unwrap();
    /// ecs.insert_comp(entity_key0, Comp::I32(63)).unwrap();
    /// ecs.insert_comp(entity_key1, Comp::I32(42)).unwrap();
    /// let mut iter = ecs.iter_comp_mut_by_entity(entity_key0, CompKind::I32).unwrap();
    ///
    /// assert_eq!(iter.next(), Some(&mut Comp::I32(42)));
    /// assert_eq!(iter.next(), Some(&mut Comp::I32(63)));
    /// assert_eq!(iter.next(), None);
    /// ```
    pub fn iter_comp_mut_by_entity(
        &mut self,
        entity_key: EntityKey,
        kind_key: K,
    ) -> Option<impl Iterator<Item = &mut C>> {
        let comp_metas = self.comp_metas.get_mut(&kind_key)?;

        let relation_1 = self.relation_1.get(&(entity_key, kind_key))?;

        // UNSAFE: allow double mutable borrow temporarily
        let iter = relation_1.iter().map(|(_, slab_key)| {
            let comp_meta = comp_metas.get_mut(*slab_key as usize).check();
            let comp = &mut comp_meta.inner;
            unsafe { &mut *(comp as *mut C) }
        });

        Some(iter)
    }

    /// Clear all entities and components.
    ///
    /// # Examples
    ///
    /// ```
    /// #[derive(Debug, Clone, PartialEq, Eq, strum_macros::EnumDiscriminants)]
    /// #[strum_discriminants(name(CompKind))]
    /// #[strum_discriminants(derive(Hash))]
    /// enum Comp {
    ///     I32(i32),
    ///     Unit(()),
    /// }
    ///
    /// let mut ecs = ecs_tiny::ECS::<Comp, CompKind>::new();
    /// let entity_key = ecs.insert_entity();
    /// let comp_key = ecs.insert_comp(entity_key, Comp::I32(42)).unwrap();
    /// ecs.clear();
    /// ```
    pub fn clear(&mut self) {
        self.entities.clear();
        self.comp_metas.clear();
        self.relation_0.clear();
        self.relation_1.clear();
    }
}

/// A trait for easily invoking unrecoverable integrity errors.
trait IntegrityCheck<T> {
    fn check(self) -> T;
}

impl<T> IntegrityCheck<T> for Option<T> {
    fn check(self) -> T {
        self.expect("integrity check")
    }
}
