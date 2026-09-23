import { expect, test } from "bun:test";

const helper = new URL("./reports-ui-state-browser-lib.sh", import.meta.url).pathname;
const run = (body) => Bun.spawnSync(["bash", "-c", `
    set -u
    reports_ui_state_evidence_dir=$(mktemp -d)
    admin=http://stub
    owner_auth=()
    . "$1"
    ${body}
`, "browser-lib", helper], { stdout: "pipe", stderr: "pipe" });
const output = (result) => new TextDecoder().decode(result.stdout);

test("browser helper fails closed without success descriptors", () => {
    const failures = [
        `create_flow(){ printf flow; }; create_run(){ printf run; }; wait_for_status(){ return 1; }; run_browser_case case '[]'`,
        `curl_json(){ return 22; }; download_screenshot run image artifact.png case 1 1`,
        `curl_json(){ printf '{"data":[]}'; }; download_screenshot run image artifact.png case 1 1`,
        `curl_json(){ if [[ "$*" == *artifacts$ ]]; then printf '{"data":[{"fileName":"image","id":"id"}]}'; else return 22; fi; }; download_screenshot run image artifact.png case 1 1`,
        `curl_json(){ if [[ "$*" == *artifacts$ ]]; then printf '{"data":[{"fileName":"image","id":"id"}]}'; else printf text; fi; }; download_screenshot run image artifact.png case 1 1`,
        `curl_json(){ return 22; }; save_run_steps run steps.json`,
        `curl_json(){ printf '{"data":[]}'; }; save_run_steps run steps.json`,
        `curl_json(){ printf '{"data":[{"runId":"other"}]}'; }; save_run_steps run steps.json`,
    ];
    for (const body of failures) {
        const result = run(body);
        expect(result.exitCode).not.toBe(0);
        expect(output(result)).toBe("");
    }
});

test("browser helper emits descriptors only after successful evidence", () => {
    const result = run(`
        create_flow(){ printf flow; }; create_run(){ printf run; }; wait_for_status(){ return 0; }
        curl_json(){
          if [[ "$*" == *'/artifacts/id' ]]; then printf 'iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVQIHWP4z8DwHwAFgAI/ScL9YQAAAABJRU5ErkJggg==' | base64 -d;
          elif [[ "$*" == *'/artifacts' ]]; then printf '{"data":[{"fileName":"image","id":"id"}]}';
          elif [[ "$*" == *'/steps' ]]; then printf '{"data":[{"runId":"run"}]}'; fi
        }
        printf 'run='; run_browser_case case '[]'
        printf 'artifact='; download_screenshot run image artifact-success.png case 1 1
        printf 'steps='; save_run_steps run steps-success.json
    `);
    expect(result.exitCode, new TextDecoder().decode(result.stderr)).toBe(0);
    expect(output(result)).toContain("run=run");
    expect(output(result)).toContain('artifact={"case":"case"');
    expect(output(result)).toContain('steps={"runId":"run"');
});
