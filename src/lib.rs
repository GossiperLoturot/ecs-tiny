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
//! // Register new component type:
//!
//! ecs.register::<i32>().unwrap();
//! ecs.register::<()>().unwrap();
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

type CompKey = (std::any::TypeId, u32);

struct CompRow<T> {
    comp: T,
    entity_key: u32,
    ref_0_row_key: u32,
    ref_1_row_key: u32,
}

const ALLOC_SIZE: usize = std::mem::size_of::<slab::Slab<CompRow<()>>>();

struct CompColumn {
    comp_rows: stack_any::StackAny<ALLOC_SIZE>,
    get_row_fn: fn(&Self, u32) -> Option<CompRow<()>>,
    remove_row_fn: fn(&mut Self, u32) -> Option<CompRow<()>>,
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
/// ecs.register::<i32>().unwrap();
///
/// let comp_key0 = ecs.insert_comp(entity_key, 42).unwrap();
/// let comp_key1 = ecs.insert_comp(entity_key, 63).unwrap();
///
/// for comp in ecs.iter_comp_mut::<i32>().unwrap() {
///     *comp += 1;
/// }
/// ```
#[derive(Default)]
pub struct ECS {
    entities: slab::Slab<()>,
    comp_cols: ahash::AHashMap<std::any::TypeId, CompColumn>,
    ref_0_cols: ahash::AHashMap<EntityKey, slab::Slab<(std::any::TypeId, u32)>>,
    ref_1_cols: ahash::AHashMap<(EntityKey, std::any::TypeId), slab::Slab<u32>>,
}

impl ECS {
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

