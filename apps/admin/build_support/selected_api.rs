use sha2::{Digest, Sha256};
use std::path::Path;

const ANALYTICS_API_SHA256: &str =
    "3f1799bcf33b1d18a479486401c35bc43820b8e89eb1381b9cc8ec9e1464e85d";

pub fn validate(preset: &str, selected: &[u8]) {
    let source = format!("../web/src/distribution/{preset}-api.ts");
    println!("cargo:rerun-if-changed={source}");
    let authoritative = std::fs::read(&source)
        .unwrap_or_else(|_| panic!("selected Admin API authority is unavailable: {source}"));
    assert_eq!(selected, authoritative, "selected Admin API differs from its source authority");
    if preset == "analytics" {
        assert_eq!(
            format!("{:x}", Sha256::digest(selected)),
            ANALYTICS_API_SHA256,
            "Analytics API differs from its reviewed digest"
        );
        validate_analytics(std::str::from_utf8(selected).expect("Analytics API must be UTF-8"));
    }
    assert!(Path::new(&source).is_file(), "selected Admin API authority must be a file");
}

fn validate_analytics(source: &str) {
    for forbidden in
        ["/api/monitor", "/api/reports", "/api/notifications", "/api/manage", "/api/system/status"]
    {
        assert!(!source.contains(forbidden), "Analytics API names excluded owner: {forbidden}");
    }
    let block = source
        .split_once("const insightsAPI = {")
        .and_then(|(_, tail)| tail.split_once("\n};"))
        .map(|(block, _)| block)
        .expect("Analytics API must define one insightsAPI object");
    assert_eq!(
        source.match_indices("const insightsAPI = {").count(),
        1,
        "Analytics API must define one insightsAPI object"
    );
    const READS: &str = r#"
    overview: (params: Insights.OverviewQuery) =>
        apiRequest<Insights.Overview, Insights.OverviewQuery>({
            url: "/api/insights/overview",
            method: "GET",
            params,
            silent: true,
        }),
    events: (params: Insights.EventQuery) =>
        apiRequest<Insights.Page<Insights.Event>, Insights.EventQuery>({
            url: "/api/insights/events",
            method: "GET",
            params,
            silent: true,
        }),"#;
    assert_eq!(block, READS, "Analytics API read object differs from reviewed contract");
    assert_eq!(
        source.match_indices("/api/insights").count(),
        2,
        "Analytics API must name only its two read routes"
    );
}
