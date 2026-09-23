#!/usr/bin/env node
import crypto from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

const MARKER_BEGIN = Buffer.from("\nRUSTZEN_BUNDLE_SIGNED_MARKER_BEGIN\n");
const MARKER_END = Buffer.from("\nRUSTZEN_BUNDLE_SIGNED_MARKER_END\n");
const PAYLOAD_VERSION = "rustzen-release-v2";
const BINARIES = ["rz", "rz-admin", "rz-monitor", "rz-insights", "rz-reports"];
const PROJECT_ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const DEFAULT_KEY_FILE = path.join(
    PROJECT_ROOT,
    ".rustzen-admin",
    "config",
    "deploy-sign-private.pem",
);
const ICLOUD_SECRET_DIR = path.join(
    os.homedir(),
    "Library",
    "Mobile Documents",
    "com~apple~CloudDocs",
    "rustzen",
    "secrets",
);
const ICLOUD_KEY_FILES = [
    "rustzen-admin-deploy-sign.key",
    "rustzen-admin-deploy-sign.pem",
    "rustzen-release-sign.key",
];

const command = process.argv[2];
const args = parseArgs(process.argv.slice(3));

try {
    if (command === "ensure-key") {
        const keyFile = resolvePrivateKeyFile();
        console.log(keyFile);
    } else if (command === "public-key") {
        process.stdout.write(`${publicKeyHex(loadPrivateKey())}\n`);
    } else if (command === "sign-bundle") {
        signBundle();
    } else if (command === "verify-bundle") {
        verifyBundle();
    } else {
        usage();
        process.exit(1);
    }
} catch (error) {
    console.error(error instanceof Error ? error.message : error);
    process.exit(1);
}

function signBundle() {
    const file = requiredArg("file");
    const version = requiredArg("version");
    const arch = requiredArg("arch");
    const privateKey = loadPrivateKey();

    const original = fs.readFileSync(file);
    const content = stripBundleMarker(original);
    const scopes = releaseScopes(content, version, arch);
    const contentSha256 = sha256Hex(content);
    const marker = signedMarker({
        component: "release",
        version,
        arch,
        contentSha256,
        ...scopes,
        privateKey,
    });
    const mode = fs.statSync(file).mode;

    fs.writeFileSync(file, Buffer.concat([content, MARKER_BEGIN, Buffer.from(marker), MARKER_END]));
    fs.chmodSync(file, mode);
    console.log(`Signed release bundle: ${file}`);
}

function verifyBundle() {
    const file = requiredArg("file");
    const version = requiredArg("version");
    const arch = requiredArg("arch");
    const data = fs.readFileSync(file);
    const begin = data.lastIndexOf(MARKER_BEGIN);
    if (begin < 0) {
        throw new Error("Signed release bundle marker is missing.");
    }
    const content = data.subarray(0, begin);
    const scopes = releaseScopes(content, version, arch);
    const markerStart = begin + MARKER_BEGIN.length;
    const markerEnd = data.indexOf(MARKER_END, markerStart);
    if (markerEnd < 0 || markerEnd + MARKER_END.length !== data.length) {
        throw new Error("Signed release bundle marker is invalid.");
    }
    const marker = JSON.parse(data.subarray(markerStart, markerEnd).toString("utf8"));
    const contentSha256 = sha256Hex(content);
    if (
        marker.schemaVersion !== 2 ||
        marker.component !== "release" ||
        marker.version !== version ||
        marker.arch !== arch ||
        marker.contentSha256 !== contentSha256 ||
        marker.frontendSha256 !== scopes.frontendSha256 ||
        marker.backendSha256 !== scopes.backendSha256
    ) {
        throw new Error("Signed release metadata does not match the artifact.");
    }
    const payload = signaturePayload({
        component: "release",
        version,
        arch,
        contentSha256,
        ...scopes,
    });
    const valid = crypto.verify(
        null,
        Buffer.from(payload),
        crypto.createPublicKey(loadPrivateKey()),
        Buffer.from(marker.signature, "hex"),
    );
    if (!valid) {
        throw new Error("Signed release signature verification failed.");
    }
    console.log(`Verified release bundle: ${file}`);
}

