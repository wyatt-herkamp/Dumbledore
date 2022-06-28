use std::any::TypeId;

pub(crate) struct TypeIdSet<V>(pub(crate) Box<[(TypeId, V)]>);

impl<V> TypeIdSet<V> {
    pub fn new<Content>(contents: Content) -> Self
        where Content: Iterator<Item=(TypeId, V)> {
        let mut contents = contents.collect::<Box<[_]>>();
        contents.sort_unstable_by_key(|&(id, _)| id);

        TypeIdSet(contents)
    }

    pub fn search(&self, id: &TypeId) -> Option<usize> {
        self.0.binary_search_by_key(id, |(d, _)| *d).ok()
    }
    pub fn get(&self, id: &TypeId) -> Option<&V> {
        self.search(id).map(|i| &self.0[i].1)
    }

    pub fn contains_key(&self, id: &TypeId) -> bool {
        self.search(id).is_some()
    }
}


