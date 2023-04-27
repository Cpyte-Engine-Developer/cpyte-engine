use vulkanalia::{
    loader::{LibloadingLoader, LIBRARY},
    prelude::v1_0::Entry as vkEntry,
};

#[derive(Debug, Clone)]
pub(crate) struct Entry {
    pub(crate) entry: vkEntry,
}

impl Entry {
    pub(crate) fn new() -> Self {
        let entry = unsafe {
            let loader = LibloadingLoader::new(LIBRARY).unwrap();
            vkEntry::new(loader).unwrap()
        };

        Self { entry }
    }
}

impl From<Entry> for vkEntry {
    fn from(value: Entry) -> Self {
        value.entry
    }
}

impl From<&Entry> for vkEntry {
    fn from(value: &Entry) -> Self {
        value.entry.clone()
    }
}
