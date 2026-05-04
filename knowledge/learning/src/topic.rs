use std::collections::HashSet;

use chrono::{DateTime, Days, Utc};
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

    pub reviewed_at: DateTime<Utc>,

    pub categories: TopicCategories,
}

#[derive(Clone)]
pub struct TopicCategories(pub Vec<Category>);

impl TopicCategories {
    pub fn new(categories: Vec<String>) -> TopicCategories {
        let merged: HashSet<String> = categories.iter().map(|c| c.to_lowercase()).collect();

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

    pub fn mastered(&self) -> bool {
        self.question_depth() == QuestionDepth::Skip
    }

    pub fn struggled(&self) -> bool {
        self.question_depth() == QuestionDepth::Full && self.ease_factor < 2.0
    }

    pub fn learning(&self) -> bool {
        !self.mastered() && !self.struggled()
    }

    pub fn is_similar(&self, value: &str) -> bool {
        trigram_similarity(value, &self.name) >= 0.2
    }

    pub fn is_overdue(&self, now: DateTime<Utc>) -> bool {
        now >= self.next_review()
    }

    pub fn interval_in_seconds(&self) -> u32 {
        let seconds_in_a_day = 86400;
        self.interval * seconds_in_a_day
    }

    pub fn next_review(&self) -> DateTime<Utc> {
        let interval_days = Days::new(self.interval as u64);
        self.reviewed_at.checked_add_days(interval_days).unwrap()
    }

    pub fn is_between(&self, start: DateTime<Utc>, end: DateTime<Utc>) -> bool {
        self.reviewed_at >= start && self.reviewed_at <= end
    }

    pub fn has_category(&self, category: &Category) -> bool {
        self.categories.0.iter().any(|t| t.name == category.name)
    }

    pub fn days_since_last_review(&self, now: DateTime<Utc>) -> i64 {
        (now - &self.reviewed_at).num_days()
    }

    pub fn update_quality(&self, quality: u32, review_date: DateTime<Utc>) -> Topic {
        sm2(&self, quality, review_date)
    }

    pub fn update_categories(self, categories: Vec<String>) -> Topic {
        Topic {
            name: self.name.clone(),
            repetitions: self.repetitions,
            interval: self.interval,
            ease_factor: self.ease_factor,
            reviewed_at: self.reviewed_at,
            categories: TopicCategories::new(categories),
        }
    }

    pub fn new(name: &str, reviewed_at: DateTime<Utc>) -> Topic {
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

#[derive(PartialEq, Serialize)]
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
            reviewed_at: DateTime::from_timestamp_secs(1000).unwrap(),
            ..mocked_topic()
        };

        let new_topic = topic.update_categories(vec![
            "One".to_string(),
            "Two".to_string(),
            "two".to_string(),
        ]);

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
    }
}

#[cfg(test)]
pub fn mocked_topic() -> Topic {
    Topic {
        name: "test topic".to_string(),
        repetitions: 1,
        interval: 10,
        ease_factor: 1.5,
        reviewed_at: DateTime::from_timestamp_secs(10000).unwrap(),
        categories: TopicCategories(vec![]),
    }
}
