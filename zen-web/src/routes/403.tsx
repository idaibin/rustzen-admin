import { createFileRoute } from "@tanstack/react-router";
import { Result } from "antd";

export const Route = createFileRoute("/403")({
    component: RouteComponent,
});

function RouteComponent() {
    return (
        <Result
            status="403"
            title="403"
            subTitle="You do not have permission to access this page."
        />
    );
}
