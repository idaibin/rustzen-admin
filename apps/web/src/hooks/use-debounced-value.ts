import { useEffect, useState } from "react";

export function useDebouncedValue<T>(value: T, delay: number, enabled = true) {
    const [debouncedValue, setDebouncedValue] = useState(value);

    useEffect(() => {
        if (!enabled) return;
        const timer = setTimeout(() => setDebouncedValue(value), delay);
        return () => clearTimeout(timer);
    }, [delay, enabled, value]);

    // Clearing a search is immediate, but composition must retain the applied value.
    if (enabled && value === "" && debouncedValue !== value) setDebouncedValue(value);
    return enabled && value === "" ? value : debouncedValue;
}
