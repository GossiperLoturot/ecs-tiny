trait AnySlab {
    fn as_any(&self) -> &dyn std::any::Any;
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
    fn remove(&mut self, key: usize) -> Option<()>;
}

impl<T: 'static> AnySlab for slab::Slab<T> {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn remove(&mut self, key: usize) -> Option<()> {
        self.try_remove(key).map(|_| ())
    }
}

struct EntityMeta {
    comp_keys: slab::Slab<(std::any::TypeId, u32)>,
}

struct CompMeta<T> {
    comp: T,
    entity_key: u32,
    inv_comp_key: u32,
}

pub struct ECS {
    entities: slab::Slab<EntityMeta>,
    comps: ahash::AHashMap<std::any::TypeId, Box<dyn AnySlab>>,
}

impl ECS {
    pub fn new() -> Self {
        Self {
            entities: Default::default(),
            comps: Default::default(),
        }
    }

    pub fn insert_entity(&mut self) -> Result<u32, ECSError> {
        let meta = EntityMeta {
            comp_keys: Default::default(),
        };
        let key = self.entities.insert(meta) as u32;

        Ok(key)
    }

    pub fn remove_entity(&mut self, entity_key: u32) -> Result<(), ECSError> {
        let entity_meta = self
            .entities
            .try_remove(entity_key as usize)
            .ok_or(ECSError::NotFound)?;

        for (_, (type_id, comp_key)) in entity_meta.comp_keys {
            self.comps
                .get_mut(&type_id)
                .check()
                .remove(comp_key as usize)
                .check();
        }

        Ok(())
    }

    pub fn get_entity(&self, entity_key: u32) -> Result<(), ECSError> {
        self.entities
            .get(entity_key as usize)
            .ok_or(ECSError::NotFound)?;

        Ok(())
    }

    pub fn insert_comp<T: 'static>(&mut self, entity_key: u32, comp: T) -> Result<u32, ECSError> {
        let type_id = std::any::TypeId::of::<T>();

        let comps = self
            .comps
            .entry(type_id)
            .or_insert_with(|| Box::new(slab::Slab::<CompMeta<T>>::new()))
            .as_any_mut()
            .downcast_mut::<slab::Slab<CompMeta<T>>>()
            .check();

        let comp_key = comps.vacant_key() as u32;

        let entity_meta = self
            .entities
            .get_mut(entity_key as usize)
            .ok_or(ECSError::NotFound)?;
        let inv_comp_key = entity_meta.comp_keys.insert((type_id, comp_key)) as u32;

        let comp_meta = CompMeta {
            comp,
            entity_key,
            inv_comp_key,
        };
        comps.insert(comp_meta);

        Ok(comp_key)
    }

    pub fn remove_comp<T: 'static>(&mut self, comp_key: u32) -> Result<T, ECSError> {
        let type_id = std::any::TypeId::of::<T>();

        let comps = self
            .comps
            .get_mut(&type_id)
            .ok_or(ECSError::NotFound)?
            .as_any_mut()
            .downcast_mut::<slab::Slab<CompMeta<T>>>()
            .check();

        let comp_meta = comps
            .try_remove(comp_key as usize)
            .ok_or(ECSError::NotFound)?;

        let entity_meta = self.entities.get_mut(comp_meta.entity_key as usize).check();
        entity_meta
            .comp_keys
            .try_remove(comp_meta.inv_comp_key as usize)
            .check();

        Ok(comp_meta.comp)
    }

    pub fn get_comp<T: 'static>(&mut self, comp_key: u32) -> Result<&T, ECSError> {
        let type_id = std::any::TypeId::of::<T>();

        let comps = self
            .comps
            .get(&type_id)
            .ok_or(ECSError::NotFound)?
            .as_any()
            .downcast_ref::<slab::Slab<CompMeta<T>>>()
            .check();

        let comp_meta = comps.get(comp_key as usize).ok_or(ECSError::NotFound)?;

        Ok(&comp_meta.comp)
    }

    pub fn get_entity_by_comp<T: 'static>(&mut self, comp_key: u32) -> Result<u32, ECSError> {
        let type_id = std::any::TypeId::of::<T>();

        let comps = self
            .comps
            .get(&type_id)
            .ok_or(ECSError::NotFound)?
            .as_any()
            .downcast_ref::<slab::Slab<CompMeta<T>>>()
            .check();

        let comp_meta = comps.get(comp_key as usize).ok_or(ECSError::NotFound)?;

        Ok(comp_meta.entity_key)
    }

    pub fn get_comp_by_entity<T: 'static>(&self, entity_key: u32) -> Result<u32, ECSError> {
        let type_id = std::any::TypeId::of::<T>();

        let entity_meta = self
            .entities
            .get(entity_key as usize)
            .ok_or(ECSError::NotFound)?;

        let comp_key = entity_meta
            .comp_keys
            .iter()
            .find(|(_, (t, _))| *t == type_id)
            .map(|(_, (_, comp_key))| *comp_key)
            .ok_or(ECSError::NotFound)?;

        Ok(comp_key)
    }
}

#[derive(Debug)]
pub enum ECSError {
    NotFound,
}

impl std::fmt::Display for ECSError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ECSError::NotFound => write!(f, "not found error"),
        }
    }
}

impl std::error::Error for ECSError {}

trait IntegrityCheck<T> {
    fn check(self) -> T;
}

impl<T> IntegrityCheck<T> for Option<T> {
    fn check(self) -> T {
        self.expect("integrity check")
    }
}

impl<T, U: std::error::Error> IntegrityCheck<T> for Result<T, U> {
    fn check(self) -> T {
        self.expect("integrity check")
    }
}
