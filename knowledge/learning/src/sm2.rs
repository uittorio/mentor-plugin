use crate::topic::Topic;

// - Quality 5 → Ease factor increases
// - Quality 4 → Ease factor stays the same
// - Quality 3 → Ease factor decreases slightly
// - Quality < 3 → Ease factor drops significantly
pub fn sm2(topic: &Topic, quality: u32, review_date: u64) -> Topic {
    let mut updated_repetitions = topic.repetitions;
    let mut updated_interval = topic.interval;
    match quality {
        q if q < 3 => {
            updated_repetitions = 0;
        }
        _ => {
            updated_repetitions = updated_repetitions + 1;
        }
    }

    let updated_ease_factor = (topic.ease_factor + ease_factor(quality)).max(1.3);

    match updated_repetitions {
        0 => {
            updated_interval = 1;
        }
        1 => updated_interval = 6,
        _ => updated_interval = ((updated_interval as f32) * updated_ease_factor) as u32,
    }

    Topic {
        name: topic.name.clone(),
        repetitions: updated_repetitions,
        interval: updated_interval,
        ease_factor: updated_ease_factor,
        reviewed_at: review_date,
        categories: topic.categories.clone(),
    }
}

fn ease_factor(quality: u32) -> f32 {
    0.1 - (5 as f32 - quality as f32) * (0.08 + (5 as f32 - quality as f32) * 0.02)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn update_last_review_date() {
        let topic = Topic {
            reviewed_at: 10000,
            ..mocked_topic()
        };

        assert_eq!(sm2(&topic, 0, 2000).reviewed_at, 2000)
    }

    #[test]
    fn quality_less_than_3_reset_repetitions_and_interval() {
        let topic = Topic {
            repetitions: 3,
            interval: 10,
            ..mocked_topic()
        };

        let result_quality_0 = sm2(&topic, 0, 2000);
        assert_eq!(result_quality_0.repetitions, 0);
        assert_eq!(result_quality_0.interval, 1);

        let result_quality_1 = sm2(&topic, 1, 2000);
        assert_eq!(result_quality_1.repetitions, 0);
        assert_eq!(result_quality_1.interval, 1);

        let result_quality_2 = sm2(&topic, 2, 2000);
        assert_eq!(result_quality_2.repetitions, 0);
        assert_eq!(result_quality_2.interval, 1);
    }

    #[test]
    fn quality_equal_more_than_3_increase_repetitions_by_one() {
        let topic = Topic {
            repetitions: 0,
            interval: 10,
            ..mocked_topic()
        };

        let result_quality_3 = sm2(&topic, 3, 2000);
        assert_eq!(result_quality_3.repetitions, 1);

        let result_quality_4 = sm2(&topic, 4, 2000);
        assert_eq!(result_quality_4.repetitions, 1);
    }

    #[test]
    fn first_time_success_topic_interval_go_to_6() {
        let topic = Topic {
            repetitions: 0,
            interval: 0,
            ..mocked_topic()
        };

        let result = sm2(&topic, 4, 2000);
        assert_eq!(result.interval, 6);
    }

    #[test]
    fn consecutive_good_quality_increase_interval_more_than_6() {
        let topic = Topic {
            repetitions: 0,
            interval: 0,
            ..mocked_topic()
        };

        let mut result = sm2(&topic, 4, 2000);
        result = sm2(&result, 4, 2000);
        result = sm2(&result, 4, 2000);
        assert!(result.interval == 13, "expected 13 got {}", result.interval);
    }

    #[test]
    fn ease_factor_minimum_value() {
        let topic = Topic {
            repetitions: 0,
            ease_factor: 1.4,
            interval: 0,
            ..mocked_topic()
        };

        let result = sm2(&topic, 1, 2000);
        assert_eq!(result.ease_factor, 1.3);
    }

    #[test]
    fn ease_factor_increase_with_max_quality() {
        let topic = Topic {
            repetitions: 0,
            ease_factor: 1.4,
            interval: 0,
            ..mocked_topic()
        };

        let result = sm2(&topic, 5, 2000);
        assert_eq!(result.ease_factor, 1.5);
    }

    #[test]
    fn ease_factor_stay_the_same_with_quality_4() {
        let topic = Topic {
            repetitions: 0,
            ease_factor: 1.4,
            interval: 0,
            ..mocked_topic()
        };

        let result = sm2(&topic, 4, 2000);
        assert_eq!(result.ease_factor, 1.4);
    }

    #[test]
    fn ease_factor_decrease_with_quality_3_or_lower() {
        let topic = Topic {
            repetitions: 0,
            ease_factor: 3.5,
            interval: 0,
            ..mocked_topic()
        };

        let mut result = sm2(&topic, 3, 2000);
        let mut ef = result.ease_factor;
        assert!(
            ef > 3.35 && ef < 3.37,
            "expected ease factor between 3.35 and 3.37 but got {}",
            ef
        );

        result = sm2(&topic, 2, 2000);
        ef = result.ease_factor;
        assert!(
            ef > 3.17 && ef < 3.19,
            "expected ease factor between 3.17 and 3.19 but got {}",
            ef
        );

        result = sm2(&topic, 1, 2000);
        ef = result.ease_factor;
        assert!(
            ef > 2.95 && ef < 2.97,
            "expected ease factor between 2.95 and 2.97 but got {}",
            ef
        );
    }

    fn mocked_topic() -> Topic {
        Topic {
            name: "test topic".to_string(),
            repetitions: 1,
            interval: 10,
            ease_factor: 1.5,
            reviewed_at: 10000,
            categories: vec![],
        }
    }
}
