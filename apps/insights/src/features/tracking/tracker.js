(() => {
    const script = document.currentScript;
    const originalFetch = window.fetch;
    const nativeFetch = originalFetch.bind(window);
    const nativeOpen = XMLHttpRequest.prototype.open;
    const nativeSend = XMLHttpRequest.prototype.send;
    const allowedEvents = new Set(["page_view", "api_request", "custom_export"]);
    const allowedFields = new Set([
        "pagePath",
        "referrer",
        "apiPath",
        "apiMethod",
        "statusCode",
        "durationMs",
        "isError",
        "properties",
    ]);
    const allowedPropertyKeys = new Set(["feature", "format", "result", "status"]);
    const VISITOR_KEY = "rz_vid";
    const VISITOR_AT_KEY = "rz_vid_at";
    const SESSION_KEY = "rz_sid";
    const SESSION_CREATED_AT_KEY = "rz_sid_created_at";
    const SESSION_LAST_SEEN_AT_KEY = "rz_sid_last_seen_at";
    const LEGACY_SESSION_AT_KEY = "rz_sid_at";
    const VISITOR_TTL_MS = 24 * 60 * 60 * 1000;
    const SESSION_IDLE_MS = 30 * 60 * 1000;
    const SESSION_TTL_MS = 24 * 60 * 60 * 1000;
    const MAX_QUEUE = 1000;
    const MAX_BATCH = 50;
    const MAX_TRANSPORT_RETRIES = 3;
    const RETRY_BASE_DELAY_MS = 500;
    const RETRY_MAX_DELAY_MS = 4000;

    let enabled = false;
    let endpoint;
    let projectKey;
    let visitorId;
    let sessionId;
    let queue = [];
    let sendTimer;
    let sendInFlight = false;
    let transportObserver;
    const xhrMetadata = new WeakMap();

    const pathname = (value) => {
        try {
            return new URL(value || "/", location.href).pathname || "/";
        } catch {
            return "/";
        }
    };

    const referrerPath = () => {
        if (!document.referrer) return undefined;
        return pathname(document.referrer);
    };

    const endpointMatches = (url) =>
        endpoint && url.origin === endpoint.origin && url.pathname === endpoint.pathname;

    const readStorageId = (storage, key, atKey, ttl) => {
        const id = storage.getItem(key);
        const valueAt = Number(storage.getItem(atKey));
        const now = Date.now();
        const expired = !id || !Number.isFinite(valueAt) || now - valueAt >= ttl;
        if (expired) {
            const next = crypto.randomUUID();
            storage.setItem(key, next);
            storage.setItem(atKey, String(now));
            return next;
        }
        return id;
    };

    const readSessionId = () => {
        const id = sessionStorage.getItem(SESSION_KEY);
        const createdAt = Number(sessionStorage.getItem(SESSION_CREATED_AT_KEY));
        const lastSeenAt = Number(sessionStorage.getItem(SESSION_LAST_SEEN_AT_KEY));
        const now = Date.now();
        const expired =
            !id ||
            !Number.isFinite(createdAt) ||
            !Number.isFinite(lastSeenAt) ||
            now - createdAt >= SESSION_TTL_MS ||
            now - lastSeenAt >= SESSION_IDLE_MS;
        if (expired) {
            const next = crypto.randomUUID();
            sessionStorage.setItem(SESSION_KEY, next);
            sessionStorage.setItem(SESSION_CREATED_AT_KEY, String(now));
            sessionStorage.setItem(SESSION_LAST_SEEN_AT_KEY, String(now));
            sessionStorage.removeItem(LEGACY_SESSION_AT_KEY);
            return next;
        }
        sessionStorage.setItem(SESSION_LAST_SEEN_AT_KEY, String(now));
        return id;
    };

    const refreshIds = () => {
        visitorId = readStorageId(localStorage, VISITOR_KEY, VISITOR_AT_KEY, VISITOR_TTL_MS);
        sessionId = readSessionId();
    };

    const clearIds = () => {
        localStorage.removeItem(VISITOR_KEY);
        localStorage.removeItem(VISITOR_AT_KEY);
        sessionStorage.removeItem(SESSION_KEY);
        sessionStorage.removeItem(SESSION_CREATED_AT_KEY);
        sessionStorage.removeItem(SESSION_LAST_SEEN_AT_KEY);
        sessionStorage.removeItem(LEGACY_SESSION_AT_KEY);
        visitorId = undefined;
        sessionId = undefined;
    };

    const observeTransport = (type, details = {}) => {
        if (typeof transportObserver !== "function") return;
        try {
            transportObserver({ type, ...details });
        } catch {
            // Observability must never affect collection or host requests.
        }
    };

    const requestDetails = (input, init) => {
        const request = input instanceof Request ? input : undefined;
        const url = new URL(request?.url || String(input), location.href);
        return {
            url,
            method: String(init?.method || request?.method || "GET").toUpperCase(),
            trackable: !endpointMatches(url),
        };
    };

    const safeProperties = (properties) => {
        if (!properties || typeof properties !== "object" || Array.isArray(properties)) return {};
        return Object.fromEntries(
            Object.entries(properties).filter(([key, value]) => {
                if (!allowedPropertyKeys.has(key)) return false;
                return (
                    typeof value === "string" ||
                    typeof value === "number" ||
                    typeof value === "boolean"
                );
            }),
        );
    };

    // Only these responses have an explicit all-or-nothing contract at the
    // ingestion boundary. A 5xx or a network failure is ambiguous: the
    // server may have committed the batch before the response was lost, so
    // retrying it would duplicate events without an idempotency key.
    const temporaryStatus = (status) => status === 429 || status === 507;
    const ambiguousStatus = (status) => status >= 500 && status <= 599;

    const retryDelay = (attempt) =>
        Math.min(RETRY_MAX_DELAY_MS, RETRY_BASE_DELAY_MS * 2 ** attempt);

    const send = () => {
        if (!enabled || !queue.length || sendInFlight) return;
        const batch = queue.splice(0, MAX_BATCH);
        sendInFlight = true;

        const sendAttempt = async (attempt) => {
            if (!enabled) {
                sendInFlight = false;
                return;
            }
            try {
                const response = await nativeFetch(endpoint.href, {
                    method: "POST",
                    headers: {
                        "content-type": "application/json",
                        "x-rustzen-project-key": projectKey,
                    },
                    body: JSON.stringify(batch),
                    keepalive: true,
                });
                if (response.ok) {
                    observeTransport("accepted", { status: response.status, attempt });
                    sendInFlight = false;
                    return;
                }
                if (temporaryStatus(response.status) && attempt < MAX_TRANSPORT_RETRIES) {
                    const delayMs = retryDelay(attempt);
                    observeTransport("temporary_retry", {
                        status: response.status,
                        attempt,
                        delayMs,
                    });
                    setTimeout(() => sendAttempt(attempt + 1), delayMs);
                    return;
                }
                if (temporaryStatus(response.status)) {
                    observeTransport("temporary_dropped", {
                        status: response.status,
                        attempt,
                        dropped: batch.length,
                    });
                } else if (ambiguousStatus(response.status)) {
                    observeTransport("temporary_dropped", {
                        status: response.status,
                        attempt,
                        dropped: batch.length,
                        reason: "ambiguous_response",
                    });
                } else {
                    observeTransport("validation_rejected", {
                        status: response.status,
                        attempt,
                    });
                }
                sendInFlight = false;
            } catch {
                // Network errors are ambiguous for a non-idempotent batch.
                // Drop once and expose the reason instead of retrying.
                observeTransport("temporary_dropped", {
                    attempt,
                    dropped: batch.length,
                    reason: "network_error",
                });
                sendInFlight = false;
            }
        };

        void sendAttempt(0);
    };

    const track = (eventName, fields = {}) => {
        if (!enabled || !allowedEvents.has(eventName)) return;
        refreshIds();
        const input = fields && typeof fields === "object" ? fields : {};
        const payload = {
            eventName,
            visitorId,
            sessionId,
            platform: navigator.userAgentData?.platform || navigator.platform,
            occurredAt: new Date().toISOString(),
        };
        Object.entries(input).forEach(([key, value]) => {
            if (!allowedFields.has(key)) return;
            if (value === undefined || value === null) return;
            if (key === "pagePath" || key === "apiPath" || key === "referrer") {
                payload[key] = pathname(value);
            } else if (key === "properties") {
                payload.properties = safeProperties(value);
            } else {
                payload[key] = value;
            }
        });
        queue.push(payload);
        if (queue.length >= MAX_BATCH) send();
    };

    const restoreHooks = () => {
        window.fetch = originalFetch;
        XMLHttpRequest.prototype.open = nativeOpen;
        XMLHttpRequest.prototype.send = nativeSend;
    };

    const optOut = () => {
        const wasEnabled = enabled;
        enabled = false;
        queue = [];
        sendInFlight = false;
        transportObserver = undefined;
        if (sendTimer) {
            clearInterval(sendTimer);
            sendTimer = undefined;
        }
        if (wasEnabled) restoreHooks();
        clearIds();
    };

    const installHooks = () => {
        window.fetch = async (input, init) => {
            const details = requestDetails(input, init);
            if (!details.trackable) return nativeFetch(input, init);
            const started = performance.now();
            try {
                const response = await nativeFetch(input, init);
                track("api_request", {
                    apiPath: details.url.pathname,
                    apiMethod: details.method,
                    statusCode: response.status,
                    durationMs: Math.round(performance.now() - started),
                    isError: !response.ok,
                });
                return response;
            } catch (error) {
                track("api_request", {
                    apiPath: details.url.pathname,
                    apiMethod: details.method,
                    durationMs: Math.round(performance.now() - started),
                    isError: true,
                });
                throw error;
            }
        };

        XMLHttpRequest.prototype.open = function (method, url, ...rest) {
            xhrMetadata.set(this, requestDetails(url, { method }));
            return nativeOpen.call(this, method, url, ...rest);
        };
        XMLHttpRequest.prototype.send = function (...args) {
            const details = xhrMetadata.get(this);
            if (details?.trackable) {
                const started = performance.now();
                this.addEventListener(
                    "loadend",
                    () => {
                        track("api_request", {
                            apiPath: details.url.pathname,
                            apiMethod: details.method,
                            statusCode: this.status || undefined,
                            durationMs: Math.round(performance.now() - started),
                            isError: this.status === 0 || this.status >= 400,
                        });
                    },
                    { once: true },
                );
            }
            return nativeSend.apply(this, args);
        };
    };

    const enable = (options = {}) => {
        if (options.consent !== true) {
            optOut();
            return false;
        }
        const nextProjectKey = String(
            options.projectKey || script?.dataset.projectKey || "",
        ).trim();
        if (!nextProjectKey) return false;
        if (enabled) return true;
        const nextEndpoint = options.endpoint || script?.dataset.endpoint || "/api/insights/track";
        endpoint = new URL(nextEndpoint, script?.src || location.href);
        projectKey = nextProjectKey;
        transportObserver =
            typeof options.onTransportEvent === "function" ? options.onTransportEvent : undefined;
        enabled = true;
        installHooks();
        if (!sendTimer) sendTimer = setInterval(send, 5000);
        track("page_view", { pagePath: location.pathname, referrer: referrerPath() });
        addEventListener("pagehide", send, { once: false });
        return true;
    };

    window.rustzenAnalytics = {
        enable,
        optOut,
        track,
        identify: () => undefined,
    };
    window.rustzenAnalyticsBootstrap = enable;
})();
