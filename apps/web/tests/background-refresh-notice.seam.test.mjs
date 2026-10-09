import { describe, expect, test } from "bun:test";

import { createElement } from "react";
import { renderToStaticMarkup } from "react-dom/server";

import { BackgroundRefreshNotice } from "../src/components/feedback/background-refresh-notice";
import { setLocale } from "../src/lib/i18n";

describe("BackgroundRefreshNotice", () => {
    test("discloses stale data and provides a retry action", () => {
        setLocale("en-US");
        let retryCalls = 0;
        const notice = BackgroundRefreshNotice({
            updatedAt: new Date("2026-07-27T10:11:12Z").getTime(),
            onRetry: () => {
                retryCalls += 1;
            },
        });
        const html = renderToStaticMarkup(createElement(BackgroundRefreshNotice, notice.props));

        expect(html).toContain(
            "Background refresh failed. The last successfully loaded data remains visible.",
        );
        expect(html).toContain("Retry");
        expect(html).toContain('role="alert"');
        notice.props.action.props.onClick();
        expect(retryCalls).toBe(1);
    });
});
