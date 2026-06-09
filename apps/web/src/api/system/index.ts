import { menuAPI } from "./menu/api";
import { roleAPI } from "./role/api";
import { userAPI } from "./user/api";

export const systemAPI = {
    user: userAPI,
    role: roleAPI,
    menu: menuAPI,
};
