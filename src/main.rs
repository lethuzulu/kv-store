use kv_store::store::KvStore;

fn main() {
    let store = KvStore::new("store.log").unwrap();
    println!("store {:?}", store);
}
