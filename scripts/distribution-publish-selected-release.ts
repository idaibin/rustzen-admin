import { readFile } from "node:fs/promises";
import { relative, resolve } from "node:path";
import { canonicalJson } from "../distribution/release-manifest-core.ts";
import { publishSelectedRelease } from "../distribution/release-publisher.ts";

const root = resolve(import.meta.dir, "..");
const args = Bun.argv.slice(2);
const value = (name: string) => args[args.indexOf(name) + 1];
const required = [
    "--selection",
    "--staging",
    "--manifest",
    "--private-key",
    "--public-key",
    "--key-id",
];
if (required.some((name) => !value(name)))
    throw new Error(`usage: ${required.join(" ")}`);
const path = (name: string) => {
    const resolved = resolve(root, value(name)!);
    if (relative(root, resolved).startsWith(".."))
        throw new Error(`${name} must be beneath repository root`);
    return resolved;
};
const pemPath = (name: string) => resolve(value(name)!);
const selection = await Bun.file(path("--selection")).json();
const staging = await Bun.file(path("--staging")).json();
const manifest = await Bun.file(path("--manifest")).json();
const result = await publishSelectedRelease({
    selection,
    staging,
    manifest,
    privateKey: await readFile(pemPath("--private-key"), "utf8"),
    trusted: {
        keyId: value("--key-id")!,
        publicKey: await readFile(pemPath("--public-key"), "utf8"),
    },
});
console.log(canonicalJson(result));
