import { describe, expect, test } from "bun:test";

import { createElement } from "react";
import { renderToStaticMarkup } from "react-dom/server";

import { BackgroundRefreshNotice } from "../src/components/feedback/background-refresh-notice";

describe("BackgroundRefreshNotice", () => {
    test("discloses stale data and provides a retry action", () => {
        let retryCalls = 0;
        const notice = BackgroundRefreshNotice({
            updatedAt: new Date("2026-07-27T10:11:12Z").getTime(),
            onRetry: () => {
                retryCalls += 1;
            },
        });
        const html = renderToStaticMarkup(createElement(BackgroundRefreshNotice, notice.props));

        expect(html).toContain("后台刷新失败，当前继续显示上次成功数据。");
        expect(html).toContain("重试");
        expect(html).toContain('role="alert"');
        notice.props.action.props.onClick();
        expect(retryCalls).toBe(1);
    });
});
