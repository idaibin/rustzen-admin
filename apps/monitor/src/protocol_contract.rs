use sha2::{Digest, Sha256};

use crate::protocol::{
    AGENT_PROTOCOL_VERSION, AGENT_REPORT_AUTH_HEADER, AGENT_REPORT_METHOD, AGENT_REPORT_PATH,
    MAX_AGENT_REPORT_BODY_BYTES, MAX_AGENT_VERSION_LEN, MAX_HOSTNAME_LEN, MAX_MOUNT_POINT_LEN,
    MAX_NODE_ID_LEN, MAX_REPORT_CLOCK_SKEW_SECONDS, RESPONSE_SUCCESS_CODE,
    RESPONSE_SUCCESS_MESSAGE,
};

#[cfg(test)]
use crate::protocol::{
    AgentReport, ByteUsage, DiskUsage, agent_reports_endpoint, parse_agent_response,
};
#[cfg(test)]
use chrono::Utc;
#[cfg(test)]
use uuid::Uuid;

/// Updated deliberately with a compatible protocol-version change.
pub const CONTRACT_PROTOCOL_SHA256: &str =
    "80257e2cddb90cca5c857d730d00b05aef1c24e18d27b7cf4fdb9f3225708928";

pub fn canonical_contract_descriptor() -> String {
    serde_json::json!({
        "acceptance": {
            "acceptedAdvancesSequence": true,
            "duplicateAdvancesSequence": true,
            "staleRetainsSequence": true,
            "errorRetainsSequence": true,
            "acceptedOrDuplicateAtMax": "exhausted",
            "fencingBeforeClockSkew": true,
            "fencing": {
                "firstNodeSequenceOne": "accepted",
                "firstNodeOtherSequence": "stale",
                "sameBootEqualSequence": "duplicate",
                "sameBootLowerSequence": "stale",
                "sameBootHigherNonIncreasingCollectedAt": "stale",
                "sameBootHigherLaterCollectedAt": "accepted",
                "newBootSequenceOneLaterCollectedAt": "accepted",
                "newBootOtherSequence": "stale",
                "retiredBoot": "stale",
                "newBootNonIncreasingCollectedAt": "stale"
            }
        },
        "auth": {"header": AGENT_REPORT_AUTH_HEADER, "required": true},
        "limits": {
            "bodyBytesMax": MAX_AGENT_REPORT_BODY_BYTES,
            "clockSkewSeconds": MAX_REPORT_CLOCK_SKEW_SECONDS,
            "nodeId": {"min": 1, "max": MAX_NODE_ID_LEN, "charset": "ascii-alnum-._-"},
            "hostname": {"minNonblank": 1, "max": MAX_HOSTNAME_LEN},
            "agentVersion": {"minNonblank": 1, "max": MAX_AGENT_VERSION_LEN},
            "cpuPercent": {"finite": true, "min": 0, "max": 100},
            "byteUsage": {
                "usedMin": 0,
                "usedMax": i64::MAX,
                "totalMin": 1,
                "totalMax": i64::MAX,
                "usedLessThanOrEqualTotal": true
            },
            "mountPoint": {"minNonblank": 1, "max": MAX_MOUNT_POINT_LEN, "unique": true},
            "disksAllowEmpty": true,
            "timestamp": "rfc3339",
            "sequenceMin": 1,
            "sequenceMax": i64::MAX
        },
        "method": AGENT_REPORT_METHOD,
        "path": AGENT_REPORT_PATH,
        "request": {
            "encoding": "application/json",
            "required": [
                "nodeId:string", "bootId:uuid", "sequence:u64", "hostname:string",
                "agentVersion:string", "collectedAt:rfc3339", "cpuPercent:f64",
                "memory:{usedBytes:u64,totalBytes:u64}",
                "disks:[{mountPoint:string,usedBytes:u64,totalBytes:u64}]"
            ]
        },
        "response": {
            "encoding": "application/json",
            "httpSuccessRequired": true,
            "code": RESPONSE_SUCCESS_CODE,
            "message": RESPONSE_SUCCESS_MESSAGE,
            "required": ["code", "message", "data.status"],
            "statuses": ["accepted", "duplicate", "stale"]
        },
        "version": AGENT_PROTOCOL_VERSION
    })
    .to_string()
}

pub fn contract_protocol_digest_for(descriptor: &str) -> String {
    format!("{:x}", Sha256::digest(descriptor.as_bytes()))
}

pub fn contract_protocol_digest() -> String {
    contract_protocol_digest_for(&canonical_contract_descriptor())
}

pub fn validate_protocol_pair(controller: &str, agent: &str) -> Result<(), &'static str> {
    if controller == agent { Ok(()) } else { Err("Monitor protocol digest mismatch") }
}

pub fn contract_protocol_output() -> String {
    format!("{}\n{}", canonical_contract_descriptor(), contract_protocol_digest())
}

#[cfg(test)]
mod contract_tests {
    use super::*;

