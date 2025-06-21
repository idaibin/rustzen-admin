// frontend/src/types/log.d.ts

declare module "Log" {
  interface Item {
    id: number;
    level: string;
    message: string;
    createdAt: string;
  }
}
