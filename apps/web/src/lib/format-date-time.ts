export function formatDateTime(value?: string | null, timeZone?: string): string {
    if (!value) {
        return "-";
    }

    const date = new Date(value);
    if (Number.isNaN(date.getTime())) {
        return "-";
    }

    try {
        const options: Intl.DateTimeFormatOptions = {
            year: "numeric",
            month: "2-digit",
            day: "2-digit",
            hour: "2-digit",
            minute: "2-digit",
            second: "2-digit",
            ...(timeZone ? { timeZone } : {}),
        };
        return new Intl.DateTimeFormat(getLocale(), options).format(date);
    } catch {
        return "-";
    }
}
import { getLocale } from "./i18n";
