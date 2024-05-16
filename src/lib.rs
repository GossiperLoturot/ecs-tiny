/// A trait for operating of Slab without type annotation.
trait AnySlab {
    fn as_any(&self) -> &dyn std::any::Any;

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;

    fn try_remove(&mut self, key: usize) -> Option<()>;
}

impl<T: 'static> AnySlab for slab::Slab<T> {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn try_remove(&mut self, key: usize) -> Option<()> {
        self.try_remove(key).map(|_| ())
    }
}

type EntityKey = u32;

type CompKey = (std::any::TypeId, u32);

struct CompMeta {
    entity_key: u32,
    relation_0: u32,
    relation_1: u32,
}

/// Entity-Component-System.
///
/// # Examples
///
/// ```
/// let mut ecs = ecs_tiny::ECS::new();
/// ```
#[derive(Default)]
pub struct ECS {
    entities: slab::Slab<()>,
    comps: ahash::AHashMap<std::any::TypeId, Box<dyn AnySlab>>,
    comp_metas: ahash::AHashMap<std::any::TypeId, slab::Slab<CompMeta>>,
    relation_0: ahash::AHashMap<EntityKey, slab::Slab<(std::any::TypeId, u32)>>,
    relation_1: ahash::AHashMap<(EntityKey, std::any::TypeId), slab::Slab<u32>>,
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
    /// If the entity corresponding to the entity key is not found, return an Err(ECSError::NotFound).
    /// Otherwise, return an Ok(()).
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
                self.comps
                    .get_mut(&type_key)
                    .check()
                    .try_remove(slab_key as usize)
                    .check();

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
    /// If the entity corresponding to the entity key is not found, return an Err(ECSError::NotFound).
    /// Otherwise, return an Ok(()).
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
    /// If the entity corresponding to the entity key is not found, return an Err(ECSError::NotFound).
    /// Otherwise, return an Ok(CompKey).
    ///
    /// # Examples
    ///
    /// ```
    /// let mut ecs = ecs_tiny::ECS::new();
    /// let entity_key = ecs.insert_entity();
    /// let comp_key = ecs.insert_comp(entity_key, 42).unwrap();
    /// ```
    pub fn insert_comp<T: 'static>(&mut self, entity_key: EntityKey, comp: T) -> Option<CompKey> {
        self.entities.get(entity_key as usize)?;

        let type_key = std::any::TypeId::of::<T>();

        let comps = self
            .comps
            .entry(type_key)
            .or_insert_with(|| Box::new(slab::Slab::<T>::new()))
            .as_any_mut()
            .downcast_mut::<slab::Slab<T>>()
            .check();

        let slab_key = comps.insert(comp) as u32;

        let relation_0 = self
            .relation_0
            .entry(entity_key)
            .or_default()
            .insert((type_key, slab_key)) as u32;

        let relation_1 = self
            .relation_1
            .entry((entity_key, type_key))
            .or_default()
            .insert(slab_key) as u32;

        let comp_meta = CompMeta {
            entity_key,
            relation_0,
            relation_1,
        };
        self.comp_metas
            .entry(type_key)
            .or_default()
            .insert(comp_meta);

