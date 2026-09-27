pub fn calculate_mmr_range(wait_time_secs: u64) -> i32 {
    let base_range = 75;
    let expansion_rate = 15;
    let max_range = 300;

    (base_range + (wait_time_secs * expansion_rate) as i32).min(max_range)
}

pub fn is_within_range(player_mmr: i32, target_mmr: i32, wait_time_secs: u64) -> bool {
    let range = calculate_mmr_range(wait_time_secs);
    (player_mmr - target_mmr).abs() <= range
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_mmr_range() {
        // range = 75 + (wait * 15)
        assert_eq!(calculate_mmr_range(0), 75);
        assert_eq!(calculate_mmr_range(5), 150);
        assert_eq!(calculate_mmr_range(10), 225);
        assert_eq!(calculate_mmr_range(20), 300); // capped
    }

    #[test]
    fn test_is_within_range() {
        // Wait time 2s -> range 105
        assert!(is_within_range(1000, 1105, 2));
        assert!(!is_within_range(1000, 1106, 2));

        // Wait time 6s -> range 165
        assert!(is_within_range(1000, 1165, 6));
        assert!(!is_within_range(1000, 1166, 6));
    }
}
