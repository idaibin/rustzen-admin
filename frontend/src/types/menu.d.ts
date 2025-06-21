// frontend/src/types/menu.d.ts

declare module "Menu" {
  interface Item {
    id: number;
    parentId: number;
    name: string;
    path?: string | null;
    component?: string | null;
    icon?: string | null;
    type: number; // 0: Directory, 1: Menu, 2: Button
  }
}
