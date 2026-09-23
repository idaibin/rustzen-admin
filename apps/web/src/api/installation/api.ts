import { apiRequest } from "@/api/request";

export const installationAPI = {
    get: () => apiRequest<Installation.Info>({ url: "/api/installation" }),
};
