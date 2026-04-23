import { createFileRoute } from "@tanstack/react-router";
import { Result } from "antd";

export const Route = createFileRoute("/404")({
    component: RouteComponent,
});

function RouteComponent() {
    return (
        <Result status="404" title="404" subTitle="The page you are looking for does not exist." />
    );
}
