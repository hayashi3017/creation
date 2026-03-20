use std::fmt::{Debug, Display};

use serde::{Deserialize, Serialize};
use sqlx::prelude::Type;

pub const DIAGRAM_NAME_MAX_CHARS: usize = 255;

#[allow(non_snake_case)]
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Diagram {
    pub id: usize,
    pub name: String,
    pub kind: DiagramKind,
    pub description: Option<String>,
}

#[derive(Deserialize, Serialize, Type)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "diagram_kind")]
#[sqlx(rename_all = "snake_case")]
pub enum DiagramKind {
    FamilyTree,
    Correlation,
}

impl Debug for DiagramKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FamilyTree => write!(f, "FAMILY TREE"),
            Self::Correlation => write!(f, "CORRELATION"),
        }
    }
}

impl Clone for DiagramKind {
    fn clone(&self) -> Self {
        match self {
            Self::FamilyTree => Self::FamilyTree,
            Self::Correlation => Self::Correlation,
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
    #[serde(default)]
    pub description: Option<String>,
}

impl Display for CreateDiagramSchema {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Ok(())
    }
}

#[derive(Debug, Deserialize)]
pub struct UpdateDiagramSchema {
    pub id: usize,
    pub name: String,
    pub kind: DiagramKind,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct DeleteDiagramSchema {
    pub id: usize,
}

#[derive(Debug, Clone)]
pub struct ExistsActiveDiagramSchema {
    pub id: usize,
}
