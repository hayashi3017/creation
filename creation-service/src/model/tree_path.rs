use serde::Serialize;

#[allow(non_snake_case)]
#[derive(Debug, Clone, Serialize)]
pub struct TreePath {
    pub world_id: usize,
    pub ancestor_id: usize,
    pub descendant_id: usize,
    pub depth: usize,
}

// stale な tree_path row を seed と同じ world に振り分けるための内部表現。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TreePathConnection {
    pub world_id: usize,
    pub ancestor_id: usize,
    pub descendant_id: usize,
}

#[derive(Debug, Clone)]
pub struct SyncTreePathsByEntityIdsSchema {
    pub world_id: usize,
    pub entity_ids: Vec<usize>,
}

#[derive(Debug, Clone)]
pub struct LoadStaleRelatedEntityIdsSchema {
    pub entity_ids: Vec<usize>,
}

#[derive(Debug, Clone)]
pub struct DeleteTreePathsByEntityIdsSchema {
    pub entity_ids: Vec<usize>,
}

#[derive(Debug, Clone)]
pub struct DeleteTreePathsByWorldSchema {
    pub world_id: usize,
}

#[derive(Debug, Clone)]
pub struct DeleteTreePathsForDiagramSchema {
    pub diagram_id: usize,
}

#[derive(Debug, Clone)]
pub struct CreateTreePathsSchema {
    pub tree_paths: Vec<TreePath>,
}

#[derive(Debug, Clone)]
pub struct LoadStaleRelatedConnectionsSchema {
    pub entity_ids: Vec<usize>,
}
