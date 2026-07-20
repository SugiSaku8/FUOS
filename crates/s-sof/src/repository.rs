pub trait Repository<T> {
    fn load(&self) -> anyhow::Result<T>;

    fn save(&self, value: &T) -> anyhow::Result<()>;
}
