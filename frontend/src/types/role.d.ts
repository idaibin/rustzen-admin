// frontend/src/types/role.d.ts

declare module "Role" {
  interface Item {
    id: number;
    roleName: string;
    roleCode: string;
    remark?: string | null;
  }
}
