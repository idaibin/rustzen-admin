import {
    createElement,
    createContext,
    type ReactNode,
    useContext,
    useSyncExternalStore,
} from "react";

export type Locale = "zh-CN" | "en-US";

const LOCALE_STORAGE_KEY = "rustzen-admin-locale";

type LocaleListener = () => void;

const listeners = new Set<LocaleListener>();

const readLocale = () => {
    if (typeof window === "undefined") {
        return "zh-CN" as const;
    }
    return localStorage.getItem(LOCALE_STORAGE_KEY) === "en-US" ? "en-US" : "zh-CN";
};

let currentLocale: Locale = readLocale();

if (typeof document !== "undefined") {
    document.documentElement.lang = currentLocale;
}

export const getLocale = (): Locale => currentLocale;

const emitChange = () => {
    listeners.forEach((listener) => listener());
};

export const setLocale = (locale: Locale) => {
    if (typeof window !== "undefined") {
        localStorage.setItem(LOCALE_STORAGE_KEY, locale);
    }
    if (typeof document !== "undefined") {
        document.documentElement.lang = locale;
    }
    const didChange = currentLocale !== locale;
    currentLocale = locale;
    if (didChange) {
        emitChange();
    }
};

export const subscribeLocale = (listener: LocaleListener) => {
    listeners.add(listener);
    return () => void listeners.delete(listener);
};

export const t = (chinese: string, english: string): string =>
    getLocale() === "en-US" ? english : chinese;

const LocaleContext = createContext<Locale>("zh-CN");

export const LocaleProvider = ({ children }: { children: ReactNode }) => {
    const locale = useSyncExternalStore(subscribeLocale, getLocale, getLocale);
    return createElement(LocaleContext.Provider, { value: locale }, children);
};

export const useLocale = () => useContext(LocaleContext);
