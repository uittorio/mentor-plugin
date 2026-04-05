use serde::Serialize;

use crate::sm2::sm2;
use crate::trigram_similarity::trigram_similarity;

mod sm2;
pub mod sqlite_topic_storage;
pub mod topic_storage;
pub mod trigram_similarity;

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

    pub fn new(name: &str, reviewed_at: u64) -> Topic {
        Topic {
            name: name.to_string(),
            repetitions: 0,
            interval: 1,
            ease_factor: 2.5,
            reviewed_at: reviewed_at,
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
