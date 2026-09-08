const ELF_MAGIC = [0x7f, 0x45, 0x4c, 0x46];
const ELF64 = 2;
const LITTLE_ENDIAN = 1;
const ELF_VERSION = 1;
const ET_DYN = 3;
const EM_X86_64 = 62;
const PT_INTERP = 3;
const PT_LOAD = 1;
const PF_X = 1;

/** Parses only enough ELF metadata to prove the exported static-PIE traits. */
export function verifyMonitorExportElf(bytes: Uint8Array, binary: string): void {
    if (bytes.byteLength < 64) throw new Error("exported binary is not an ELF64 header");
    for (let index = 0; index < ELF_MAGIC.length; index++)
        if (bytes[index] !== ELF_MAGIC[index]) throw new Error("exported binary is not ELF");
    if (bytes[4] !== ELF64 || bytes[5] !== LITTLE_ENDIAN)
        throw new Error("exported binary must be ELF64 little-endian");
    if (bytes[6] !== ELF_VERSION)
        throw new Error("exported binary must have ELF EI_VERSION 1");
    const view = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
    if (view.getUint32(20, true) !== ELF_VERSION)
        throw new Error("exported binary must have ELF e_version 1");
    if (view.getUint16(52, true) !== 64 || view.getUint16(54, true) !== 56)
        throw new Error("exported binary has invalid ELF header sizes");
    if (view.getUint16(16, true) !== ET_DYN)
        throw new Error("exported binary must be ET_DYN static PIE");
    if (view.getUint16(18, true) !== EM_X86_64)
        throw new Error("exported binary must be EM_X86_64");
    const entryPoint = view.getBigUint64(24, true);
    const programOffset = number(view.getBigUint64(32, true), "ELF program header offset");
    const programSize = view.getUint16(54, true);
    const programCount = view.getUint16(56, true);
    if (programSize < 56 || programCount === 0)
        throw new Error("exported binary has invalid ELF program headers");
    let load = 0, executableEntry = false;
    const mapped: Array<{ offset: number; files: number }> = [];
    for (let index = 0; index < programCount; index++) {
        const offset = programOffset + index * programSize;
        if (offset < 0 || offset + 56 > bytes.byteLength)
            throw new Error("exported binary program headers exceed file bounds");
        if (view.getUint32(offset, true) === PT_INTERP)
            throw new Error("exported binary must not contain PT_INTERP");
        if (view.getUint32(offset, true) !== PT_LOAD) continue;
        load++;
        const fileOffset = number(view.getBigUint64(offset + 8, true), "ELF load file offset");
        const virtual = view.getBigUint64(offset + 16, true);
        const fileSize = number(view.getBigUint64(offset + 32, true), "ELF load file size");
        const memorySize = view.getBigUint64(offset + 40, true);
        if (memorySize < BigInt(fileSize)) throw new Error("ELF PT_LOAD memory size is smaller than file size");
        if (fileOffset + fileSize > bytes.byteLength) throw new Error("ELF PT_LOAD exceeds file bounds");
        mapped.push({ offset: fileOffset, files: fileSize });
        if ((view.getUint32(offset + 4, true) & PF_X) && entryPoint !== 0n && entryPoint >= virtual && entryPoint < virtual + memorySize) executableEntry = true;
    }
    if (!load) throw new Error("exported binary has no PT_LOAD segment");
    if (entryPoint === 0n || !executableEntry) throw new Error("ELF entry point is not in executable PT_LOAD segment");
    const marker = new TextEncoder().encode(
        `RUSTZEN_RELEASE_MARKER\nartifact=rz-bundle-member\nbinary=${binary}\n`,
    );
    const markerOffset = includes(bytes, marker);
    if (markerOffset < 0 || !mapped.some((segment) => markerOffset >= segment.offset && markerOffset + marker.length <= segment.offset + segment.files))
        throw new Error(`exported binary is missing release marker for ${binary}`);
}

function number(value: bigint, label: string): number {
    if (value > BigInt(Number.MAX_SAFE_INTEGER)) throw new Error(`${label} is too large`);
    return Number(value);
}

function includes(bytes: Uint8Array, marker: Uint8Array): number {
    return Buffer.from(bytes.buffer, bytes.byteOffset, bytes.byteLength).indexOf(
        Buffer.from(marker.buffer, marker.byteOffset, marker.byteLength),
    );
}
