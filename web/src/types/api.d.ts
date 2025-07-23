declare module "Api" {
    type BaseType = string | number | boolean | undefined | null;
    type BaseRecord = Record<string, BaseType>;
    type BaseArray = BaseType[] | BaseRecord[];
    type BaseItem = BaseType | BaseArray | BaseRecord;

    // 基础响应类型
    export interface ApiResponse<T> {
        code: number;
        message: string;
        data: T;
        total?: number;
    }

    // 分页响应类型
    export interface PageResponse<T> {
        data: T[];
        total: number;
        success: boolean;
    }

    // 基础查询参数
    export interface BaseParams {
        current?: number;
        pageSize?: number;
        keyword?: string;
        [key: string]: BaseItem;
    }

    // 选项类型
    export interface OptionItem {
        label: string;
        value: string | number;
        [key: string]: unknown;
    }
}
