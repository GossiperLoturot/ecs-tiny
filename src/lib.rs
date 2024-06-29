//! # ecs-tiny
//!
//! A minimal ECS supporting entity and component insertion/removal, association, and single-type iteration.
//!
//! # Usages
//!
//! ```
//! // Create new ecs instance and inserts new entity:
//!
//! let mut ecs = ecs_tiny::ECS::new();
//!
//! let entity_key0 = ecs.insert_entity();
//! let entity_key1 = ecs.insert_entity();
//!
//! // Inserts new component associated with specified entity:
//!
//! let comp_key0 = ecs.insert_comp(entity_key0, 42).unwrap();
//! let comp_key1 = ecs.insert_comp(entity_key0, 63).unwrap();
//! let comp_key2 = ecs.insert_comp(entity_key1, 42).unwrap();
//! let comp_key3 = ecs.insert_comp(entity_key1, ()).unwrap();
//!
//! // Iterates over all components associated with specified entity:
//!
//! for comp in ecs.iter_comp_mut_by_entity::<i32>(entity_key0).unwrap() {
//!     *comp += 1;
//! }
//!
//! // Iterates over all components of specified type (single type only):
//!
//! for comp in ecs.iter_comp_mut::<i32>().unwrap() {
//!     *comp += 1;
//! }
//!
//! // Removes specified component:
//!
//! ecs.remove_comp::<i32>(comp_key0).unwrap();
//!
//! // Removes specified entity:
//!
//! ecs.remove_entity(entity_key1).unwrap();
//! ```

type EntityKey = u32;

type CompKey = (u32, u32);

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
/// let mut ecs = ecs_tiny::ECS::new();
///
/// let entity_key = ecs.insert_entity();
///
/// let comp_key0 = ecs.insert_comp(entity_key, 42).unwrap();
/// let comp_key1 = ecs.insert_comp(entity_key, 63).unwrap();
///
/// for comp in ecs.iter_comp_mut::<i32>().unwrap() {
///     *comp += 1;
/// }
/// ```
pub struct ECS<C, K> {
    entities: slab::Slab<()>,
    comp_metas: ahash::AHashMap<u32, slab::Slab<CompMeta<C>>>,
    relation_0: ahash::AHashMap<EntityKey, slab::Slab<(u32, u32)>>,
    relation_1: ahash::AHashMap<(EntityKey, u32), slab::Slab<u32>>,
    _phantom: std::marker::PhantomData<K>,
}

impl<C, K> Default for ECS<C, K> {
    fn default() -> Self {
        Self {
            entities: Default::default(),
            comp_metas: Default::default(),
            relation_0: Default::default(),
            relation_1: Default::default(),
            _phantom: Default::default(),
        }
    }
}

