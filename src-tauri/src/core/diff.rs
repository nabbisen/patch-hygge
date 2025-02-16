use similar::{DiffTag, TextDiff};
use std::ops::Range;

use super::{
    str::split_lines_with_endings,
    types::{LinesDiff, ReplaceDetailLinesDiff, ReplaceDiffChars},
};

pub fn lines_diffs(old_content: &str, new_content: &str) -> Vec<LinesDiff> {
    let diff = TextDiff::configure().diff_lines(old_content, new_content);
    let old_lines: Vec<String> = split_lines_with_endings(old_content);
    let new_lines: Vec<String> = split_lines_with_endings(new_content);
    diff.ops()
        .iter()
        .map(|x| {
            let diff_kind = x.tag();
            match diff_kind {
                DiffTag::Equal => {
                    let old_range = x.old_range();
                    let lines_count = old_range.end - old_range.start;
                    let lines = old_lines[old_range.start..old_range.end].to_vec();
                    LinesDiff {
                        diff_kind,
                        lines_count,
                        old_lines: lines.to_owned(),
                        new_lines: lines,
                        replace_detail: None,
                    }
                }
                DiffTag::Delete => {
                    let old_range = x.old_range();
                    let lines_count = old_range.end - old_range.start;
                    let old_lines = old_lines[old_range.start..old_range.end].to_vec();
                    LinesDiff {
                        diff_kind,
                        lines_count,
                        old_lines,
                        new_lines: vec![],
                        replace_detail: None,
                    }
                }
                DiffTag::Insert => {
                    let new_range = x.new_range();
                    let lines_count = new_range.end - new_range.start;
                    let new_lines = new_lines[new_range.start..new_range.end].to_vec();
                    LinesDiff {
                        diff_kind,
                        lines_count,
                        old_lines: vec![],
                        new_lines,
                        replace_detail: None,
                    }
                }
                DiffTag::Replace => replace_lines_diff(
                    old_lines.as_ref(),
                    new_lines.as_ref(),
                    x.old_range().by_ref(),
                    x.new_range().by_ref(),
                    &diff_kind,
                ),
            }
        })
        .collect::<Vec<LinesDiff>>()
}

fn replace_lines_diff(
    old_lines: &Vec<String>,
    new_lines: &Vec<String>,
    old_range: &Range<usize>,
    new_range: &Range<usize>,
    diff_kind: &DiffTag,
) -> LinesDiff {
    let old_lines_count = old_range.end - old_range.start;
    let new_lines_count = new_range.end - new_range.start;
    let lines_count = if old_lines_count < new_lines_count {
        new_lines_count
    } else {
        old_lines_count
    };
    let old_lines = old_lines[old_range.start..old_range.end].to_vec();
    let new_lines = new_lines[new_range.start..new_range.end].to_vec();

    let old_str = old_lines.join("");
    let new_str = new_lines.join("");

    let replace_detail = replace_diff_lines(old_str.as_str(), new_str.as_str());
    LinesDiff {
        diff_kind: diff_kind.to_owned(),
        lines_count,
        old_lines,
        new_lines,
        replace_detail: Some(replace_detail),
    }
}

