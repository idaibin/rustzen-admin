import { expect, test } from "bun:test";
import { readFile, stat } from "node:fs/promises";

const native = await readFile("scripts/verify-monitor-native-runtime-linux-amd64.sh", "utf8"), prepare = await readFile("scripts/prepare-monitor-load-runtime.sh", "utf8"), cleanup = await readFile("scripts/cleanup-monitor-load-runtime.sh", "utf8"), preflight = await readFile("scripts/verify-monitor-load-runtime-preflight.ts", "utf8");

test("P8e default cleanup and explicit retain use distinct bounded Docker paths", () => {
    expect(native).toContain('--retain-container-output');
    expect(native).toContain('test "$status" -ne 0 || test -z "${retain_output:-}"');
    expect(native).toContain('--cgroupns=private --cpus 4 --memory 512m --pids-limit 256 -p 127.0.0.1::19801');
    expect(native).toContain('ownerToken}));');
    expect(native).toContain('--label "io.rustzen.p8g-owner=${retain_owner:?}"');
});

test("prepare runs P8f in retained context and cleanup removes the exact context resources", async () => {
    expect(prepare).toContain('--retain-container-output "$native_context"');
    expect(prepare).toContain('--retain-owner-token "$owner_token"');
    expect(prepare).toContain('label=io.rustzen.p8g-owner=$owner_token');
    expect(prepare).toContain('realpath "$export_root/release/server/bin/rz-admin"');
    expect(prepare).not.toContain('--admin-bin "$native_output/rz"');
    expect(prepare).toContain('verify-selected-web-bootstrap-browser.py --selection distribution/fixtures/monitor.json --admin-url "$admin_url"');
    expect(prepare).not.toContain('--selection distribution/fixtures/monitor-notify.json');
    expect((await stat("scripts/verify-selected-web-bootstrap-browser.py")).mode & 0o111).not.toBe(0);
    expect(prepare).toContain('const [health,native]');
    expect(prepare).toContain('const s=native.selection');
    expect(prepare).not.toContain('const s=release.selection');
    expect(prepare).toContain('expectedSourceIdentity:source,exportRoot,nativeEvidence:value.nativeEvidence');
    expect(prepare).toContain('docker rm -f "$ids"');
    expect(cleanup).toContain('docker rm -f "$container"');
    expect(cleanup).toContain('$actual_owner" = "$owner_token"');
    expect(cleanup).toContain('docker inspect --format');
    expect(cleanup).toContain('"$actual" = "$container_id"');
    expect(cleanup).toContain('rm -f "$password" "$agent" "$context"');
    expect(preflight).toContain('["containerId", "imageId", "hostPort", "containerPort"]');
    expect(preflight).toContain('["pid", "dev", "ino", "sha256"]');
    expect(preflight).not.toContain('canonicalJson(admin) !== canonicalJson(admission.runtime)');
});

test("cleanup keeps a reused name out of docker rm while still removing secrets", () => {
    expect(cleanup).toContain('$actual" = "$container_id" -a "$actual_owner" = "$owner_token"');
    expect(cleanup).toContain('rm -f "$password" "$agent" "$context"');
});

test("prepare failure cleanup removes only its retained container, credentials and browser output", () => {
    expect(prepare).toContain('if test "$status" -ne 0; then');
    expect(prepare).toContain('label=io.rustzen.p8g-owner=$owner_token');
    expect(prepare).toContain('test -z "$ids" || docker rm -f "$ids"');
    expect(prepare).toContain('rm -f "$password" "$agent" "$context_output"');
    expect(prepare).toContain('rm -rf "$browser_output"');
});
