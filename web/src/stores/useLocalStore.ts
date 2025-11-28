import { create } from 'zustand';
import { persist } from 'zustand/middleware';

type LocalStoreState = Record<string, string> & {
  update: (key: string, value: string) => void;
};

const useStore = create<LocalStoreState>()(
  persist(
    (set) =>
      ({
        update: (key, value) => {
          set((state) => ({
            ...state,
            [key]: value,
          }));
        },
      }) as LocalStoreState,
    {
      name: 'local-store',
    },
  ),
);

export const useLocalStore = (storeKey: string, defaultValue: string = ''): [string, (val: string) => void] => {
  const value = useStore((state) => state[storeKey] ?? defaultValue);
  const update = useStore((state) => state.update);

  return [value, (val: string) => update(storeKey, val)];
};
