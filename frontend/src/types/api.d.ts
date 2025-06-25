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

  // 认证相关接口
  export interface LoginRequest {
    username: string;
    password: string;
  }

  export interface LoginResponse {
    token: string;
  }

  export interface RegisterRequest {
    username: string;
    email: string;
    password: string;
  }

  export interface RegisterResponse {
    user: {
      id: number;
      username: string;
    };
    token: string;
  }

  export interface UserInfo {
    id: number;
    username: string;
    real_name?: string;
    avatar_url?: string;
    roles: Array<{
      id: number;
      role_name: string;
      description?: string;
    }>;
    menus: Array<{
      id: number;
      name: string;
      path?: string;
      icon?: string;
      sort_order: number;
      parent_id?: number;
      children?: Array<any>;
    }>;
  }
}
