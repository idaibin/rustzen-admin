import { createFileRoute } from "@tanstack/react-router";

import { useLocale } from "@/lib/i18n";

import { ModuleLogDiagnostics } from "./-module-log-diagnostics";

export const Route = createFileRoute("/system/module-log")({
    component: ModuleLogPage,
});

function ModuleLogPage() {
    useLocale();
    return <ModuleLogDiagnostics />;
}
