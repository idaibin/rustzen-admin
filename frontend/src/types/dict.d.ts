// frontend/src/types/dict.d.ts

declare module "Dict" {
  interface Item {
    id: number;
    dictType: string;
    label: string;
    value: string;
    isDefault: boolean;
  }
}
