declare namespace Api {
  type BaseType = string | number | boolean | undefined | null;
  type BaseRecord = Record<string, BaseType>;
  type BaseArray = BaseType[] | BaseRecord[];
  type BaseItem = BaseType | BaseArray | BaseRecord;

  // Base response type
  interface ApiResponse<T> {
    code: number;
    message: string;
    data: T;
    total?: number;
  }

  // Page response type
  interface PageResponse<T> {
    data: T[];
    total: number;
    success: boolean;
  }

  // Base query params
  interface BaseParams {
    current?: number;
    pageSize?: number;
    keyword?: string;
    [key: string]: BaseItem;
  }

  // Option type
  interface OptionItem {
    label: string;
    value: string | number;
    [key: string]: unknown;
  }
}
