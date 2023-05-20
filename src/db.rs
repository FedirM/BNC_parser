

use mysql::{Pool, PooledConn};

const CONNECTION_STRING: &str = "mysql://root:abcde@localhost:3306/bnc";


#[derive(Debug, PartialEq, Eq)]
pub struct NewEntry {
    pub stem: String,
    pub form: String,
    pub pos: String,
}

pub struct Connector {
    pool: Option<Pool>

}

impl Default for Connector {
    fn default() -> Self {
        Connector { pool: Option::None }
    }
}

impl Connector {
    pub fn new() -> Self {
        Self { ..Default::default() }
    }

    pub fn pool(&mut self) -> Result<&Pool, mysql::Error> {
        if self.pool.is_some() {
            Ok(self.pool.as_ref().unwrap())
        } else {
            let pool = Pool::new(CONNECTION_STRING)?;
            self.pool = Some(pool);
            Ok(self.pool.as_ref().unwrap())
        }
    }

    pub fn connection(&mut self) -> Result<PooledConn, mysql::Error> {
        match self.pool() {
            Ok(pool) => pool.get_conn(),
            Err(err) => Err(err)
        }
    }
}