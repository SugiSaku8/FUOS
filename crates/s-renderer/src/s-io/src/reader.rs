pub trait Reader<T> {
    fn read(path: &str) -> anyhow::Result<T>;
}
