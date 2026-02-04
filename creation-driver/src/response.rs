use creation_service::model::user::FilteredUser;
use serde::Serialize;

#[derive(Serialize, Debug)]
#[allow(dead_code)]
pub struct UserData {
    pub user: FilteredUser,
}

#[derive(Serialize, Debug)]
#[allow(dead_code)]
pub struct UserResponse {
    pub status: String,
    pub data: UserData,
}