impl<C, K> ECS<C, K>
where
    C: AsRef<K>,
    K: AsRef<u32>,
{
    /// Create a new ECS instance.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut ecs = ecs_tiny::ECS::new();
    /// ```
    pub fn new() -> Self {
        Default::default()
    }

    /// Insert a new entity and return the corresponding entity key.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut ecs = ecs_tiny::ECS::new();
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
    /// let mut ecs = ecs_tiny::ECS::new();
    /// let entity_key = ecs.insert_entity();
    /// ecs.remove_entity(entity_key).unwrap();
    /// ```
    pub fn remove_entity(&mut self, entity_key: EntityKey) -> Option<()> {
        self.entities.try_remove(entity_key as usize)?;

        if let Some(relation_0) = self.relation_0.remove(&entity_key) {
            for (_, (type_key, slab_key)) in relation_0 {
                let comp_meta = self
                    .comp_metas
                    .get_mut(&type_key)
                    .check()
                    .try_remove(slab_key as usize)
                    .check();

                self.relation_1
                    .get_mut(&(entity_key, type_key))
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
    /// let mut ecs = ecs_tiny::ECS::new();
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
    /// let mut ecs = ecs_tiny::ECS::new();
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
    /// let mut ecs = ecs_tiny::ECS::new();
    /// let entity_key = ecs.insert_entity();
    /// let comp_key = ecs.insert_comp(entity_key, 42).unwrap();
    /// ```
    pub fn insert_comp(&mut self, entity_key: EntityKey, comp: C) -> Option<CompKey> {
        self.entities.get(entity_key as usize)?;

        let kind_key = *comp.as_ref().as_ref();

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
    /// let mut ecs = ecs_tiny::ECS::new();
    /// let entity_key = ecs.insert_entity();
    /// let comp_key = ecs.insert_comp(entity_key, 42).unwrap();
    /// let comp = ecs.remove_comp::<i32>(comp_key).unwrap();
    ///
    /// assert_eq!(comp, 42);
    /// ```
    pub fn remove_comp(&mut self, comp_key: CompKey) -> Option<C> {
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
    /// let mut ecs = ecs_tiny::ECS::new();
    /// let entity_key = ecs.insert_entity();
    /// let comp_key = ecs.insert_comp(entity_key, 42).unwrap();
    /// let comp = ecs.get_comp::<i32>(comp_key).unwrap();
    ///
    /// assert_eq!(comp, &42);
    /// ```
    pub fn get_comp(&self, comp_key: CompKey) -> Option<&C> {
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
    /// let mut ecs = ecs_tiny::ECS::new();
    /// let entity_key = ecs.insert_entity();
    /// let comp_key = ecs.insert_comp(entity_key, 42).unwrap();
    /// let comp = ecs.get_comp_mut::<i32>(comp_key).unwrap();
    ///
    /// assert_eq!(comp, &mut 42);
    /// ```
    pub fn get_comp_mut(&mut self, comp_key: CompKey) -> Option<&mut C> {
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
    /// let mut ecs = ecs_tiny::ECS::new();
    /// let entity_key0 = ecs.insert_entity();
    /// let entity_key1 = ecs.insert_entity();
    /// ecs.insert_comp(entity_key0, 42).unwrap();
    /// ecs.insert_comp(entity_key0, 63).unwrap();
    /// ecs.insert_comp(entity_key1, 42).unwrap();
    /// let mut iter = ecs.iter_comp::<i32>().unwrap();
    ///
    /// assert_eq!(iter.next(), Some(&42));
    /// assert_eq!(iter.next(), Some(&63));
    /// assert_eq!(iter.next(), Some(&42));
    /// assert_eq!(iter.next(), None);
    /// ```
    pub fn iter_comp(&self, kind: K) -> Option<impl Iterator<Item = &C>> {
        let kind_key = *kind.as_ref();

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
    /// let mut ecs = ecs_tiny::ECS::new();
    /// let entity_key0 = ecs.insert_entity();
    /// let entity_key1 = ecs.insert_entity();
    /// ecs.insert_comp(entity_key0, 42).unwrap();
    /// ecs.insert_comp(entity_key0, 63).unwrap();
    /// ecs.insert_comp(entity_key1, 42).unwrap();
    /// let mut iter = ecs.iter_comp_mut::<i32>().unwrap();
    ///
    /// assert_eq!(iter.next(), Some(&mut 42));
    /// assert_eq!(iter.next(), Some(&mut 63));
    /// assert_eq!(iter.next(), Some(&mut 42));
    /// assert_eq!(iter.next(), None);
    /// ```
    pub fn iter_comp_mut(&mut self, kind: K) -> Option<impl Iterator<Item = &mut C>> {
        let kind_key = *kind.as_ref();

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
    /// let mut ecs = ecs_tiny::ECS::new();
    /// let entity_key0 = ecs.insert_entity();
    /// let entity_key1 = ecs.insert_entity();
    /// let comp_key0 = ecs.insert_comp(entity_key0, 42).unwrap();
    /// let comp_key1 = ecs.insert_comp(entity_key0, 63).unwrap();
    /// let comp_key2 = ecs.insert_comp(entity_key1, 42).unwrap();
    /// let entity_key = ecs.get_entity_by_comp(comp_key0).unwrap();
    ///
    /// assert_eq!(entity_key, entity_key0);
    /// ```
    pub fn get_entity_by_comp(&self, comp_key: CompKey) -> Option<EntityKey> {
        let (type_key, slab_key) = comp_key;

        let comp_metas = self.comp_metas.get(&type_key)?;
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
    /// let mut ecs = ecs_tiny::ECS::new();
    /// let entity_key0 = ecs.insert_entity();
    /// let entity_key1 = ecs.insert_entity();
    /// ecs.insert_comp(entity_key0, 42).unwrap();
    /// ecs.insert_comp(entity_key0, 63).unwrap();
    /// ecs.insert_comp(entity_key1, 42).unwrap();
    /// let mut iter = ecs.iter_comp_by_entity::<i32>(entity_key0).unwrap();
    ///
    /// assert_eq!(iter.next(), Some(&42));
    /// assert_eq!(iter.next(), Some(&63));
    /// assert_eq!(iter.next(), None);
    /// ```
    pub fn iter_comp_by_entity(
        &self,
        entity_key: EntityKey,
        kind: K,
    ) -> Option<impl Iterator<Item = &C>> {
        let kind_key = *kind.as_ref();

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
    /// let mut ecs = ecs_tiny::ECS::new();
    /// let entity_key0 = ecs.insert_entity();
    /// let entity_key1 = ecs.insert_entity();
    /// ecs.insert_comp(entity_key0, 42).unwrap();
    /// ecs.insert_comp(entity_key0, 63).unwrap();
    /// ecs.insert_comp(entity_key1, 42).unwrap();
    /// let mut iter = ecs.iter_comp_mut_by_entity::<i32>(entity_key0).unwrap();
    ///
    /// assert_eq!(iter.next(), Some(&mut 42));
    /// assert_eq!(iter.next(), Some(&mut 63));
    /// assert_eq!(iter.next(), None);
    /// ```
    pub fn iter_comp_mut_by_entity(
        &mut self,
        entity_key: EntityKey,
        kind: K,
    ) -> Option<impl Iterator<Item = &mut C>> {
        let kind_key = *kind.as_ref();

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
    /// let mut ecs = ecs_tiny::ECS::new();
    /// let entity_key = ecs.insert_entity();
    /// let comp_key = ecs.insert_comp(entity_key, 42).unwrap();
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
