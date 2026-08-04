import { readFile, writeFile } from "node:fs/promises";
import { resolve } from "node:path";

const generatedPath = resolve(
    process.env.CONTRACT_CLIENT_OUTPUT ??
        new URL("../src/api/generated/admin-contract.ts", import.meta.url).pathname,
);
let source = await readFile(generatedPath, "utf8");

const avatarType = /export interface AvatarUpload \{\s*file: Blob;\s*\}/;
if (!avatarType.test(source)) {
    throw new Error(
        "Orval AvatarUpload shape changed; review the filename-bearing client contract.",
    );
}
source = source.replace(avatarType, "export interface AvatarUpload {\n  file: File;\n}");

const avatarAppend = /formData\.append\(`file`, avatarUpload\.file\);/;
if (!avatarAppend.test(source)) {
    throw new Error("Orval avatar multipart append changed; review filename preservation.");
}
source = source.replace(
    avatarAppend,
    "formData.append(`file`, avatarUpload.file, avatarUpload.file.name);",
);

await writeFile(generatedPath, source);
