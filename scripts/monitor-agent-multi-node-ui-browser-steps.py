#!/usr/bin/env python3
import json
import os
import sys


if len(sys.argv) != 3 or sys.argv[1] == sys.argv[2]:
    raise SystemExit("expected two distinct Monitor node IDs")


def selector(name, node_id):
    return f'[data-testid="monitor-node-{name}-{node_id}"]'


def details(node_id, boot_id):
    history = selector("history-5m", node_id)
    view = f'[data-testid="monitor-node-view"][data-node-id="{node_id}"]'
    detail_boot = selector("details-boot-id", node_id)
    return [
        {"action": "waitFor", "selector": view},
        {"action": "assertText", "selector": selector("boot-id", node_id), "text": boot_id},
        {"action": "click", "selector": view},
        {"action": "waitFor", "selector": "[data-testid=monitor-node-details]"},
        {"action": "waitFor", "selector": detail_boot},
        {"action": "assertText", "selector": detail_boot, "text": boot_id},
        {"action": "waitFor", "selector": history},
        {"action": "assertText", "selector": history, "text": "5-minute history"},
        {"action": "waitFor", "selector": f"{history} .recharts-line-dot"},
        {"action": "click", "selector": f"{history} .recharts-line-dot"},
        {"action": "assertElementLayout", "selector": f"{history} .recharts-line-dot", "elementCount": 2, "visibleCount": 2, "maxHeight": None, "withinViewportRight": False, "withinViewport": True},
        {"action": "screenshotViewport", "name": f"monitor-agent-{node_id}-detail-dark-en"},
        {"action": "pressKey", "key": "Escape"},
    ]


first, second = sys.argv[1:]
login = [
    {"action": "goto", "url": "/login"},
    {"action": "waitFor", "selector": "#login_username"},
    {"action": "fill", "selector": "#login_username", "value": "owner"},
    {"action": "fill", "selector": "#login_password", "value": "rustzen@123"},
    {"action": "click", "selector": "button[type=submit]"},
    {"action": "waitFor", "selector": ".shell-content"},
]
steps = [
    {"action": "setUiPreferences", "theme": "dark", "locale": "en-US"},
    {"action": "setViewport", "width": 1440, "height": 900},
    *login,
    {"action": "goto", "url": "/monitoring/nodes"},
    {"action": "waitFor", "selector": "[data-testid=monitor-nodes-table]"},
    *details(first, os.environ.get("RUSTZEN_VERIFY_BOOT_A", "")),
    *details(second, os.environ.get("RUSTZEN_VERIFY_BOOT_B", "")),
    {"action": "assertNoHorizontalOverflow"},
]
print(json.dumps(steps, separators=(",", ":")))
