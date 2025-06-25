import { request } from "./api";
import type {
  LoginRequest,
  LoginResponse,
  RegisterRequest,
  RegisterResponse,
  UserInfo,
} from "Api";

/**
 * Authentication service for user auth operations
 */
export class AuthAPI {
  private static readonly BASE_URL = "/api/auth";

  /**
   * Login with username and password
   * Stores JWT token in localStorage on success
   */
  static async login(data: LoginRequest) {
    const response = await request.post<LoginResponse, LoginRequest>(
      `${this.BASE_URL}/login`,
      data
    );

    // Store token for subsequent requests
    if (response.code === 0 && response.data.token) {
      localStorage.setItem("token", response.data.token);
      console.log("Login successful, token stored");
    }

    return response;
  }

  /**
   * Register new user account
   * Auto-login after successful registration
   */
  static async register(data: RegisterRequest) {
    const response = await request.post<RegisterResponse, RegisterRequest>(
      `${this.BASE_URL}/register`,
      data
    );

    // Store token after registration
    if (response.code === 0 && response.data.token) {
      localStorage.setItem("token", response.data.token);
      console.log("Registration successful, auto-logged in");
    }

    return response;
  }

  /**
   * Logout user and clear cache
   * Cleans up local storage even if server call fails
   */
  static async logout() {
    try {
      // Notify server to clear cache
      await request.post<void>(`${this.BASE_URL}/logout`);
      console.log("Server logout completed");
    } catch (error) {
      console.warn("Server logout failed, continuing cleanup:", error);
    } finally {
      // Always clean up locally
      localStorage.removeItem("token");
      localStorage.removeItem("userInfo");
      console.log("Local auth data cleared");
    }
  }

  /**
   * Get current user info with roles and menus
   * Caches result in localStorage
   */
  static async getUserInfo() {
    const response = await request.get<UserInfo>(`${this.BASE_URL}/me`);

    // Cache user info locally
    if (response.code === 0 && response.data) {
      localStorage.setItem("userInfo", JSON.stringify(response.data));
      console.log("User info cached");
    }

    return response;
  }

  /**
   * Check if user is logged in (token exists)
   * Note: Doesn't validate token - just checks presence
   */
  static isLoggedIn(): boolean {
    return !!localStorage.getItem("token");
  }

  /**
   * Get stored JWT token
   */
  static getToken(): string | null {
    return localStorage.getItem("token");
  }

  /**
   * Get cached user info from localStorage
   * Returns null if no cache or parse error
   */
  static getCachedUserInfo(): UserInfo | null {
    const userInfo = localStorage.getItem("userInfo");
    try {
      return userInfo ? JSON.parse(userInfo) : null;
    } catch (error) {
      console.error("Failed to parse cached user info:", error);
      localStorage.removeItem("userInfo");
      return null;
    }
  }
}
