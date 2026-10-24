def overview_payload():
    return {
        "registeredNodes": 1,
        "onlineNodes": 1,
        "offlineNodes": 0,
        "activeIncidents": 21,
        "latestResource": {
            "nodeId": "fixture-overview-node",
            "cpuPercent": 21.0,
            "memoryPercent": 34.0,
            "disks": [
                {
                    "mountPoint": "/",
                    "usedBytes": 55,
                    "totalBytes": 100,
                    "usagePercent": 55.0,
                }
            ],
            "collectedAt": "2026-09-07T00:00:00Z",
            "lastReceivedAt": "2026-09-07T00:00:01Z",
        },
    }


def nodes_payload():
    return [
        {
            "nodeId": "fixture-node",
            "hostname": "fixture-node",
            "agentVersion": "verify",
            "bootId": "fixture-boot",
            "sequence": 1,
            "lastReportAt": "2026-09-07T00:00:00Z",
            "lastReceivedAt": "2026-09-07T00:00:01Z",
            "status": "online",
            "alertPolicySource": "global",
            "cpuPercent": 10.0,
            "memory": {
                "usedBytes": 10,
                "totalBytes": 100,
                "usagePercent": 10.0,
            },
            "disks": [
                {
                    "mountPoint": "/",
                    "collectedAt": "2026-09-07T00:00:00Z",
                    "usedBytes": 55,
                    "totalBytes": 100,
                    "usagePercent": 55.0,
                }
            ],
            "createdAt": "2026-09-07T00:00:00Z",
            "updatedAt": "2026-09-07T00:00:00Z",
        }
    ]


def incident_rows():
    return [
        {
            "id": f"incident-{index}",
            "nodeId": "fixture-node",
            "kind": "cpuHigh",
            "target": f"cpu-{index}",
            "status": "active",
            "title": f"Fixture incident {index}",
            "thresholdPercent": 90.0,
            "observedPercent": 95.0,
            "openedAt": "2026-09-07T00:00:00Z",
            "lastObservedAt": "2026-09-07T00:00:00Z",
            "resolvedAt": None,
            "resolutionReason": None,
            "details": {},
        }
        for index in range(1, 22)
    ]


def summary_rows():
    return [
        {
            "nodeId": "fixture-node",
            "date": f"2026-08-{index:02d}",
            "sampleCount": 3,
            "coverage": 100.0,
            "coveragePercent": 100.0,
            "cpu": {"min": 10.0, "avg": 20.0, "max": 30.0},
            "memory": {"min": 11.0, "avg": 21.0, "max": 31.0},
            "diskSummary": {
                "/": {"min": 12.0, "avg": 22.0, "max": 32.0},
            },
            "offlineSeconds": 0,
            "incidentCount": index,
        }
        for index in range(1, 22)
    ]


def page_payload(rows, query):
    current = int(query.get("current", ["1"])[0])
    page_size = int(query.get("pageSize", ["20"])[0])
    start = (current - 1) * page_size
    return {
        "data": rows[start : start + page_size],
        "total": len(rows),
        "success": True,
    }


def route_payload(name, query):
    if name == "overview":
        return overview_payload()
    if name == "nodes":
        return nodes_payload()
    if name == "incidents":
        return page_payload(incident_rows(), query)
    return page_payload(summary_rows(), query)


