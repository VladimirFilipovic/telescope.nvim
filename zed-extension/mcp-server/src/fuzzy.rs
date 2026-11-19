//! fzf-style fuzzy scoring: query chars must appear in order; contiguous runs,
//! word starts and basename matches score higher.

/// Score of `candidate` for `query`, or `None` when it does not match.
/// Smart case: matching is case-insensitive unless the query has an uppercase letter.
pub fn score(query: &str, candidate: &str) -> Option<i64> {
    let query: Vec<char> = query.chars().filter(|c| !c.is_whitespace()).collect();
    if query.is_empty() {
        return Some(0);
    }
    let case_sensitive = query.iter().any(|c| c.is_uppercase());
    let norm = |c: char| if case_sensitive { c } else { c.to_ascii_lowercase() };
    let cand: Vec<char> = candidate.chars().collect();
    let basename_start = candidate.rfind('/').map_or(0, |i| candidate[..=i].chars().count());

    let mut total = 0i64;
    let mut qi = 0;
    let mut prev_match: Option<usize> = None;
    for (ci, &c) in cand.iter().enumerate() {
        if qi == query.len() {
            break;
        }
        if norm(c) != norm(query[qi]) {
            continue;
        }
        let mut s = 16;
        let prev = ci.checked_sub(1).map(|i| cand[i]);
        let word_start = match prev {
            None => true,
            Some(p) => matches!(p, '/' | '_' | '-' | '.' | ' ') || (p.is_lowercase() && c.is_uppercase()),
        };
        if word_start {
            s += 24;
        }
        if prev_match.is_some_and(|p| p + 1 == ci) {
            s += 32;
        } else if let Some(p) = prev_match {
            s -= ((ci - p - 1) as i64).min(12);
        }
        if ci >= basename_start {
            s += 8;
        }
        total += s;
        prev_match = Some(ci);
        qi += 1;
    }
    (qi == query.len()).then(|| total - cand.len() as i64 / 4)
}

/// Candidates that match `query`, best first, at most `limit`.
pub fn rank<'a>(query: &str, candidates: impl IntoIterator<Item = &'a str>, limit: usize) -> Vec<&'a str> {
    let mut scored: Vec<(i64, &str)> = candidates
        .into_iter()
        .filter_map(|c| score(query, c).map(|s| (s, c)))
        .collect();
    scored.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.len().cmp(&b.1.len())));
    scored.into_iter().take(limit).map(|(_, c)| c).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn requires_ordered_subsequence() {
        assert!(score("abc", "a_b_c").is_some());
        assert!(score("cba", "abc").is_none());
    }

    #[test]
    fn smart_case() {
        assert!(score("readme", "README.md").is_some());
        assert!(score("README", "readme.md").is_none());
    }

    #[test]
    fn prefers_basename_and_contiguous_matches() {
        let files = [
            "lua/telescope/builtin/__files.lua",
            "lua/telescope/finders.lua",
            "doc/telescope.txt",
            "lua/telescope/_extensions/init.lua",
        ];
        assert_eq!(rank("files", files, 4)[0], "lua/telescope/builtin/__files.lua");
        assert_eq!(rank("finders", files, 4)[0], "lua/telescope/finders.lua");
    }

    #[test]
    fn empty_query_matches_everything() {
        assert_eq!(rank("", ["b", "a"], 10).len(), 2);
    }
}
