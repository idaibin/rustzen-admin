/** Bounded, generated client for the Admin route-contract pilot. */
export default {
    adminContract: {
        input: "../../openapi/admin-contract.json",
        output: {
            client: "fetch",
            clean: true,
            mode: "single",
            target: "src/api/generated/admin-contract.ts",
            override: {
                fetch: { includeHttpResponseReturnType: false },
                mutator: { path: "./src/api/request.ts", name: "generatedApiRequest" },
            },
        },
    },
};
