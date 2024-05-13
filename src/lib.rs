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

#[derive(Default)]
pub struct ECS {
    entities: slab::Slab<()>,
    comps: ahash::AHashMap<std::any::TypeId, Box<dyn AnySlab>>,
    comp_metas: ahash::AHashMap<std::any::TypeId, slab::Slab<CompMeta>>,
    relation_0: ahash::AHashMap<EntityKey, slab::Slab<(std::any::TypeId, u32)>>,
    relation_1: ahash::AHashMap<(EntityKey, std::any::TypeId), slab::Slab<u32>>,
}

impl ECS {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn insert_entity(&mut self) -> Result<EntityKey, ECSError> {
        Ok(self.entities.insert(()) as u32)
    }

    pub fn remove_entity(&mut self, entity_key: EntityKey) -> Result<(), ECSError> {
        self.entities
            .try_remove(entity_key as usize)
            .ok_or(ECSError::NotFound)?;

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

        Ok(())
    }

    pub fn get_entity(&self, entity_key: EntityKey) -> Result<(), ECSError> {
        self.entities
            .get(entity_key as usize)
            .ok_or(ECSError::NotFound)?;
        Ok(())
    }

    pub fn iter_entity(&self) -> impl Iterator<Item = EntityKey> + '_ {
        self.entities.iter().map(|(key, _)| key as u32)
    }

    pub fn insert_comp<T: 'static>(
        &mut self,
        entity_key: EntityKey,
        comp: T,
    ) -> Result<CompKey, ECSError> {
        self.entities
            .get(entity_key as usize)
            .ok_or(ECSError::NotFound)?;

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

        Ok((type_key, slab_key))
    }

    pub fn remove_comp<T: 'static>(&mut self, comp_key: CompKey) -> Result<T, ECSError> {
        let (type_key, slab_key) = comp_key;

        let comps = self
            .comps
            .get_mut(&type_key)
            .ok_or(ECSError::NotFound)?
            .as_any_mut()
            .downcast_mut::<slab::Slab<T>>()
            .check();
        let comp = comps
            .try_remove(slab_key as usize)
            .ok_or(ECSError::NotFound)?;

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

        Ok(comp)
    }

    pub fn get_comp<T: 'static>(&mut self, comp_key: CompKey) -> Result<&T, ECSError> {
        let (type_key, slab_key) = comp_key;

        let comps = self
            .comps
            .get(&type_key)
            .ok_or(ECSError::NotFound)?
            .as_any()
            .downcast_ref::<slab::Slab<T>>()
            .check();
        let comp = comps.get(slab_key as usize).ok_or(ECSError::NotFound)?;

        Ok(comp)
    }

    pub fn iter_comp<T: 'static>(&self) -> Result<impl Iterator<Item = &T>, ECSError> {
        let type_key = std::any::TypeId::of::<T>();

        let comps = self
            .comps
            .get(&type_key)
            .ok_or(ECSError::NotFound)?
            .as_any()
            .downcast_ref::<slab::Slab<T>>()
            .check();
        let iter = comps.iter().map(|(_, comp)| comp);

        Ok(iter)
    }

    pub fn get_entity_by_comp(&self, comp_key: CompKey) -> Result<EntityKey, ECSError> {
        let (type_key, slab_key) = comp_key;

        let comp_metas = self.comp_metas.get(&type_key).ok_or(ECSError::NotFound)?;
        let comp_meta = comp_metas
            .get(slab_key as usize)
            .ok_or(ECSError::NotFound)?;

        Ok(comp_meta.entity_key)
    }

    pub fn iter_comp_by_entity<T: 'static>(
        &self,
        entity_key: EntityKey,
    ) -> Result<impl Iterator<Item = &T>, ECSError> {
        let type_key = std::any::TypeId::of::<T>();

        let comps = self
            .comps
            .get(&type_key)
            .ok_or(ECSError::NotFound)?
            .as_any()
            .downcast_ref::<slab::Slab<T>>()
            .check();

        let relation_1 = self
            .relation_1
            .get(&(entity_key, type_key))
            .ok_or(ECSError::NotFound)?;

        let iter = relation_1
            .iter()
            .map(|(_, slab_key)| comps.get(*slab_key as usize).check());

        Ok(iter)
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
