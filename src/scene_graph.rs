use hashbrown::HashMap;

use crate::entity::Entity;

#[derive(Clone, Debug)]
pub(crate) struct SceneGraph {
    pub(crate) entities_with_names: HashMap<String, Entity>,
}

impl SceneGraph {
    pub(crate) fn new() -> Self {
        Self {
            entities_with_names: HashMap::new(),
        }
    }

    pub(crate) fn insert(&mut self, key: &str, value: Entity) {
        self.entities_with_names.insert(key.to_string(), value);
    }

    pub(crate) fn on_update(&mut self, exec_time: f32) {
        self.entities_with_names.iter_mut().for_each(|(_, entity)| {
            entity.on_update(exec_time);
        });
    }
}
