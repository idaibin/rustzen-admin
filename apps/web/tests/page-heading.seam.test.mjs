import { describe, expect, test } from "bun:test";

import { createElement } from "react";
import { renderToStaticMarkup } from "react-dom/server";

import { PageCard } from "../src/components/page/page-card";
import { PageHeader } from "../src/components/page/page-header";

const headingCount = (html, level, title) =>
    (html.match(new RegExp(`<h${level}[^>]*>${title}</h${level}>`, "g")) ?? []).length;

describe("shared page headings", () => {
    test("PageHeader renders one default page h1", () => {
        const html = renderToStaticMarkup(createElement(PageHeader, { title: "用户列表" }));
        expect(headingCount(html, 1, "用户列表")).toBe(1);
        expect(headingCount(html, 2, "用户列表")).toBe(0);
    });
    test("PageCard passes its requested section level through exactly once", () => {
        const html = renderToStaticMarkup(
            createElement(PageCard, { title: "账号信息", headingLevel: 2 }, "内容"),
        );
        expect(headingCount(html, 2, "账号信息")).toBe(1);
        expect(headingCount(html, 1, "账号信息")).toBe(0);
    });
});
