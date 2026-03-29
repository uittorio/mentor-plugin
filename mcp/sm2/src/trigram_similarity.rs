use std::collections::HashSet;

pub fn trigram_similarity(search: &str, candidate: &str) -> f32 {
    let value_trigrams = trigrams(candidate);
    let search_trigrams = trigrams(search);

    let intersections = value_trigrams.intersection(&search_trigrams);
    let union = value_trigrams.union(&search_trigrams);
    return intersections.count() as f32 / union.count() as f32;
}

fn trigrams(value: &str) -> HashSet<String> {
    let chars: Vec<char> = value.to_lowercase().chars().collect();
    chars
        .windows(3)
        .map(|x| x.iter().collect::<String>())
        .collect()
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn no_matches() {
        assert_eq!(trigram_similarity("rust", "javascript"), 0.0)
    }

    #[test]
    fn one_match() {
        assert_eq!(trigram_similarity("java", "javascript"), 0.25)
    }

    #[test]
    fn full_match() {
        assert_eq!(trigram_similarity("javascript", "javascript"), 1.0)
    }

    #[test]
    fn one_word_match() {
        assert_eq!(
            trigram_similarity("javascript closure", "rust closure"),
            0.36842105
        )
    }
}
