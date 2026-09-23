# shellcheck shell=sh
# Sourced by verify-services.sh after its runtime helpers and cleanup trap are installed.
# It intentionally relies on that verifier's ROOT, credentials, curl helpers, and run_bun wrapper.

verify_module_log_diagnostics() {
    module_log_date="$(run_bun -e 'console.log(new Date().toISOString().slice(0, 10))')"
    module_log_old_date="$(run_bun -e 'const date = new Date(); date.setUTCDate(date.getUTCDate() - 90); console.log(date.toISOString().slice(0, 10))')"
    module_log_dir="$ROOT/logs"
    mkdir -p "$module_log_dir/admin" "$module_log_dir/monitor"
    MODULE_LOG_DIR="$module_log_dir" MODULE_LOG_DATE="$module_log_date" MODULE_LOG_OLD_DATE="$module_log_old_date" run_bun -e '
        const dir = process.env.MODULE_LOG_DIR;
        const tailFixture = Array.from(
            { length: 24_000 },
            (_, index) => `fixture-${String(index).padStart(6, "0")}\n`,
        ).join("");
        await Bun.write(`${dir}/admin/admin.${process.env.MODULE_LOG_DATE}`, tailFixture);
        await Bun.write(`${dir}/monitor/monitor.${process.env.MODULE_LOG_OLD_DATE}`, "cleanup-candidate\n");
    '

    module_log_list="$(curl --fail --silent --show-error \
        -H "authorization: Bearer $RUSTZEN_ADMIN_TOKEN" \
        "http://127.0.0.1:$RUSTZEN_ADMIN_PORT/api/system/status/module-logs?module=admin&date=$module_log_date")"
    MODULE_LOG_LIST="$module_log_list" MODULE_LOG_DATE="$module_log_date" run_bun -e '
        const payload = JSON.parse(process.env.MODULE_LOG_LIST);
        if (!payload.data?.some((item) => item.module === "admin" && item.date === process.env.MODULE_LOG_DATE && item.readable)) {
            throw new Error(`module-log list did not return the readable admin fixture: ${JSON.stringify(payload)}`);
        }
    '
    wait_for_status 403 "http://127.0.0.1:$RUSTZEN_ADMIN_PORT/api/system/status/module-logs?module=admin&date=$module_log_date" "$denied_token"

    MODULE_LOG_BASE_URL="http://127.0.0.1:$RUSTZEN_ADMIN_PORT" MODULE_LOG_DATE="$module_log_date" MODULE_LOG_TOKEN="$RUSTZEN_ADMIN_TOKEN" run_bun -e '
        const expectedLines = 24_000;
        const pages = [];
        let cursor;
        for (let pageIndex = 0; pageIndex < 32; pageIndex += 1) {
            const params = new URLSearchParams({ module: "admin", date: process.env.MODULE_LOG_DATE });
            if (cursor) params.set("cursor", cursor);
            const response = await fetch(`${process.env.MODULE_LOG_BASE_URL}/api/system/status/module-logs/tail?${params}`, {
                headers: { authorization: `Bearer ${process.env.MODULE_LOG_TOKEN}` },
            });
            if (!response.ok) throw new Error(`module-log tail returned HTTP ${response.status}`);
            const tail = (await response.json()).data;
            const ids = tail.content ? tail.content.split("\n").map((line) => Number(line.slice("fixture-".length))) : [];
            if (tail.byteCount > 256 * 1024 || tail.lineCount > 2000 || ids.length !== tail.lineCount) {
                throw new Error(`module-log tail cap contract failed: ${JSON.stringify(tail)}`);
            }
            if (ids.some((id, index) => !Number.isInteger(id) || (index > 0 && id !== ids[index - 1] + 1))) {
                throw new Error(`module-log tail page is not an ordered fixture range: ${JSON.stringify(ids)}`);
            }
            pages.push(ids);
            if (!tail.nextCursor) break;
            if (!tail.truncated || tail.nextCursor === cursor) {
                throw new Error(`module-log cursor did not advance: ${JSON.stringify(tail)}`);
            }
            cursor = tail.nextCursor;
        }
        if (pages.length < 2 || pages.at(-1)?.length === 0 || pages.at(-1)?.[0] !== 0) {
            throw new Error(`module-log tail did not converge: ${JSON.stringify(pages)}`);
        }
        const first = pages[0];
        const second = pages[1];
        if (first.some((id) => second.includes(id)) || first[0] <= second.at(-1)) {
            throw new Error(`module-log first and second pages overlap or are unordered: ${JSON.stringify({ first, second })}`);
        }
        const reconstructed = pages.toReversed().flat();
        if (reconstructed.length !== expectedLines || reconstructed.some((id, index) => id !== index)) {
            throw new Error(`module-log tail reconstruction failed: ${JSON.stringify({ length: reconstructed.length, first: reconstructed[0], last: reconstructed.at(-1) })}`);
        }
    '

    module_log_headers="$ROOT/module-log-backup.headers"
    module_log_archive="$ROOT/module-log-backup.tar"
    curl --fail --silent --show-error \
        -D "$module_log_headers" \
        -o "$module_log_archive" \
        -H "authorization: Bearer $RUSTZEN_ADMIN_TOKEN" \
        -H 'content-type: application/json' \
        -d "{\"files\":[{\"module\":\"admin\",\"date\":\"$module_log_date\"}]}" \
        "http://127.0.0.1:$RUSTZEN_ADMIN_PORT/api/system/status/module-logs/backup"
    MODULE_LOG_HEADERS="$module_log_headers" MODULE_LOG_ARCHIVE="$module_log_archive" run_bun -e '
        const headers = await Bun.file(process.env.MODULE_LOG_HEADERS).text();
        const value = (name) => headers.match(new RegExp(`^${name}:\s*(.+)\r?$`, "im"))?.[1]?.trim();
        if (!/^attachment;\s*filename=rustzen-module-logs\.tar$/i.test(value("content-disposition") ?? "")) throw new Error("missing backup filename header");
        const archiveSha256 = value("x-rustzen-archive-sha256");
        if (!/^[a-f0-9]{64}$/.test(archiveSha256 ?? "")) throw new Error("missing backup hash header");
        if (value("x-rustzen-archive-file-count") !== "1") throw new Error("backup count header did not match one selected file");
        const archive = await Bun.file(process.env.MODULE_LOG_ARCHIVE).arrayBuffer();
        if (archive.byteLength === 0) throw new Error("empty backup archive");
        const digest = [...new Uint8Array(await crypto.subtle.digest("SHA-256", archive))].map((byte) => byte.toString(16).padStart(2, "0")).join("");
        if (digest !== archiveSha256) throw new Error("backup archive hash does not match its header");
    '

    cleanup_preview="$(curl --fail --silent --show-error \
        -X POST -H "authorization: Bearer $RUSTZEN_ADMIN_TOKEN" \
        "http://127.0.0.1:$RUSTZEN_ADMIN_PORT/api/system/status/module-logs/cleanup/preview")"
    cleanup_token="$(CLEANUP_PREVIEW="$cleanup_preview" MODULE_LOG_OLD_DATE="$module_log_old_date" run_bun -e '
        const preview = JSON.parse(process.env.CLEANUP_PREVIEW).data;
        if (typeof preview?.token !== "string" || !preview.candidates?.some((item) => item.module === "monitor" && item.date === process.env.MODULE_LOG_OLD_DATE)) {
            throw new Error(`module-log cleanup preview fixture missing: ${JSON.stringify(preview)}`);
        }
        console.log(preview.token);
    ')"
    printf '%s\n' changed >>"$module_log_dir/monitor/monitor.$module_log_old_date"
    cleanup_result="$(curl --fail --silent --show-error \
        -X POST -H "authorization: Bearer $RUSTZEN_ADMIN_TOKEN" -H 'content-type: application/json' \
        -d "{\"token\":\"$cleanup_token\"}" \
        "http://127.0.0.1:$RUSTZEN_ADMIN_PORT/api/system/status/module-logs/cleanup/confirm")"
    CLEANUP_RESULT="$cleanup_result" run_bun -e '
        const result = JSON.parse(process.env.CLEANUP_RESULT).data;
        if (!result.partial || !Array.isArray(result.failures) || result.failures.length === 0) {
            throw new Error(`module-log cleanup change must be partial: ${JSON.stringify(result)}`);
        }
    '
    repeated_status="$(http_status -X POST -H "authorization: Bearer $RUSTZEN_ADMIN_TOKEN" -H 'content-type: application/json' \
        -d "{\"token\":\"$cleanup_token\"}" \
        "http://127.0.0.1:$RUSTZEN_ADMIN_PORT/api/system/status/module-logs/cleanup/confirm")"
    [ "$repeated_status" = 400 ] || { echo "verify-services: reused module-log cleanup token returned $repeated_status" >&2; exit 1; }
}
