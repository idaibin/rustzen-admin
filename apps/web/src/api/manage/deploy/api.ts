import { apiRequest, apiUpload } from "@/api/request";

export const deployAPI = {
    list: async (params?: Deploy.ListParams) => {
        const res = await apiRequest<Deploy.Item[], Deploy.ListParams>({
            url: "/api/manage/deploy/list",
            params,
            raw: true,
        });
        return {
            data: res.data,
            total: res.total ?? 0,
            success: true,
        };
    },
    upload: (data: Deploy.UploadForm) => {
        const formData = new FormData();
        formData.append("component", data.component);
        formData.append("version", data.version);
        if (data.arch) {
            formData.append("arch", data.arch);
        }
        if (data.notes) {
            formData.append("notes", data.notes);
        }
        formData.append("file", data.file);
        return apiUpload<Deploy.Item>("/api/manage/deploy/upload", formData);
    },
    detail: (id: number) => {
        return apiRequest<Deploy.Item>({
            url: `/api/manage/deploy/${id}`,
        });
    },
    deploy: (id: number, data: Deploy.DeployRequest) => {
        return apiRequest<boolean, Deploy.DeployRequest>({
            url: `/api/manage/deploy/${id}/deploy`,
            method: "POST",
            params: data,
        });
    },
    expire: (id: number, data: Deploy.ExpireRequest) => {
        return apiRequest<Deploy.Item, Deploy.ExpireRequest>({
            url: `/api/manage/deploy/${id}/expire`,
            method: "PUT",
            params: data,
        });
    },
    remove: (id: number) => {
        return apiRequest<Deploy.Item>({
            url: `/api/manage/deploy/${id}`,
            method: "DELETE",
        });
    },
    cleanup: (component?: Deploy.Component) => {
        const query = component ? `?component=${encodeURIComponent(component)}` : "";
        return apiRequest<number>({
            url: `/api/manage/deploy/cleanup${query}`,
            method: "POST",
        });
    },
};
