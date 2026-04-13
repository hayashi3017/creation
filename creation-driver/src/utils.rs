use crate::config::RuntimeMode;

pub fn get_port(mode: RuntimeMode) -> String {
    if mode == RuntimeMode::Debug {
        return "8000".to_string();
        // return "8001".to_string();
    } else {
        return "8000".to_string();
    }
}