    #[test]
    fn descriptor_binds_runtime_constants_wire_shape_and_response_parser() {
        let descriptor = canonical_contract_descriptor();
        let digest = contract_protocol_digest_for(&descriptor);
        assert_eq!(digest, CONTRACT_PROTOCOL_SHA256);
        for (from, to) in [
            ("\"version\":1", "\"version\":2"),
            (AGENT_REPORT_PATH, "/changed"),
            (AGENT_REPORT_AUTH_HEADER, "x-changed"),
            ("acceptedAdvancesSequence", "acceptedDoesNotAdvance"),
        ] {
            assert_ne!(digest, contract_protocol_digest_for(&descriptor.replacen(from, to, 1)));
        }
        assert!(validate_protocol_pair(&digest, &digest).is_ok());
        assert!(validate_protocol_pair(&digest, "bad").is_err());
        assert_eq!(
            agent_reports_endpoint("https://monitor.example/"),
            format!("https://monitor.example{}", AGENT_REPORT_PATH)
        );

        let value: serde_json::Value = serde_json::from_str(&descriptor).unwrap();
        assert_eq!(value["method"], AGENT_REPORT_METHOD);
        assert_eq!(value["path"], AGENT_REPORT_PATH);
        assert_eq!(value["auth"]["header"], AGENT_REPORT_AUTH_HEADER);
        assert_eq!(value["limits"]["bodyBytesMax"], MAX_AGENT_REPORT_BODY_BYTES);
        assert_eq!(value["limits"]["clockSkewSeconds"], MAX_REPORT_CLOCK_SKEW_SECONDS);
        assert_eq!(value["limits"]["nodeId"]["min"], 1);
        assert_eq!(value["limits"]["nodeId"]["max"], MAX_NODE_ID_LEN);
        assert_eq!(value["limits"]["nodeId"]["charset"], "ascii-alnum-._-");
        assert_eq!(value["limits"]["hostname"]["minNonblank"], 1);
        assert_eq!(value["limits"]["hostname"]["max"], MAX_HOSTNAME_LEN);
        assert_eq!(value["limits"]["agentVersion"]["minNonblank"], 1);
        assert_eq!(value["limits"]["agentVersion"]["max"], MAX_AGENT_VERSION_LEN);
        assert_eq!(value["limits"]["cpuPercent"]["finite"], true);
        assert_eq!(value["limits"]["cpuPercent"]["min"], 0);
        assert_eq!(value["limits"]["cpuPercent"]["max"], 100);
        assert_eq!(value["limits"]["byteUsage"]["totalMin"], 1);
        assert_eq!(value["limits"]["byteUsage"]["totalMax"], i64::MAX);
        assert_eq!(value["limits"]["byteUsage"]["usedMin"], 0);
        assert_eq!(value["limits"]["byteUsage"]["usedMax"], i64::MAX);
        assert_eq!(value["limits"]["byteUsage"]["usedLessThanOrEqualTotal"], true);
        assert_eq!(value["limits"]["mountPoint"]["minNonblank"], 1);
        assert_eq!(value["limits"]["mountPoint"]["max"], MAX_MOUNT_POINT_LEN);
        assert_eq!(value["limits"]["mountPoint"]["unique"], true);
        assert_eq!(value["limits"]["disksAllowEmpty"], true);
        assert_eq!(value["limits"]["timestamp"], "rfc3339");
        assert_eq!(value["response"]["httpSuccessRequired"], true);
        assert_eq!(value["response"]["code"], RESPONSE_SUCCESS_CODE);
        assert_eq!(value["response"]["message"], RESPONSE_SUCCESS_MESSAGE);
        assert_eq!(
            value["response"]["required"],
            serde_json::json!(["code", "message", "data.status"])
        );
        assert_eq!(value["acceptance"]["acceptedAdvancesSequence"], true);
        assert_eq!(value["acceptance"]["duplicateAdvancesSequence"], true);
        assert_eq!(value["acceptance"]["staleRetainsSequence"], true);
        assert_eq!(value["acceptance"]["errorRetainsSequence"], true);
        assert_eq!(value["acceptance"]["acceptedOrDuplicateAtMax"], "exhausted");
        assert_eq!(value["acceptance"]["fencingBeforeClockSkew"], true);

        let sample = AgentReport {
            node_id: "n".into(),
            boot_id: Uuid::nil(),
            sequence: 1,
            hostname: "h".into(),
            agent_version: "v".into(),
            collected_at: Utc::now(),
            cpu_percent: 1.0,
            memory: ByteUsage { used_bytes: 1, total_bytes: 2 },
            disks: vec![DiskUsage {
                mount_point: "/".into(),
                usage: ByteUsage { used_bytes: 1, total_bytes: 2 },
            }],
        };
        let wire = serde_json::to_value(sample).unwrap();
        let mut keys = wire.as_object().unwrap().keys().cloned().collect::<Vec<_>>();
        keys.sort();
        assert_eq!(
            keys,
            [
                "agentVersion",
                "bootId",
                "collectedAt",
                "cpuPercent",
                "disks",
                "hostname",
                "memory",
                "nodeId",
                "sequence"
            ]
        );
        let mut memory = wire["memory"].as_object().unwrap().keys().cloned().collect::<Vec<_>>();
        memory.sort();
        assert_eq!(memory, ["totalBytes", "usedBytes"]);
        let mut disk = wire["disks"][0].as_object().unwrap().keys().cloned().collect::<Vec<_>>();
        disk.sort();
        assert_eq!(disk, ["mountPoint", "totalBytes", "usedBytes"]);

        for status in ["accepted", "duplicate", "stale"] {
            let body =
                format!(r#"{{"code":0,"message":"Success","data":{{"status":"{status}"}}}}"#);
            assert!(parse_agent_response(true, &body).is_ok());
        }
        assert!(parse_agent_response(false, "{}").is_err());
        assert!(
            parse_agent_response(
                true,
                r#"{"code":1,"message":"Success","data":{"status":"accepted"}}"#
            )
            .is_err()
        );
        assert!(
            parse_agent_response(true, r#"{"code":0,"message":"No","data":{"status":"accepted"}}"#)
                .is_err()
        );
    }
}
