#!/usr/bin/env bun

const path = process.argv[2];
if (!path) {
    console.error("usage: verify-admin-browser-verifier-dockerfile.mjs <Dockerfile>");
    process.exit(2);
}

const source = await Bun.file(path).text();
const requireMatches = (pattern, expected, label) => {
    const matches = [...source.matchAll(pattern)];
    if (matches.length !== expected) {
        throw new Error(`${label}: expected ${expected}, found ${matches.length}`);
    }
    return matches.map((match) => match.index);
};

const expectedSources = [
    '"deb [check-valid-until=no] http://snapshot.debian.org/archive/debian/${SNAPSHOT_TIMESTAMP} bullseye main" \\',
    '"deb [check-valid-until=no] http://snapshot.debian.org/archive/debian-security/${SNAPSHOT_TIMESTAMP} bullseye-security main" \\',
];
const activeSources = source
    .split("\n")
    .filter((line) => /\bdeb\s+/.test(line) && !/^\s*#/.test(line))
    .map((line) => line.trim());
if (activeSources.length !== expectedSources.length || activeSources.some((line, index) => line !== expectedSources[index])) {
    throw new Error("the only active Debian sources must be the fixed main and security snapshots");
}

const aptCalls = requireMatches(/\bapt-get\b/g, 2, "apt-get commands");
const update = requireMatches(/\bapt-get\b[^\n]*\bupdate\b/g, 1, "apt update")[0];
const install = requireMatches(/\bapt-get\b[^\n]*\binstall\b/g, 1, "apt install")[0];
if (!aptCalls.includes(update) || !aptCalls.includes(install)) {
    throw new Error("every apt-get command must be the single update or install");
}

const listClears = requireMatches(/rm -rf \/var\/lib\/apt\/lists\/\*/g, 2, "APT list cleanup");
const main = requireMatches(
    /^\s*"deb \[check-valid-until=no\] http:\/\/snapshot\.debian\.org\/archive\/debian\/\$\{SNAPSHOT_TIMESTAMP\} bullseye main"/gm,
    1,
    "fixed Debian snapshot source",
)[0];
const security = requireMatches(
    /^\s*"deb \[check-valid-until=no\] http:\/\/snapshot\.debian\.org\/archive\/debian-security\/\$\{SNAPSHOT_TIMESTAMP\} bullseye-security main"/gm,
    1,
    "fixed Debian security snapshot source",
)[0];
const sourceCleanup = requireMatches(
    /rm -f \/etc\/apt\/sources\.list\.d\/\*/g,
    1,
    "rolling source cleanup",
)[0];
const certificates = requireMatches(/^\s*ca-certificates \\$/gm, 1, "ca-certificates package")[0];

if (!(listClears[0] < main && main < security && security < sourceCleanup && sourceCleanup < update && update < install && install < certificates && certificates < listClears[1])) {
    throw new Error("APT snapshot setup, install, and cleanup are out of order");
}
console.log("Admin browser verifier Dockerfile policy passed");
