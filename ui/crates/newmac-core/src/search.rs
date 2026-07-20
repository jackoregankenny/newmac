//! Fuzzy search across the catalog (ROADMAP #3).
//!
//! Uses `nucleo-matcher` — the same matcher behind Helix/Nucleo — so a query
//! like `dokr` still finds `docker` and `ff` finds `firefox`. We match against
//! a composite haystack (name + id + description) so typing what a tool *does*
//! ("password", "vpn", "diff") surfaces it too.

use nucleo_matcher::pattern::{CaseMatching, Normalization, Pattern};
use nucleo_matcher::{Config, Matcher, Utf32Str};

/// A reusable fuzzy matcher. Cheap to keep around; expensive to recreate.
pub struct Searcher {
    matcher: Matcher,
}

impl Default for Searcher {
    fn default() -> Self {
        Self::new()
    }
}

impl Searcher {
    pub fn new() -> Self {
        Self {
            matcher: Matcher::new(Config::DEFAULT),
        }
    }

    /// Score one haystack against a query. `None` = no match. Higher = better.
    pub fn score(&mut self, query: &str, haystack: &str) -> Option<u32> {
        if query.is_empty() {
            return Some(0);
        }
        let pattern = Pattern::parse(query, CaseMatching::Ignore, Normalization::Smart);
        let mut buf = Vec::new();
        let hay = Utf32Str::new(haystack, &mut buf);
        pattern.score(hay, &mut self.matcher)
    }

    /// Rank a set of `(key, haystack)` candidates against `query`, best first.
    /// An empty query keeps the original order (every candidate scores 0).
    pub fn rank<'a, K, I>(&mut self, query: &str, candidates: I) -> Vec<(K, u32)>
    where
        I: IntoIterator<Item = (K, &'a str)>,
        K: Copy,
    {
        let mut out: Vec<(K, u32)> = candidates
            .into_iter()
            .filter_map(|(key, hay)| self.score(query, hay).map(|s| (key, s)))
            .collect();
        if !query.is_empty() {
            // Stable sort by score desc; ties keep catalog order.
            out.sort_by_key(|&(_, score)| std::cmp::Reverse(score));
        }
        out
    }
}

/// Build the composite haystack a catalog item is matched against.
pub fn haystack(item: &crate::catalog::Item) -> String {
    format!("{} {} {}", item.name, item.id, item.description)
}
