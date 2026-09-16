export async function assertSelectedApiSource(apiPath: string, preset = "monitor") {
    assertSelectedApiText(await Bun.file(apiPath).text(), preset);
}

export function assertSelectedApiText(apiSource: string, preset = "monitor") {
    if (/\/(?:api)\/\s*["'`]\s*\+/.test(apiSource))
        throw new Error("selected Web API adapter dynamically constructs an API namespace");
    const excluded =
        preset === "analytics"
            ? /(?:monitor|notifications|reports|manage|system\/status)/
            : preset === "reports"
              ? /(?:monitor|insights|notifications|manage|system\/status)/
              : /(?:insights|reports|manage|system\/status)/;
    if (excluded.test(apiSource))
        throw new Error("selected Web API adapter names an excluded capability");
    if (preset === "analytics") assertAnalyticsReadContract(apiSource);
}

function assertAnalyticsReadContract(source: string) {
    const block = source.match(/const insightsAPI = \{([\s\S]*?)\n\};/)?.[1];
    if (!block || /\.\.\.|\[|\$\{|fetch\(/.test(block))
        throw new Error("Analytics selected API permits only direct named properties");
    const members = splitTopLevelMembers(block);
    const keys = members.map((member) => member.match(/^([a-zA-Z][a-zA-Z0-9]*)\s*:/)?.[1]).sort();
    const routes = [
        ...block.matchAll(/url:\s*["'`]([^"'`]+)["'`],[\s\S]*?method:\s*["'`]([^"'`]+)["'`]/g),
    ]
        .map((match) => `${match[2]} ${match[1]}`)
        .sort();
    const calls = block.match(/apiRequest(?:<|\s*\()/g)?.length ?? 0;
    const namespaceLiterals = source.match(/\/api\/insights/g)?.length ?? 0;
    if (
        calls !== 2 ||
        namespaceLiterals !== 2 ||
        members.length !== 2 ||
        JSON.stringify(keys) !== JSON.stringify(["events", "overview"]) ||
        JSON.stringify(routes) !==
            JSON.stringify(["GET /api/insights/events", "GET /api/insights/overview"])
    )
        throw new Error("Analytics selected API must contain only its two read routes");
}

function splitTopLevelMembers(block: string): string[] {
    const members: string[] = [];
    let start = 0;
    let braces = 0;
    let parentheses = 0;
    let brackets = 0;
    let angles = 0;
    let quote = "";
    let escaped = false;
    for (let index = 0; index < block.length; index += 1) {
        const character = block[index];
        if (quote) {
            if (escaped) escaped = false;
            else if (character === "\\") escaped = true;
            else if (character === quote) quote = "";
            continue;
        }
        if (character === '"' || character === "'") quote = character;
        else if (character === "{") braces += 1;
        else if (character === "}") braces -= 1;
        else if (character === "(") parentheses += 1;
        else if (character === ")") parentheses -= 1;
        else if (character === "[") brackets += 1;
        else if (character === "]") brackets -= 1;
        else if (character === "<") angles += 1;
        else if (character === ">" && angles > 0) angles -= 1;
        else if (
            character === "," &&
            braces === 0 &&
            parentheses === 0 &&
            brackets === 0 &&
            angles === 0
        ) {
            const member = block.slice(start, index).trim();
            if (member) members.push(member);
            start = index + 1;
        }
        if (braces < 0 || parentheses < 0 || brackets < 0)
            throw new Error("Analytics selected API object structure is invalid");
    }
    const tail = block.slice(start).trim();
    if (tail) members.push(tail);
    if (quote || braces || parentheses || brackets || angles)
        throw new Error("Analytics selected API object structure is invalid");
    return members;
}