fn replace_diff_lines(old_str: &str, new_str: &str) -> ReplaceDetailLinesDiff {
    let mut old_lines: Vec<Vec<ReplaceDiffChars>> = vec![];
    let mut new_lines: Vec<Vec<ReplaceDiffChars>> = vec![];

    let mut old_line: Vec<ReplaceDiffChars> = vec![];
    let mut new_line: Vec<ReplaceDiffChars> = vec![];
    let mut old_chars = String::new();
    let mut new_chars = String::new();
    TextDiff::configure()
        .diff_chars(old_str, new_str)
        .ops()
        .iter()
        .for_each(|x| {
            let diff_kind = x.tag();
            match diff_kind {
                DiffTag::Equal => {
                    let old_range = x.old_range();
                    // let str = &old_str[old_range.start..old_range.end];
                    let (start, end) =
                        multibyte_str_byte_indices(old_str, old_range.start, old_range.end)
                            .unwrap();
                    let str = &old_str[start..end];
                    str.chars().for_each(|x| {
                        old_chars.push(x);
                        new_chars.push(x);
                        if x == '\n' || x == '\r' {
                            if 0 < old_chars.len() {
                                old_line.push(ReplaceDiffChars {
                                    diff_kind,
                                    chars: old_chars.clone(),
                                });
                                old_chars = String::new();
                                new_line.push(ReplaceDiffChars {
                                    diff_kind,
                                    chars: new_chars.clone(),
                                });
                                new_chars = String::new();
                            }
                            old_lines.push(old_line.clone());
                            old_line = vec![];
                            new_lines.push(new_line.clone());
                            new_line = vec![];
                        }
                    });
                    if 0 < old_chars.len() {
                        old_line.push(ReplaceDiffChars {
                            diff_kind,
                            chars: old_chars.clone(),
                        });
                        old_chars = String::new();
                        new_line.push(ReplaceDiffChars {
                            diff_kind,
                            chars: new_chars.clone(),
                        });
                        new_chars = String::new();
                    }
                }
                DiffTag::Delete => {
                    let old_range = x.old_range();
                    // let str = &old_str[old_range.start..old_range.end];
                    let (start, end) =
                        multibyte_str_byte_indices(old_str, old_range.start, old_range.end)
                            .unwrap();
                    let str = &old_str[start..end];
                    str.chars().for_each(|x| {
                        old_chars.push(x);
                        if x == '\n' || x == '\r' {
                            if 0 < old_chars.len() {
                                old_line.push(ReplaceDiffChars {
                                    diff_kind,
                                    chars: old_chars.clone(),
                                });
                                old_chars = String::new();
                            }
                            old_lines.push(old_line.clone());
                            old_line = vec![];
                        }
                    });
                    if 0 < old_chars.len() {
                        old_line.push(ReplaceDiffChars {
                            diff_kind,
                            chars: old_chars.clone(),
                        });
                        old_chars = String::new();
                    }
                }
                DiffTag::Insert => {
                    let new_range = x.new_range();
                    // let str = &new_str[new_range.start..new_range.end];
                    let (start, end) =
                        multibyte_str_byte_indices(new_str, new_range.start, new_range.end)
                            .unwrap();
                    let str = &new_str[start..end];
                    str.chars().for_each(|x| {
                        new_chars.push(x);
                        if x == '\n' || x == '\r' {
                            if 0 < new_chars.len() {
                                new_line.push(ReplaceDiffChars {
                                    diff_kind,
                                    chars: new_chars.clone(),
                                });
                                new_chars = String::new();
                            }
                            new_lines.push(new_line.clone());
                            new_line = vec![];
                        }
                    });
                    if 0 < new_chars.len() {
                        new_line.push(ReplaceDiffChars {
                            diff_kind,
                            chars: new_chars.clone(),
                        });
                        new_chars = String::new();
                    }
                }
                DiffTag::Replace => {
                    let old_range = x.old_range();
                    // let old_str = (&old_str[old_range.start..old_range.end]).to_owned();
                    let (old_start, old_end) =
                        multibyte_str_byte_indices(old_str, old_range.start, old_range.end)
                            .unwrap();
                    let old_str = &old_str[old_start..old_end];
                    old_str.chars().for_each(|x| {
                        old_chars.push(x);
                        if x == '\n' || x == '\r' {
                            if 0 < old_chars.len() {
                                old_line.push(ReplaceDiffChars {
                                    diff_kind,
                                    chars: old_chars.clone(),
                                });
                                old_chars = String::new();
                            }
                            old_lines.push(old_line.clone());
                            old_line = vec![];
                        }
                    });
                    if 0 < old_chars.len() {
                        old_line.push(ReplaceDiffChars {
                            diff_kind,
                            chars: old_chars.clone(),
                        });
                        old_chars = String::new();
                    }

                    let new_range = x.new_range();
                    // let new_str = (&new_str[new_range.start..new_range.end]).to_owned();
                    let (new_start, new_end) =
                        multibyte_str_byte_indices(new_str, new_range.start, new_range.end)
                            .unwrap();
                    let new_str = &new_str[new_start..new_end];
                    new_str.chars().for_each(|x| {
                        new_chars.push(x);
                        if x == '\n' || x == '\r' {
                            if 0 < new_chars.len() {
                                new_line.push(ReplaceDiffChars {
                                    diff_kind,
                                    chars: new_chars.clone(),
                                });
                                new_chars = String::new();
                            }
                            new_lines.push(new_line.clone());
                            new_line = vec![];
                        }
                    });
                    if 0 < new_chars.len() {
                        new_line.push(ReplaceDiffChars {
                            diff_kind,
                            chars: new_chars.clone(),
                        });
                        new_chars = String::new();
                    }
                }
            }
        });
    if 0 < old_line.len() {
        old_lines.push(old_line);
    }
    if 0 < new_line.len() {
        new_lines.push(new_line);
    }
    ReplaceDetailLinesDiff {
        old_lines,
        new_lines,
    }
}

fn multibyte_str_byte_indices(
    text: &str,
    index_start: usize,
    index_end: usize,
) -> Option<(usize, usize)> {
    let char_indices = text.char_indices().collect::<Vec<_>>();

    // Ensure the start and end indices are within bounds
    if index_start >= char_indices.len()
        || index_end > char_indices.len()
        || index_start >= index_end
    {
        return None;
    }

    // Get the byte indices corresponding to the char indices
    let byte_start = char_indices[index_start].0;
    let byte_end = char_indices[index_end - 1].0 + char_indices[index_end - 1].1.len_utf8();

    Some((byte_start, byte_end))
}