        Some((type_key, slab_key))
    }

    /// Remove a component with the corresponding component key and type, and return the component.
    /// If the component corresponding to the component key and type is not found, return an Err(ECSError::NotFound).
    /// Otherwise, return an Ok(T).
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
    pub fn remove_comp<T: 'static>(&mut self, comp_key: CompKey) -> Option<T> {
        let (type_key, slab_key) = comp_key;

        if type_key != std::any::TypeId::of::<T>() {
            return None;
        }

        let comps = self
            .comps
            .get_mut(&type_key)?
            .as_any_mut()
            .downcast_mut::<slab::Slab<T>>()
            .check();
        let comp = comps.try_remove(slab_key as usize)?;

        let comp_metas = self.comp_metas.get_mut(&type_key).check();
        let comp_meta = comp_metas.try_remove(slab_key as usize).check();

        self.relation_0
            .get_mut(&comp_meta.entity_key)
            .check()
            .try_remove(comp_meta.relation_0 as usize)
            .check();

        self.relation_1
            .get_mut(&(comp_meta.entity_key, type_key))
            .check()
            .try_remove(comp_meta.relation_1 as usize)
            .check();

        Some(comp)
    }

    /// Return a component with the corresponding component key and type.
    /// If the component corresponding to the component key and type is not found, return an Err(ECSError::NotFound).
    /// Otherwise, return an Ok(&T).
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
    pub fn get_comp<T: 'static>(&self, comp_key: CompKey) -> Option<&T> {
        let (type_key, slab_key) = comp_key;

        if type_key != std::any::TypeId::of::<T>() {
            return None;
        }

        let comps = self
            .comps
            .get(&type_key)?
            .as_any()
            .downcast_ref::<slab::Slab<T>>()
            .check();
        let comp = comps.get(slab_key as usize)?;

        Some(comp)
    }

    /// Return a mutable component with the corresponding component key and type.
    /// If the component corresponding to the component key and type is not found, return an Err(ECSError::NotFound).
    /// Otherwise, return an Ok(&mut T).
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
    pub fn get_comp_mut<T: 'static>(&mut self, comp_key: CompKey) -> Option<&mut T> {
        let (type_key, slab_key) = comp_key;

        if type_key != std::any::TypeId::of::<T>() {
            return None;
        }

        let comps = self
            .comps
            .get_mut(&type_key)?
            .as_any_mut()
            .downcast_mut::<slab::Slab<T>>()
            .check();
        let comp = comps.get_mut(slab_key as usize)?;

        Some(comp)
    }

    /// Return an iterator over all components of the corresponding type.
    /// If the component type is not found, return an Err(ECSError::NotFound).
    /// Otherwise, return an Ok(impl Iterator<Item = &T>).
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
    pub fn iter_comp<T: 'static>(&self) -> Option<impl Iterator<Item = &T>> {
        let type_key = std::any::TypeId::of::<T>();

        let comps = self
            .comps
            .get(&type_key)?
            .as_any()
            .downcast_ref::<slab::Slab<T>>()
            .check();
        let iter = comps.iter().map(|(_, comp)| comp);

        Some(iter)
    }

    /// Return a mutable iterator over all components of the corresponding type.
    /// If the component type is not found, return an Err(ECSError::NotFound).
    /// Otherwise, return an Ok(impl Iterator<Item = &mut T>).
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
    pub fn iter_comp_mut<T: 'static>(&mut self) -> Option<impl Iterator<Item = &mut T>> {
        let type_key = std::any::TypeId::of::<T>();

        let comps = self
            .comps
            .get_mut(&type_key)?
            .as_any_mut()
            .downcast_mut::<slab::Slab<T>>()
            .check();
        let iter = comps.iter_mut().map(|(_, comp)| comp);

        Some(iter)
    }

    /// Return an entity key with the corresponding component key.
    /// If the component corresponding to the component key is not found, return an Err(ECSError::NotFound).
    /// Otherwise, return an Ok(EntityKey).
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
    /// If the entity corresponding to the entity key and type is not found, return an Err(ECSError::NotFound).
    /// Otherwise, return an Ok(impl Iterator<Item = &T>).
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
    pub fn iter_comp_by_entity<T: 'static>(
        &self,
        entity_key: EntityKey,
    ) -> Option<impl Iterator<Item = &T>> {
        let type_key = std::any::TypeId::of::<T>();

        let comps = self
            .comps
            .get(&type_key)?
            .as_any()
            .downcast_ref::<slab::Slab<T>>()
            .check();

        let relation_1 = self.relation_1.get(&(entity_key, type_key))?;

        let iter = relation_1
            .iter()
            .map(|(_, slab_key)| comps.get(*slab_key as usize).check());

        Some(iter)
    }

    /// Return a mutable iterator over all components with the corresponding entity key and type.
    /// If the entity corresponding to the entity key and type is not found, return an Err(ECSError::NotFound).
    /// Otherwise, return an Ok(impl Iterator<Item = &mut T>).
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
    pub fn iter_comp_mut_by_entity<T: 'static>(
        &mut self,
        entity_key: EntityKey,
    ) -> Option<impl Iterator<Item = &mut T>> {
        let type_key = std::any::TypeId::of::<T>();

        let comps = self
            .comps
            .get_mut(&type_key)?
            .as_any_mut()
            .downcast_mut::<slab::Slab<T>>()
            .check();

        let relation_1 = self.relation_1.get(&(entity_key, type_key))?;

        // UNSAFE: allow double mutable borrow temporarily
        let iter = relation_1
            .iter()
            .map(|(_, slab_key)| comps.get_mut(*slab_key as usize).check() as *mut T)
            .map(|ptr| unsafe { &mut *ptr });

        Some(iter)
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
