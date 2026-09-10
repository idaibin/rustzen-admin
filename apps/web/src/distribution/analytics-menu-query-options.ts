import { queryOptions } from "@tanstack/react-query";

import { apiRequest } from "@/api/request";
import { t } from "@/lib/i18n";

const options = () =>
    apiRequest<Menu.OptionItem[]>({ url: "/api/system/menus/options" }).then((items) => [
        {
            label: t("根目录", "Root"),
            value: 0,
            code: "",
            isSystem: true,
            moduleId: null,
            moduleMenuCode: null,
        },
        ...items,
    ]);

export const menuQueryOptions = {
    options: () => queryOptions({ queryKey: ["system", "menus", "options"], queryFn: options }),
};