function signedMarker({
    component,
    version,
    arch,
    contentSha256,
    frontendSha256,
    backendSha256,
    privateKey,
}) {
    const payload = signaturePayload({
        component,
        version,
        arch,
        contentSha256,
        frontendSha256,
        backendSha256,
    });
    const signature = crypto.sign(null, Buffer.from(payload), privateKey).toString("hex");
    const marker = {
        schemaVersion: 2,
        component,
        version,
        arch,
        contentSha256,
        frontendSha256,
        backendSha256,
        signature,
    };
    return JSON.stringify(marker);
}

function signaturePayload({
    component,
    version,
    arch,
    contentSha256,
    frontendSha256,
    backendSha256,
}) {
    return `${PAYLOAD_VERSION}\ncomponent=${component}\nversion=${version}\narch=${arch}\ncontent_sha256=${contentSha256}\nfrontend_sha256=${frontendSha256}\nbackend_sha256=${backendSha256}\n`;
}

function releaseScopes(content, version, arch) {
    const members = readTarFiles(content);
    const root = `rz-${version}-${arch}`;
    const binaries = BINARIES.map((name) => {
        const data = members.get(`${root}/bin/${name}`);
        if (!data) {
            throw new Error(`Release bundle is missing binary: ${name}`);
        }
        validateReleaseBinary(data, name, version, arch);
        return { name, data, sha256: sha256Hex(data) };
    });
    const admin = binaries.find(({ name }) => name === "rz-admin").data;
    const markerPrefix = Buffer.from(
        `RUSTZEN_RELEASE_MARKER\nartifact=rz-bundle-member\nbinary=rz-admin\nversion=${version}\nfrontend_sha256=`,
    );
    const markerStart = admin.indexOf(markerPrefix);
    if (markerStart < 0 || admin.indexOf(markerPrefix, markerStart + 1) >= 0) {
        throw new Error("Admin release identity marker is missing or duplicated.");
    }
    const valueStart = markerStart + markerPrefix.length;
    const frontendSha256 = admin.subarray(valueStart, valueStart + 64).toString("ascii");
    if (!/^[0-9a-f]{64}$/.test(frontendSha256) || admin[valueStart + 64] !== 10) {
        throw new Error("Admin frontend digest marker is invalid.");
    }
    const backendInventory = binaries
        .map(
            ({ name, data, sha256 }) =>
                `binary=${name}\nsize=${data.length}\nsha256=${sha256}\n`,
        )
        .join("");
    return { frontendSha256, backendSha256: sha256Hex(Buffer.from(backendInventory)) };
}

function validateReleaseBinary(data, name, version, arch) {
    if (data.length < 20 || !data.subarray(0, 4).equals(Buffer.from([0x7f, 0x45, 0x4c, 0x46]))) {
        throw new Error(`Release bundle member is not an ELF executable: ${name}`);
    }
    const expectedMachine = arch === "x86_64" ? 62 : arch === "aarch64" ? 183 : null;
    if (expectedMachine === null || data.readUInt16LE(18) !== expectedMachine) {
        throw new Error(`Release bundle member architecture mismatch: ${name}`);
    }
    const marker = Buffer.from(
        `RUSTZEN_RELEASE_MARKER\nartifact=rz-bundle-member\nbinary=${name}\nversion=${version}\n`,
    );
    if (data.indexOf(marker) < 0) {
        throw new Error(`Release bundle member identity marker mismatch: ${name}`);
    }
}

function readTarFiles(content) {
    if (content.length === 0 || content.length % 512 !== 0) {
        throw new Error("Release bundle is not an aligned tar archive.");
    }
    const files = new Map();
    let offset = 0;
    while (offset + 512 <= content.length) {
        const header = content.subarray(offset, offset + 512);
        if (header.every((byte) => byte === 0)) {
            break;
        }
        const name = tarText(header.subarray(0, 100));
        const prefix = tarText(header.subarray(345, 500));
        const pathName = prefix ? `${prefix}/${name}` : name;
        const sizeText = tarText(header.subarray(124, 136)).trim();
        if (!/^[0-7]+$/.test(sizeText)) {
            throw new Error("Release bundle contains an invalid tar size.");
        }
        const size = Number.parseInt(sizeText, 8);
        if (!Number.isSafeInteger(size) || size < 0) {
            throw new Error("Release bundle contains an unsupported tar size.");
        }
        const dataStart = offset + 512;
        const dataEnd = dataStart + size;
        if (dataEnd > content.length) {
            throw new Error("Release bundle contains a truncated tar member.");
        }
        const type = header[156];
        if (type === 0 || type === 48) {
            if (files.has(pathName)) {
                throw new Error(`Release bundle contains a duplicate file: ${pathName}`);
            }
            files.set(pathName, content.subarray(dataStart, dataEnd));
        }
        offset = dataStart + Math.ceil(size / 512) * 512;
    }
    return files;
}

