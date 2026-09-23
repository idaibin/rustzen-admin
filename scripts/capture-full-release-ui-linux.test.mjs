import { describe, expect, test } from "bun:test";

const script = await Bun.file(new URL("./capture-full-release-ui-linux.sh", import.meta.url)).text();

describe("full preview runner isolation", () => {
    test("each run cleans only its own staging directory and uses helper output order", () => {
        expect(script).toContain("read -r image_id verifier_key provenance_sha");
        expect(script).toContain('rm -rf "$staged"');
        expect(script).not.toContain("-name '.bin-*'");
        expect(script).toContain('"$image_id" bash /verify/run.sh');
        expect(script).toContain("verifier-key.txt");
        expect(script).toContain("verifier-image-inspect-id.txt");
        expect(script).toContain('(.pages|length)==19');
        expect(script).toContain('(.screenshots|length)==19');
        expect(script).toContain("server build provenance does not match the current source tree");
        expect(script).toContain("Agent build provenance does not match the current source tree");
        expect(script).toContain("management-deployments.png");
    });
});
