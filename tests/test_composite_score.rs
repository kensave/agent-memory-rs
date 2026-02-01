use memory_rs::services::composite_score::CompositeScoreCalculator;

#[test]
fn test_calculate_recency() {
    let calc = CompositeScoreCalculator::new();
    
    let recent = "2026-01-31 00:00:00";
    let score = calc.calculate_recency(recent).unwrap();
    assert!(score > 0.9, "Recent memory should have high recency score");
    
    let old = "2025-01-01 00:00:00";
    let score = calc.calculate_recency(old).unwrap();
    assert!(score < 0.5, "Old memory should have low recency score");
}

#[test]
fn test_calculate_relevance() {
    let calc = CompositeScoreCalculator::new();
    
    assert_eq!(calc.calculate_relevance(0.85), 0.85);
    assert_eq!(calc.calculate_relevance(1.5), 1.0);
    assert_eq!(calc.calculate_relevance(-0.1), 0.0);
}

#[test]
fn test_calculate_utility() {
    let calc = CompositeScoreCalculator::new();
    
    let score = calc.calculate_utility(50, 0.9, 0.8);
    assert!(score > 0.7 && score <= 1.0);
    
    let score = calc.calculate_utility(0, 0.0, 0.0);
    assert_eq!(score, 0.0);
}

#[test]
fn test_calculate_composite() {
    let calc = CompositeScoreCalculator::new();
    
    let score = calc.calculate_composite(0.9, 0.8, 0.7);
    let expected = 0.9 * 0.3 + 0.8 * 0.4 + 0.7 * 0.3;
    assert!((score.combined - expected).abs() < 0.01);
    assert_eq!(score.recency, 0.9);
    assert_eq!(score.relevance, 0.8);
    assert_eq!(score.utility, 0.7);
}

#[test]
fn test_calculate_for_memory() {
    let calc = CompositeScoreCalculator::new();
    
    let score = calc.calculate_for_memory(
        "2026-01-30 00:00:00",
        0.85,
        25,
        0.9,
        0.8
    ).unwrap();
    
    assert!(score.recency > 0.8, "Recent memory should have high recency");
    assert_eq!(score.relevance, 0.85);
    assert!(score.utility > 0.6);
    assert!(score.combined > 0.7);
}

#[test]
fn test_custom_weights() {
    let calc = CompositeScoreCalculator::with_weights(0.5, 0.3, 0.2);
    
    let score = calc.calculate_composite(1.0, 1.0, 1.0);
    assert_eq!(score.combined, 1.0);
    
    let score = calc.calculate_composite(1.0, 0.0, 0.0);
    assert_eq!(score.combined, 0.5);
}