function tarText(data) {
    const end = data.indexOf(0);
    return data.subarray(0, end < 0 ? data.length : end).toString("utf8");
}

function stripBundleMarker(data) {
    const begin = data.lastIndexOf(MARKER_BEGIN);
    if (begin === -1) {
        return data;
    }
    const end = data.indexOf(MARKER_END, begin + MARKER_BEGIN.length);
    if (end === -1) {
        return data;
    }
    const after = end + MARKER_END.length;
    if (after !== data.length) {
        return data;
    }
    return data.subarray(0, begin);
}

function loadPrivateKey() {
    const keyFile = process.env.RUSTZEN_DEPLOY_SIGN_KEY_FILE;
    const key = process.env.RUSTZEN_DEPLOY_SIGN_KEY;
    if (keyFile) {
        return crypto.createPrivateKey(fs.readFileSync(keyFile, "utf8"));
    }
    if (key) {
        return crypto.createPrivateKey(key.replaceAll("\\n", "\n"));
    }
    return crypto.createPrivateKey(fs.readFileSync(resolvePrivateKeyFile(), "utf8"));
}

function resolvePrivateKeyFile() {
    const secretsDir = process.env.RUSTZEN_SECRETS_DIR || ICLOUD_SECRET_DIR;
    for (const fileName of ICLOUD_KEY_FILES) {
        const keyFile = path.join(secretsDir, fileName);
        if (fs.existsSync(keyFile)) {
            return keyFile;
        }
    }
    if (fs.existsSync(DEFAULT_KEY_FILE)) {
        return DEFAULT_KEY_FILE;
    }
    throw new Error(
        "Deploy signing key is unavailable. Set RUSTZEN_DEPLOY_SIGN_KEY or " +
            "RUSTZEN_DEPLOY_SIGN_KEY_FILE, or add the existing key under iCloud rustzen/secrets.",
    );
}

function publicKeyHex(privateKey) {
    const der = crypto.createPublicKey(privateKey).export({ format: "der", type: "spki" });
    const ed25519SpkiPrefix = Buffer.from("302a300506032b6570032100", "hex");
    if (der.length !== ed25519SpkiPrefix.length + 32 || !der.subarray(0, ed25519SpkiPrefix.length).equals(ed25519SpkiPrefix)) {
        throw new Error("Deploy signing key must be an Ed25519 private key.");
    }
    return der.subarray(ed25519SpkiPrefix.length).toString("hex");
}

function sha256Hex(data) {
    return crypto.createHash("sha256").update(data).digest("hex");
}

function requiredArg(name) {
    const value = args.get(name);
    if (!value) {
        throw new Error(`Missing --${name}`);
    }
    return value;
}

function parseArgs(values) {
    const result = new Map();
    for (let index = 0; index < values.length; index += 1) {
        const item = values[index];
        if (!item.startsWith("--")) {
            throw new Error(`Unexpected argument: ${item}`);
        }
        const key = item.slice(2);
        const value = values[index + 1];
        if (!value || value.startsWith("--")) {
            throw new Error(`Missing value for ${item}`);
        }
        result.set(key, value);
        index += 1;
    }
    return result;
}

function usage() {
    console.error(`Usage:
  node scripts/deploy-sign.mjs ensure-key
  node scripts/deploy-sign.mjs public-key
  bun scripts/deploy-sign.mjs sign-bundle --file <tar> --version <version> --arch <x86_64|aarch64>
  bun scripts/deploy-sign.mjs verify-bundle --file <tar> --version <version> --arch <x86_64|aarch64>

Environment:
  RUSTZEN_DEPLOY_SIGN_KEY_FILE=/path/to/ed25519-private.pem
  # or
  RUSTZEN_DEPLOY_SIGN_KEY='-----BEGIN PRIVATE KEY-----\\n...'
  # optional iCloud secrets directory override
  RUSTZEN_SECRETS_DIR=/path/to/rustzen/secrets

Lookup order:
  environment key file -> environment key -> iCloud rustzen/secrets -> existing local key
`);
}
