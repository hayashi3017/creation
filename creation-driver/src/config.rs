#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub jwt_secret: String,
    pub jwt_expires_in: String,
    pub jwt_maxage: i32,
    pub runtime_mode: RuntimeMode,
}

impl Config {
    pub fn init() -> Config {
        let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let jwt_secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");
        let jwt_expires_in = std::env::var("JWT_EXPIRED_IN").expect("JWT_EXPIRED_IN must be set");
        let jwt_maxage = std::env::var("JWT_MAXAGE").expect("JWT_MAXAGE must be set");
        let runtime_mode = std::env::var("RUNTIME_MODE").expect("RUNTIME_MODE must be set");
        Config {
            database_url,
            jwt_secret,
            jwt_expires_in,
            jwt_maxage: jwt_maxage.parse::<i32>().unwrap(),
            runtime_mode: runtime_mode.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Copy)]
pub enum RuntimeMode {
    Release,
    Debug,
}

impl From<String> for RuntimeMode {
    fn from(value: String) -> Self {
        match value.as_str() {
            "release" => RuntimeMode::Release,
            "debug" => RuntimeMode::Debug,
            _ => unreachable!(),
        }
    }
}
