use super::support::*;

#[tokio::test]
async fn protocol_fencing_fixture_corpus_matches_the_contract_descriptor() {
    use crate::protocol::canonical_contract_descriptor;

    let descriptor: serde_json::Value =
        serde_json::from_str(&canonical_contract_descriptor()).expect("canonical descriptor");
    let fencing = &descriptor["acceptance"]["fencing"];
    for (rule, outcome) in [
        ("firstNodeSequenceOne", "accepted"),
        ("firstNodeOtherSequence", "stale"),
        ("sameBootEqualSequence", "duplicate"),
        ("sameBootLowerSequence", "stale"),
        ("sameBootHigherNonIncreasingCollectedAt", "stale"),
        ("sameBootHigherLaterCollectedAt", "accepted"),
        ("newBootOtherSequence", "stale"),
        ("newBootNonIncreasingCollectedAt", "stale"),
        ("newBootSequenceOneLaterCollectedAt", "accepted"),
        ("retiredBoot", "stale"),
    ] {
        assert_eq!(fencing[rule], outcome, "descriptor rule {rule}");
    }

    let pool = migrated_test_pool().await;
    let received = Utc.with_ymd_and_hms(2026, 9, 4, 12, 0, 0).unwrap();
    let first_boot = Uuid::new_v4();
    let first = report("contract-fencing", first_boot, 1, received, 10.0, 10, 10);
    assert_eq!(
        record_at(&pool, first.clone(), received).await.unwrap(),
        AgentReportStatus::Accepted
    );
    assert_eq!(
        record_at(
            &pool,
            report("first-other", Uuid::new_v4(), 2, received, 10.0, 10, 10),
            received
        )
        .await
        .unwrap(),
        AgentReportStatus::Stale
    );
    assert_eq!(
        record_at(&pool, first.clone(), received).await.unwrap(),
        AgentReportStatus::Duplicate
    );
    assert_eq!(
        record_at(
            &pool,
            report("contract-fencing", first_boot, 2, received, 10.0, 10, 10),
            received,
        )
        .await
        .unwrap(),
        AgentReportStatus::Stale
    );
    assert_eq!(
        record_at(
            &pool,
            report(
                "contract-fencing",
                first_boot,
                2,
                received + ChronoDuration::seconds(1),
                10.0,
                10,
                10
            ),
            received + ChronoDuration::seconds(1),
        )
        .await
        .unwrap(),
        AgentReportStatus::Accepted
    );
    assert_eq!(
        record_at(
            &pool,
            report(
                "contract-fencing",
                first_boot,
                1,
                received + ChronoDuration::seconds(2),
                10.0,
                10,
                10
            ),
            received + ChronoDuration::seconds(2),
        )
        .await
        .unwrap(),
        AgentReportStatus::Stale
    );

    let second_boot = Uuid::new_v4();
    assert_eq!(
        record_at(
            &pool,
            report(
                "contract-fencing",
                second_boot,
                2,
                received + ChronoDuration::seconds(3),
                10.0,
                10,
                10
            ),
            received + ChronoDuration::seconds(3),
        )
        .await
        .unwrap(),
        AgentReportStatus::Stale
    );
    assert_eq!(
        record_at(
            &pool,
            report(
                "contract-fencing",
                second_boot,
                1,
                received + ChronoDuration::seconds(1),
                10.0,
                10,
                10
            ),
            received + ChronoDuration::seconds(1),
        )
        .await
        .unwrap(),
        AgentReportStatus::Stale
    );
    assert_eq!(
        record_at(
            &pool,
            report(
                "contract-fencing",
                second_boot,
                1,
                received + ChronoDuration::seconds(3),
                10.0,
                10,
                10
            ),
            received + ChronoDuration::seconds(3),
        )
        .await
        .unwrap(),
        AgentReportStatus::Accepted
    );
    assert_eq!(
        record_at(
            &pool,
            report(
                "contract-fencing",
                first_boot,
                3,
                received + ChronoDuration::seconds(4),
                10.0,
                10,
                10
            ),
            received + ChronoDuration::seconds(4),
        )
        .await
        .unwrap(),
        AgentReportStatus::Stale
    );

    let stale_historical = report("contract-fencing", Uuid::new_v4(), 2, received, 10.0, 10, 10);
    assert_eq!(
        record_at(&pool, stale_historical, received + ChronoDuration::minutes(10)).await.unwrap(),
        AgentReportStatus::Stale
    );
}
