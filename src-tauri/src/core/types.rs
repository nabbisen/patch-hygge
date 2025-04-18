use similar::DiffTag;

// use serde::{Deserialize, Serialize};
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CompareSet {
    pub old: CompareSetItem,
    pub new: CompareSetItem,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CompareSetItem {
    pub filepath: String,
    pub binary_comparison_only: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LinesDiffResponse {
    pub old_charset: String,
    pub new_charset: String,
    pub diffs: Vec<LinesDiff>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LinesDiff {
    pub diff_index: usize,
    pub diff_kind: DiffTag,
    pub lines_count: usize,
    pub old_lines: Vec<String>,
    pub new_lines: Vec<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CharsDiffResponse {
    pub diffs: Vec<CharsDiffLines>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CharsDiffLines {
    pub diff_index: usize,
    pub old_lines: Vec<Vec<CharsDiff>>,
    pub new_lines: Vec<Vec<CharsDiff>>,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CharsDiff {
    pub diff_kind: DiffTag,
    pub chars: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListDirResponse {
    pub current_dir: String,
    pub dirs: Vec<String>,
    pub files: Vec<FileAttr>,
}

#[derive(Serialize, Eq, PartialEq, Ord, PartialOrd)]
#[serde(rename_all = "camelCase")]
pub struct FileAttr {
    pub name: String, // first field is default sort key
    pub bytes_size: String,
    pub human_readable_size: String,
    pub last_modified: String,
    pub binary_comparison_only: bool,
}

#[derive(Clone, Default)]
pub struct ReadContent {
    pub charset: String,
    pub content: String,
}
