import { dictAPI } from "./dict/api";
import { logAPI } from "./log/api";
import { menuAPI } from "./menu/api";
import { roleAPI } from "./role/api";
import { userAPI } from "./user/api";

export const systemAPI = {
    user: userAPI,
    role: roleAPI,
    menu: menuAPI,
    dict: dictAPI,
    log: logAPI,
};
