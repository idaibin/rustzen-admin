export interface SseEvent {
    event: string;
    data: string;
    id?: string;
}

const MAX_LINE_BYTES = 8 * 1024;
const MAX_EVENT_BYTES = 16 * 1024;

export class SseParser {
    private readonly decoder = new TextDecoder("utf-8", { fatal: true });
    private readonly encode = new TextEncoder();
    private buffer = "";
    private event = "message";
    private data: string[] = [];
    private eventBytes = 0;
    private id: string | undefined;

    constructor(private readonly emit: (event: SseEvent) => void) {}

    push(bytes: Uint8Array): void {
        this.buffer += this.decoder.decode(bytes, { stream: true });
        this.consumeLines();
        if (this.encode.encode(this.buffer).byteLength > MAX_LINE_BYTES)
            throw new Error("SSE line exceeds 8 KiB");
    }

    finish(): void {
        this.buffer += this.decoder.decode();
        this.consumeLines();
        if (this.buffer.length) throw new Error("SSE ended with an unterminated line");
    }

    private consumeLines(): void {
        for (;;) {
            const end = this.buffer.indexOf("\n");
            if (end < 0) return;
            const wireLine = this.buffer.slice(0, end);
            let line = wireLine;
            this.buffer = this.buffer.slice(end + 1);
            if (line.endsWith("\r")) line = line.slice(0, -1);
            if (this.encode.encode(line).byteLength > MAX_LINE_BYTES)
                throw new Error("SSE line exceeds 8 KiB");
            this.consumeLine(line, this.encode.encode(wireLine).byteLength + 1);
        }
    }

    private consumeLine(line: string, wireBytes: number): void {
        if (!line) {
            if (this.data.length)
                this.emit({ event: this.event, data: this.data.join("\n"), id: this.id });
            this.event = "message";
            this.data = [];
            this.eventBytes = 0;
            this.id = undefined;
            return;
        }
        if (line.startsWith(":")) return;
        const separator = line.indexOf(":");
        const field = separator < 0 ? line : line.slice(0, separator);
        let value = separator < 0 ? "" : line.slice(separator + 1);
        if (["event", "id", "data"].includes(field)) {
            this.eventBytes += wireBytes;
            if (this.eventBytes > MAX_EVENT_BYTES) throw new Error("SSE event exceeds 16 KiB");
        }
        if (value.startsWith(" ")) value = value.slice(1);
        if (field === "event") this.event = value;
        if (field === "id" && !value.includes("\0")) this.id = value;
        if (field === "data") {
            this.data.push(value);
        }
    }
}
