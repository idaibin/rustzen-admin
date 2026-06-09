import { deployAPI } from "./deploy/api";
import { dictAPI } from "./dict/api";
import { logAPI } from "./log/api";
import { taskAPI } from "./task/api";

export const manageAPI = {
    dict: dictAPI,
    log: logAPI,
    task: taskAPI,
    deploy: deployAPI,
};
