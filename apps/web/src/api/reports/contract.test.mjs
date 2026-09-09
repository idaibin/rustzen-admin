import { expect, test } from "bun:test";
import { reportsAPIContract } from "./contract";

test("Reports exposes the protected notification delivery contract", () => {
    expect(reportsAPIContract.notificationDelivery).toEqual({ method: "GET", path: "/api/reports/notification-delivery" });
});
