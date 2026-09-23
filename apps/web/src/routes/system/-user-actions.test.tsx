import { expect, test } from "bun:test";

import { getUserActionItems } from "./-user-actions";

const user = { id: 1, username: "operator", status: 1 } as User.Item;

test("user action items follow the granted action permissions", () => {
    const keys = (status: boolean, password: boolean, remove: boolean) =>
        getUserActionItems(user, status, password, remove).map((item) => item?.key);

    expect(keys(false, false, false)).toEqual([]);
    expect(keys(true, false, true)).toEqual(["status", "delete"]);
    expect(keys(true, true, true)).toEqual(["status", "password", "delete"]);
});
