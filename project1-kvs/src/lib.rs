pub struct KvStore {

}

impl KvStore {
    pub fn new() -> KvStore {
        KvStore {}
    }

    pub fn get(&self, key: String) -> Option<String> {
        panic!("get")
    }

    pub fn set(&mut self, key: String, value: String) {
        panic!("set");
    }

    pub fn remove(&mut self, key: String) {
        panic!("remove");
    }

}
