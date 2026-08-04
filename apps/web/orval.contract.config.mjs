/** Bounded, generated client for the Admin route-contract pilot. */
const contractInput = process.env.CONTRACT_OPENAPI_INPUT ?? "../../openapi/admin-contract.json";
const contractOutput = process.env.CONTRACT_CLIENT_OUTPUT ?? "src/api/generated/admin-contract.ts";
const mutatorPath = process.env.CONTRACT_MUTATOR_PATH ?? "./src/api/request.ts";

export default {
    adminContract: {
        input: contractInput,
        output: {
            client: "fetch",
            clean: true,
            mode: "single",
            target: contractOutput,
            override: {
                fetch: { includeHttpResponseReturnType: false },
                mutator: { path: mutatorPath, name: "generatedApiRequest" },
            },
        },
    },
};
