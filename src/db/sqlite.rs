pub trait Table {
    fn create_table(&self) -> Result<(), String>;
    fn drop_table(&self) -> Result<(), String>;
}

struct UserTable {}
