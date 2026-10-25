//! Three-way merge of two sequences against their common ancestor.
//!
//! Changes that touch different parts of the ancestor merge, even when they
//! are next to each other; changes that overlap, or insert at the same
//! place, are returned as a region for the caller to resolve.

#[derive(Debug, Clone)]
pub struct Hunk {
    pub start: usize,
    pub end: usize,
    pub replacement: Vec<String>,
}

/// The edits that turn `base` into `other`: the parts of `base` replaced.
pub fn hunks(base: &[String], other: &[String]) -> Vec<Hunk> {
    let mut prefix = 0;
    while prefix < base.len() && prefix < other.len() && base[prefix] == other[prefix] {
        prefix += 1;
    }
    let mut suffix = 0;
    while suffix < base.len() - prefix && suffix < other.len() - prefix && base[base.len() - 1 - suffix] == other[other.len() - 1 - suffix] {
        suffix += 1;
    }
    let (a, b) = (&base[prefix..base.len() - suffix], &other[prefix..other.len() - suffix]);
    if a.is_empty() && b.is_empty() {
        return Vec::new();
    }
    if a.is_empty() || b.is_empty() || a.len() * b.len() > 16_000_000 {
        return vec![Hunk { start: prefix, end: prefix + a.len(), replacement: b.to_vec() }];
    }
    // Longest common subsequence of the differing middle.
    let (n, m) = (a.len(), b.len());
    let mut table = vec![0u32; (n + 1) * (m + 1)];
    let at = |i: usize, j: usize| i * (m + 1) + j;
    for i in (0..n).rev() {
        for j in (0..m).rev() {
            table[at(i, j)] = if a[i] == b[j] { table[at(i + 1, j + 1)] + 1 } else { table[at(i + 1, j)].max(table[at(i, j + 1)]) };
        }
    }
    let mut out = Vec::new();
    let (mut i, mut j) = (0, 0);
    let mut open: Option<(usize, usize)> = None;
    while i < n || j < m {
        if i < n && j < m && a[i] == b[j] {
            if let Some((si, sj)) = open.take() {
                out.push(Hunk { start: prefix + si, end: prefix + i, replacement: b[sj..j].to_vec() });
            }
            i += 1;
            j += 1;
        } else {
            if open.is_none() {
                open = Some((i, j));
            }
            if j < m && (i == n || table[at(i, j + 1)] >= table[at(i + 1, j)]) {
                j += 1;
            } else {
                i += 1;
            }
        }
    }
    if let Some((si, sj)) = open {
        out.push(Hunk { start: prefix + si, end: prefix + n, replacement: b[sj..].to_vec() });
    }
    out
}

#[derive(Debug, Clone)]
pub enum Chunk {
    /// Settled: the same on both sides, or changed on one only.
    Text(Vec<String>),
    /// Both sides changed the same part of the ancestor, differently.
    Region { base: Vec<String>, ours: Vec<String>, theirs: Vec<String> },
}

fn conflicts(a: (usize, usize), b: (usize, usize)) -> bool {
    match (a.0 == a.1, b.0 == b.1) {
        (true, true) => a.0 == b.0,
        (true, false) => b.0 < a.0 && a.0 < b.1,
        (false, true) => a.0 < b.0 && b.0 < a.1,
        (false, false) => a.0 < b.1 && b.0 < a.1,
    }
}

fn apply(base: &[String], start: usize, end: usize, hunks: &[&Hunk]) -> Vec<String> {
    let mut out = Vec::new();
    let mut at = start;
    for h in hunks {
        out.extend_from_slice(&base[at..h.start]);
        out.extend(h.replacement.iter().cloned());
        at = h.end;
    }
    out.extend_from_slice(&base[at..end]);
    out
}

pub fn merge3(base: &[String], ours: &[String], theirs: &[String]) -> Vec<Chunk> {
    let mut all: Vec<(bool, Hunk)> = hunks(base, ours).into_iter().map(|h| (true, h)).collect();
    all.extend(hunks(base, theirs).into_iter().map(|h| (false, h)));
    all.sort_by(|x, y| (x.1.start, x.1.end, !x.0).cmp(&(y.1.start, y.1.end, !y.0)));
    let mut chunks: Vec<Chunk> = Vec::new();
    let push_text = |chunks: &mut Vec<Chunk>, lines: Vec<String>| {
        if lines.is_empty() {
            return;
        }
        match chunks.last_mut() {
            Some(Chunk::Text(previous)) => previous.extend(lines),
            _ => chunks.push(Chunk::Text(lines)),
        }
    };
    let (mut position, mut index) = (0, 0);
    while index < all.len() {
        let (first_ours, first) = &all[index];
        let (mut start, mut end) = (first.start, first.end);
        let mut cluster = vec![(*first_ours, first)];
        index += 1;
        while index < all.len() && conflicts((start, end), (all[index].1.start, all[index].1.end)) {
            let (side, h) = &all[index];
            start = start.min(h.start);
            end = end.max(h.end);
            cluster.push((*side, h));
            index += 1;
        }
        push_text(&mut chunks, base[position..start].to_vec());
        let mine: Vec<&Hunk> = cluster.iter().filter(|c| c.0).map(|c| c.1).collect();
        let yours: Vec<&Hunk> = cluster.iter().filter(|c| !c.0).map(|c| c.1).collect();
        let (o, t) = (apply(base, start, end, &mine), apply(base, start, end, &yours));
        if yours.is_empty() {
            push_text(&mut chunks, o);
        } else if mine.is_empty() || o == t {
            push_text(&mut chunks, t);
        } else {
            chunks.push(Chunk::Region { base: base[start..end].to_vec(), ours: o, theirs: t });
        }
        position = end;
    }
    push_text(&mut chunks, base[position..].to_vec());
    chunks
}

#[cfg(test)]
mod tests {
    use super::*;

    fn v(items: &[&str]) -> Vec<String> {
        items.iter().map(|s| s.to_string()).collect()
    }

    fn flat(chunks: &[Chunk]) -> Option<Vec<String>> {
        let mut out = Vec::new();
        for c in chunks {
            match c {
                Chunk::Text(t) => out.extend(t.iter().cloned()),
                Chunk::Region { .. } => return None,
            }
        }
        Some(out)
    }

    #[test]
    fn adjacent_changes_on_each_side_merge() {
        let merged = merge3(&v(&["a", "b", "c", "d"]), &v(&["a", "B", "c", "d"]), &v(&["a", "b", "C", "d"]));
        assert_eq!(flat(&merged).unwrap(), v(&["a", "B", "C", "d"]));
    }

    #[test]
    fn the_same_line_changed_differently_is_a_region() {
        let merged = merge3(&v(&["a", "b"]), &v(&["a", "X"]), &v(&["a", "Y"]));
        assert!(flat(&merged).is_none());
    }

    #[test]
    fn identical_changes_agree() {
        let merged = merge3(&v(&["a", "b"]), &v(&["a", "X"]), &v(&["a", "X"]));
        assert_eq!(flat(&merged).unwrap(), v(&["a", "X"]));
    }

    #[test]
    fn insertions_at_the_same_place_are_a_region() {
        let merged = merge3(&v(&["a", "b"]), &v(&["a", "X", "b"]), &v(&["a", "Y", "b"]));
        assert!(flat(&merged).is_none());
    }
}
