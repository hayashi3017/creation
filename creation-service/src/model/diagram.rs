use std::fmt::Debug;

use serde::{Deserialize, Serialize};
use sqlx::{
    prelude::Type,
    types::chrono::{DateTime, Utc},
};

#[allow(non_snake_case)]
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Diagram {
    pub id: usize,
    pub name: String,
    pub kind: DiagramKind,
    pub description: Option<String>,
    pub createdAt: DateTime<Utc>,
    pub updatedAt: DateTime<Utc>,
    pub deletedAt: Option<DateTime<Utc>>,
}

#[derive(Deserialize, Serialize, Type)]
#[sqlx(type_name = "diagram_kind")]
#[sqlx(rename_all = "lowercase")]
pub enum DiagramKind {
    FamilyTree,
    Correlation,
}

impl Debug for DiagramKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FamilyTree => write!(f, "FAMILY TREE"),
            _ => unimplemented!(),
        }
    }
}

impl Clone for DiagramKind {
    fn clone(&self) -> Self {
        match self {
            Self::FamilyTree => Self::FamilyTree,
            _ => unimplemented!(),
        }
    }
}

// impl From<String> for DiagramKind {
//     fn from(value: String) -> Self {
//         match value.as_str() {
//             "family_tree" => Self::FamilyTree,
//             _ => unreachable!(),
//         }
//     }
// }

// impl Type<MySql> for DiagramKind {
//     fn type_info() -> MySqlTypeInfo {
//         <i32 as Type<MySql>>::type_info()
//     }
// }

// impl<'q> Encode<'q, MySql> for DiagramKind {
//     fn encode_by_ref(&self, buf: &mut Vec<u8>) -> sqlx::encode::IsNull {
//         let v = match self {
//             DiagramKind::FamilyTree => 1,
//         };
//         <i32 as Encode<MySql>>::encode(v, buf)
//     }
// }

// impl<'r> Decode<'r, MySql> for DiagramKind {
//     fn decode(value: MySqlValueRef<'r>) -> Result<Self, Box<dyn Error + 'static + Send + Sync>> {
//         let v = <i32 as Decode<MySql>>::decode(value)?;
//         match v {
//             1 => Ok(DiagramKind::FamilyTree),
//             _ => Err("invalid value for DiagramKind".into()),
//         }
//     }
// }

#[derive(Debug, Deserialize)]
pub struct GetDiagramsSchema {}

#[derive(Debug, Deserialize)]
pub struct CreateDiagramSchema {
    pub name: String,
    pub kind: DiagramKind,
    pub description: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateDiagramSchema {
    pub name: String,
    pub kind: DiagramKind,
    pub description: String,
}

pub struct DeleteDiagramSchema {
    pub id: usize,
}