        if let Some(ref_0_col) = self.ref_0_cols.remove(&entity_key) {
            for (_, (type_key, row_key)) in ref_0_col {
                let comp_col = self.comp_cols.get_mut(&type_key).unwrap();
                let comp_row = (comp_col.remove_row_fn)(comp_col, row_key).unwrap();

                self.ref_1_cols
                    .get_mut(&(entity_key, type_key))
                    .unwrap()
                    .try_remove(comp_row.ref_1_row_key as usize)
                    .unwrap();
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

    /// Register component type.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut ecs = ecs_tiny::ECS::new();
    /// let entity_key = ecs.insert_entity();
    /// ecs.register::<i32>().unwrap();
    /// let comp_key = ecs.insert_comp(entity_key, 42).unwrap();
    /// ```
    pub fn register<T>(&mut self) -> Option<()>
    where
        T: std::any::Any,
    {
        let type_key = std::any::TypeId::of::<T>();

        if self.comp_cols.contains_key(&type_key) {
            return None;
        }

        let comp_col = CompColumn {
            comp_rows: stack_any::StackAny::try_new(slab::Slab::<CompRow<T>>::new()).unwrap(),
            get_row_fn: |comp_col, row_key| {
                let comp_row = comp_col
                    .comp_rows
                    .downcast_ref::<slab::Slab<CompRow<T>>>()
                    .unwrap()
                    .get(row_key as usize)?;
                Some(CompRow {
                    comp: (),
                    entity_key: comp_row.entity_key,
                    ref_0_row_key: comp_row.ref_0_row_key,
                    ref_1_row_key: comp_row.ref_1_row_key,
                })
            },
            remove_row_fn: |comp_col, row_key| {
                let comp_row = comp_col
                    .comp_rows
                    .downcast_mut::<slab::Slab<CompRow<T>>>()
                    .unwrap()
                    .try_remove(row_key as usize)?;
                Some(CompRow {
                    comp: (),
                    entity_key: comp_row.entity_key,
                    ref_0_row_key: comp_row.ref_0_row_key,
                    ref_1_row_key: comp_row.ref_1_row_key,
                })
            },
        };
        self.comp_cols.insert(type_key, comp_col);

        Some(())
    }

    /// Unregister component type.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut ecs = ecs_tiny::ECS::new();
    /// let entity_key = ecs.insert_entity();
    /// ecs.register::<i32>().unwrap();
    /// let comp_key = ecs.insert_comp(entity_key, 42).unwrap();
    /// ecs.unregister::<i32>().unwrap();
    /// ```
    pub fn unregister<T>(&mut self) -> Option<()>
    where
        T: std::any::Any,
    {
        let type_key = std::any::TypeId::of::<T>();

        if !self.comp_cols.contains_key(&type_key) {
            return None;
        }

        self.comp_cols.remove(&type_key);

        Some(())
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
    /// ecs.register::<i32>().unwrap();
    /// let comp_key = ecs.insert_comp(entity_key, 42).unwrap();
    /// ```
    pub fn insert_comp<T>(&mut self, entity_key: EntityKey, comp: T) -> Option<CompKey>
    where
        T: std::any::Any,
    {
        self.entities.get(entity_key as usize)?;

        let type_key = std::any::TypeId::of::<T>();

        let comp_rows = self
            .comp_cols
            .get_mut(&type_key)?
            .comp_rows
            .downcast_mut::<slab::Slab<CompRow<T>>>()
            .unwrap();

        let row_key = comp_rows.vacant_key() as u32;

        let ref_0_row_key = self
            .ref_0_cols
            .entry(entity_key)
            .or_default()
            .insert((type_key, row_key)) as u32;

        let ref_1_row_key = self
            .ref_1_cols
            .entry((entity_key, type_key))
            .or_default()
            .insert(row_key) as u32;

        comp_rows.insert(CompRow {
            comp,
            entity_key,
            ref_0_row_key,
            ref_1_row_key,
        });

        Some((type_key, row_key))
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
    /// ecs.register::<i32>().unwrap();
    /// let comp_key = ecs.insert_comp(entity_key, 42).unwrap();
    /// let comp = ecs.remove_comp::<i32>(comp_key).unwrap();
    ///
    /// assert_eq!(comp, 42);
    /// ```
    pub fn remove_comp<T>(&mut self, comp_key: CompKey) -> Option<T>
    where
        T: std::any::Any,
    {
        let (type_key, row_key) = comp_key;

        if type_key != std::any::TypeId::of::<T>() {
            return None;
        }

        let comp_rows = self
            .comp_cols
            .get_mut(&type_key)?
            .comp_rows
            .downcast_mut::<slab::Slab<CompRow<T>>>()
            .unwrap();
        let comp_row = comp_rows.try_remove(row_key as usize)?;

        self.ref_0_cols
            .get_mut(&comp_row.entity_key)
            .unwrap()
            .try_remove(comp_row.ref_0_row_key as usize)
            .unwrap();

        self.ref_1_cols
            .get_mut(&(comp_row.entity_key, type_key))
            .unwrap()
            .try_remove(comp_row.ref_1_row_key as usize)
            .unwrap();

        Some(comp_row.comp)
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
    /// ecs.register::<i32>().unwrap();
    /// let comp_key = ecs.insert_comp(entity_key, 42).unwrap();
    /// let comp = ecs.get_comp::<i32>(comp_key).unwrap();
    ///
    /// assert_eq!(comp, &42);
    /// ```
    pub fn get_comp<T>(&self, comp_key: CompKey) -> Option<&T>
    where
        T: std::any::Any,
    {
        let (type_key, row_key) = comp_key;

        if type_key != std::any::TypeId::of::<T>() {
            return None;
        }

        let comp_rows = self
            .comp_cols
            .get(&type_key)?
            .comp_rows
            .downcast_ref::<slab::Slab<CompRow<T>>>()
            .unwrap();
        let comp_row = comp_rows.get(row_key as usize)?;

        Some(&comp_row.comp)
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
    /// ecs.register::<i32>().unwrap();
    /// let comp_key = ecs.insert_comp(entity_key, 42).unwrap();
    /// let comp = ecs.get_comp_mut::<i32>(comp_key).unwrap();
    ///
    /// assert_eq!(comp, &mut 42);
    /// ```
    pub fn get_comp_mut<T>(&mut self, comp_key: CompKey) -> Option<&mut T>
    where
        T: std::any::Any,
    {
        let (type_key, row_key) = comp_key;

        if type_key != std::any::TypeId::of::<T>() {
            return None;
        }

        let comp_rows = self
            .comp_cols
            .get_mut(&type_key)?
            .comp_rows
            .downcast_mut::<slab::Slab<CompRow<T>>>()
            .unwrap();
        let comp = comp_rows.get_mut(row_key as usize)?;

        Some(&mut comp.comp)
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
    /// ecs.register::<i32>().unwrap();
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
    pub fn iter_comp<T>(&self) -> Option<impl Iterator<Item = &T>>
    where
        T: std::any::Any,
    {
        let type_key = std::any::TypeId::of::<T>();

        let comp_rows = self
            .comp_cols
            .get(&type_key)?
            .comp_rows
            .downcast_ref::<slab::Slab<CompRow<T>>>()
            .unwrap();
        let iter = comp_rows.iter().map(|(_, comp_row)| &comp_row.comp);

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
    /// ecs.register::<i32>().unwrap();
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
    pub fn iter_comp_mut<T>(&mut self) -> Option<impl Iterator<Item = &mut T>>
    where
        T: std::any::Any,
    {
        let type_key = std::any::TypeId::of::<T>();

        let comp_rows = self
            .comp_cols
            .get_mut(&type_key)?
            .comp_rows
            .downcast_mut::<slab::Slab<CompRow<T>>>()
            .unwrap();
        let iter = comp_rows.iter_mut().map(|(_, comp_row)| &mut comp_row.comp);

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
    /// ecs.register::<i32>().unwrap();
    /// let comp_key0 = ecs.insert_comp(entity_key0, 42).unwrap();
    /// let comp_key1 = ecs.insert_comp(entity_key0, 63).unwrap();
    /// let comp_key2 = ecs.insert_comp(entity_key1, 42).unwrap();
    /// let entity_key = ecs.get_entity_by_comp(comp_key0).unwrap();
    ///
    /// assert_eq!(entity_key, entity_key0);
    /// ```
    pub fn get_entity_by_comp(&self, comp_key: CompKey) -> Option<EntityKey> {
        let (type_key, row_key) = comp_key;

        let comp_col = self.comp_cols.get(&type_key)?;
        let comp_row = (comp_col.get_row_fn)(comp_col, row_key)?;

        Some(comp_row.entity_key)
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
    /// ecs.register::<i32>().unwrap();
    /// ecs.insert_comp(entity_key0, 42).unwrap();
    /// ecs.insert_comp(entity_key0, 63).unwrap();
    /// ecs.insert_comp(entity_key1, 42).unwrap();
    /// let mut iter = ecs.iter_comp_by_entity::<i32>(entity_key0).unwrap();
    ///
    /// assert_eq!(iter.next(), Some(&42));
    /// assert_eq!(iter.next(), Some(&63));
    /// assert_eq!(iter.next(), None);
    /// ```
    pub fn iter_comp_by_entity<T>(&self, entity_key: EntityKey) -> Option<impl Iterator<Item = &T>>
    where
        T: std::any::Any,
    {
        let type_key = std::any::TypeId::of::<T>();

        let comp_rows = self
            .comp_cols
            .get(&type_key)?
            .comp_rows
            .downcast_ref::<slab::Slab<CompRow<T>>>()
            .unwrap();

        let ref_1_col = self.ref_1_cols.get(&(entity_key, type_key))?;

        let iter = ref_1_col
            .iter()
            .map(|(_, row_key)| &comp_rows.get(*row_key as usize).unwrap().comp);

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
    /// ecs.register::<i32>().unwrap();
    /// ecs.insert_comp(entity_key0, 42).unwrap();
    /// ecs.insert_comp(entity_key0, 63).unwrap();
    /// ecs.insert_comp(entity_key1, 42).unwrap();
    /// let mut iter = ecs.iter_comp_mut_by_entity::<i32>(entity_key0).unwrap();
    ///
    /// assert_eq!(iter.next(), Some(&mut 42));
    /// assert_eq!(iter.next(), Some(&mut 63));
    /// assert_eq!(iter.next(), None);
    /// ```
    pub fn iter_comp_mut_by_entity<T>(
        &mut self,
        entity_key: EntityKey,
    ) -> Option<impl Iterator<Item = &mut T>>
    where
        T: std::any::Any,
    {
        let type_key = std::any::TypeId::of::<T>();

        let comp_rows = self
            .comp_cols
            .get_mut(&type_key)?
            .comp_rows
            .downcast_mut::<slab::Slab<CompRow<T>>>()
            .unwrap();

        let ref_1_col = self.ref_1_cols.get(&(entity_key, type_key))?;

        // UNSAFE: allow double mutable borrow temporarily
        let iter = ref_1_col
            .iter()
            .map(|(_, row_key)| &mut comp_rows.get_mut(*row_key as usize).unwrap().comp as *mut T)
            .map(|ptr| unsafe { &mut *ptr });

        Some(iter)
    }

    /// Clear all entities and components.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut ecs = ecs_tiny::ECS::new();
    /// let entity_key = ecs.insert_entity();
    /// ecs.register::<i32>().unwrap();
    /// let comp_key = ecs.insert_comp(entity_key, 42).unwrap();
    /// ecs.clear();
    /// ```
    pub fn clear(&mut self) {
        self.entities.clear();
        self.comp_cols.clear();
        self.ref_0_cols.clear();
        self.ref_1_cols.clear();
    }
}
