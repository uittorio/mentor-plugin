use std::collections::HashSet;

use serde::Serialize;

use crate::{category::Category, sm2::sm2, trigram_similarity::trigram_similarity};

#[derive(Clone)]
pub struct Topic {
    pub name: String,

    // consecutive successes review with high quality
    pub repetitions: u32,

    // number of days until next review
    pub interval: u32,

    // it increases with higher quality
    pub ease_factor: f32,
    pub reviewed_at: u64,

    pub categories: TopicCategories,
}

#[derive(Clone)]
pub struct TopicCategories(pub Vec<Category>);

impl TopicCategories {
    pub fn add(&self, categories: Vec<String>) -> TopicCategories {
        let merged: HashSet<String> = self
            .0
            .iter()
            .map(|c| c.name.clone())
            .chain(categories.into_iter())
            .map(|c| c.to_lowercase())
            .collect();

        TopicCategories(merged.into_iter().map(|name| Category { name }).collect())
    }
}

impl Topic {
    pub fn question_depth(&self) -> QuestionDepth {
        match (self.ease_factor, self.repetitions) {
            (_, r) if r < 3 => QuestionDepth::Full,
            (ef, _) if ef < 2.0 => QuestionDepth::Full,
            (ef, _) if ef <= 2.5 => QuestionDepth::Light,
            (_, _) => QuestionDepth::Skip,
        }
    }

    pub fn is_similar(&self, value: &str) -> bool {
        trigram_similarity(value, &self.name) >= 0.2
    }

    pub fn update_quality(&self, quality: u32, review_date: u64) -> Topic {
        sm2(&self, quality, review_date)
    }

    pub fn add_categories(&self, categories: Vec<String>) -> Topic {
        Topic {
            name: self.name.clone(),
            repetitions: self.repetitions,
            interval: self.interval,
            ease_factor: self.ease_factor,
            reviewed_at: self.reviewed_at,
            categories: self.categories.add(categories),
        }
    }

    pub fn new(name: &str, reviewed_at: u64) -> Topic {
        Topic {
            name: name.to_string(),
            repetitions: 0,
            interval: 1,
            ease_factor: 2.5,
            reviewed_at: reviewed_at,
            categories: TopicCategories(vec![]),
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "lowercase")]
pub enum QuestionDepth {
    Full,
    Light,
    Skip,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn topic_categories_merge() {
        let topic = Topic {
            reviewed_at: 10000,
            ..mocked_topic()
        };

        let new_topic = topic.add_categories(vec!["One".to_string(), "Two".to_string()]);

        let mut names = new_topic
            .categories
            .0
            .iter()
            .map(|c| &c.name)
            .collect::<Vec<_>>();

        let mut expected = vec!["one", "two"];
        names.sort();
        expected.sort();

        assert_eq!(names, expected);

        let without_duplicates = new_topic.add_categories(vec!["oNE".to_string()]);

        let mut without_duplicates_names = without_duplicates
            .categories
            .0
            .iter()
            .map(|c| &c.name)
            .collect::<Vec<_>>();

        without_duplicates_names.sort();

        assert_eq!(without_duplicates_names, expected);
    }
}

#[cfg(test)]
pub fn mocked_topic() -> Topic {
    Topic {
        name: "test topic".to_string(),
        repetitions: 1,
        interval: 10,
        ease_factor: 1.5,
        reviewed_at: 10000,
        categories: TopicCategories(vec![]),
    }
}
