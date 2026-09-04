import { dlopen, FFIType, read } from "bun:ffi";

const AT_FDCWD = -100;
const RENAME_NOREPLACE = 1;
const RENAME_EXCL = 0x4;
const EEXIST = 17;
let beforeCallHook: (() => void) | undefined;
export const setAtomicRenameBeforeCallHookForTest = (hook?: () => void) => {
    beforeCallHook = hook;
};

export function atomicRenameNoReplace(
    source: string,
    destination: string,
): void {
    if (process.platform === "linux") return renameLinux(source, destination);
    if (process.platform === "darwin") return renameDarwin(source, destination);
    throw new Error("atomic no-replace rename is unsupported on this platform");
}
function renameLinux(source: string, destination: string) {
    const library = dlopen("libc.so.6", {
        renameat2: {
            args: [
                FFIType.i32,
                FFIType.cstring,
                FFIType.i32,
                FFIType.cstring,
                FFIType.u32,
            ],
            returns: FFIType.i32,
        },
        __errno_location: { args: [], returns: FFIType.ptr },
    });
    try {
        beforeCallHook?.();
        check(
            library.symbols.renameat2(
                AT_FDCWD,
                cstring(source),
                AT_FDCWD,
                cstring(destination),
                RENAME_NOREPLACE,
            ),
            read.i32(library.symbols.__errno_location()),
        );
    } finally {
        library.close();
    }
}
function renameDarwin(source: string, destination: string) {
    const library = dlopen("/usr/lib/libSystem.B.dylib", {
        renamex_np: {
            args: [FFIType.cstring, FFIType.cstring, FFIType.u32],
            returns: FFIType.i32,
        },
        __error: { args: [], returns: FFIType.ptr },
    });
    try {
        beforeCallHook?.();
        check(
            library.symbols.renamex_np(
                cstring(source),
                cstring(destination),
                RENAME_EXCL,
            ),
            read.i32(library.symbols.__error()),
        );
    } finally {
        library.close();
    }
}
function check(result: number, errno: number) {
    if (result === 0) return;
    if (errno === EEXIST)
        throw new Error("atomic release final output already exists");
    throw new Error(`atomic release rename failed with errno ${errno}`);
}
function cstring(value: string): Buffer {
    if (value.includes("\0"))
        throw new Error("native rename path contains NUL");
    return Buffer.from(`${value}\0`);
}
