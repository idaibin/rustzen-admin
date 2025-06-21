declare module "Api" {
  export interface ApiResponse<T> {
    code: number;
    message: string;
    data: T;
    total?: number;
  }

  export interface BaseParams {
    current?: number;
    pageSize?: number;
    keyword?: string;
  }
}
