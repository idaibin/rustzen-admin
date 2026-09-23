#!/usr/bin/env python3
import base64, hashlib, json, sys, time
from pathlib import Path
from selected_web_bootstrap_cdp import CDP

base, password, output = sys.argv[1:]
out = Path(output)
cdp = CDP("/usr/bin/chromium")

def wait(expr, label):
    deadline = time.monotonic() + 30
    while time.monotonic() < deadline:
        if cdp.evaluate(expr): return
        time.sleep(.1)
    raise RuntimeError("timed out: " + label)

def go(path):
    cdp.call("Page.navigate", {"url": base + path})

def capture(name):
    raw = base64.b64decode(cdp.call("Page.captureScreenshot", {"format":"png"})["data"])
    if raw[:8] != b"\x89PNG\r\n\x1a\n" or int.from_bytes(raw[16:20], "big") != 1920 or int.from_bytes(raw[20:24], "big") != 1080: raise RuntimeError("invalid screenshot dimensions")
    path = out / (name + ".png"); path.write_bytes(raw)
    return {"route": cdp.evaluate("location.pathname"), "file":path.name, "sha256":hashlib.sha256(raw).hexdigest(), "bytes":len(raw), "dimensions":{"width":1920,"height":1080}}

def page_state():
    return cdp.evaluate("""(() => {
        const visible = (element) => {
            if (!element) return false;
            const style = getComputedStyle(element);
            const box = element.getBoundingClientRect();
            return style.visibility !== 'hidden' && style.display !== 'none' && box.width > 0 && box.height > 0;
        };
        const text = (document.body.innerText || '').trim();
        const title = [...document.querySelectorAll('h1,h2,.ant-card-head-title')]
            .filter(visible)
            .map((element) => (element.textContent || '').trim())
            .find(Boolean) || '';
        const loading = [...document.querySelectorAll('.ant-spin-spinning,[aria-busy=true]')].some(visible);
        const error = [...document.querySelectorAll('[role=alert],.ant-result-error')]
            .filter(visible)
            .map((element) => (element.textContent || '').trim())
            .find(Boolean) || '';
        const empty = /暂无|没有|为空|No data|No .* found|Empty/i.test(text);
        return {
            route: location.pathname,
            documentTitle: document.title,
            pageTitle: title,
            authenticated: !!document.querySelector('button[aria-label="账号菜单"],button[aria-label="Account menu"]'),
            loading,
            error,
            dataState: empty ? 'empty' : 'populated',
            viewport: {width: innerWidth, height: innerHeight, devicePixelRatio},
        };
    })()""")

def wait_page(route, label):
    wait("location.pathname === " + json.dumps(route) + " && !!document.querySelector('.shell-content')", label + " route")
    deadline = time.monotonic() + 30
    state = None
    while time.monotonic() < deadline:
        state = page_state()
        if state and state["pageTitle"] and not state["loading"]:
            break
        time.sleep(.1)
    if not state or not state["pageTitle"] or state["loading"]:
        raise RuntimeError(label + " did not reach a titled settled state")
    if state["route"] != route or not state["authenticated"]:
        raise RuntimeError(label + " route or authenticated state differs")
    if state["viewport"] != {"width":1920,"height":1080,"devicePixelRatio":1}:
        raise RuntimeError(label + " viewport differs")
    if state["documentTitle"] != "Rustzen Admin":
        raise RuntimeError(label + " document title differs")
    if state["error"]:
        raise RuntimeError(label + " rendered an error alert: " + state["error"])
    return state

def assert_text(name, values):
    body = cdp.evaluate("document.body.innerText") or ""
    missing = [value for value in values if value not in body]
    if missing: raise RuntimeError(f"{name} missing DOM text: {missing}")
    return {"route": cdp.evaluate("location.pathname"), "contains":values}

try:
    cdp.call("Emulation.setDeviceMetricsOverride", {"width":1920,"height":1080,"deviceScaleFactor":1,"mobile":False})
    go("/login"); wait("!!document.querySelector('#login_username')", "login form")
    cdp.evaluate("""(() => { const put=(s,v)=>{const e=document.querySelector(s),d=Object.getOwnPropertyDescriptor(HTMLInputElement.prototype,'value');d.set.call(e,v);e.dispatchEvent(new Event('input',{bubbles:true}));e.dispatchEvent(new Event('change',{bubbles:true}));}; put('#login_username','owner'); put('#login_password',%s); document.querySelector('button[type=submit]').click(); })()""" % json.dumps(password))
    wait("!!document.querySelector('.shell-content')", "application shell")
    shots=[]; assertions=[]; pages=[]
    routes = [
        ("dashboard", "/", []),
        ("profile", "/profile", []),
        ("monitoring-overview", "/monitoring/overview", []),
        ("monitoring-nodes", "/monitoring/nodes", ["preview-node-a", "preview-node-b"]),
        ("monitoring-incidents", "/monitoring/incidents", []),
        ("monitoring-summaries", "/monitoring/summaries", []),
        ("analytics-overview", "/analytics/overview", []),
        ("analytics-details", "/analytics/details", ["preview-visitor-001", "/release-preview"]),
        ("reports-templates", "/reports/templates", ["Release preview automatic report"]),
        ("reports-runs", "/reports/runs", []),
        ("system-users", "/system/user", []),
        ("system-roles", "/system/role", []),
        ("system-menus", "/system/menu", []),
        ("system-modules", "/system/module", []),
        ("system-status", "/system/status", []),
        ("system-module-logs", "/system/module-log", []),
        ("management-operation-logs", "/manage/log", []),
        ("management-scheduled-tasks", "/manage/task", []),
        ("management-deployments", "/manage/deploy", []),
    ]
    for name, route, values in routes:
        go(route); state = wait_page(route, name); time.sleep(.7)
        if name == "monitoring-nodes":
            online = cdp.evaluate("(document.body.innerText.match(/(在线|Online)/g)||[]).length")
            if online < 2: raise RuntimeError("nodes does not show two online states")
            assertions.append({"route":route,"contains":values,"onlineStateCount":online})
        elif name == "reports-templates":
            body = cdp.evaluate("document.body.innerText") or ""
            cadence = "每日" if "每日" in body else "Daily" if "Daily" in body else None
            enabled = "已启用" if "已启用" in body else "On" if "On" in body else None
            if not cadence or not enabled: raise RuntimeError("schedule does not show daily enabled state")
            assertions.append({"route":route,"contains":values,"cadence":cadence,"enabledState":enabled})
        elif values: assertions.append(assert_text(name, values))
        shots.append(capture(name))
        pages.append(state)
    (out / "browser-direct.json").write_text(json.dumps({"viewport":{"width":1920,"height":1080},"routeCount":len(routes),"pages":pages,"domAssertions":assertions,"screenshots":shots},separators=(",",":")))
finally:
    cdp.close()
