declare module "Api" {
    type BaseType = string | number | boolean | undefined | null;
    type BaseRecord = Record<string, BaseType>;
    type BaseArray = BaseType[] | BaseRecord[];
    type BaseItem = BaseType | BaseArray | BaseRecord;

    // Base response type
    export interface ApiResponse<T> {
        code: number;
        message: string;
        data: T;
        total?: number;
    }

    // Page response type
    export interface PageResponse<T> {
        data: T[];
        total: number;
        success: boolean;
    }

    // Base query params
    export interface BaseParams {
        current?: number;
        pageSize?: number;
        keyword?: string;
        [key: string]: BaseItem;
    }

    // Option type
    export interface OptionItem {
        label: string;
        value: string | number;
        [key: string]: unknown;
    }
}
