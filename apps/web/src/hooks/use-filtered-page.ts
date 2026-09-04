import { useState } from "react";

// Reset before query hooks render: never request a new filter on the old page.
export function useFilteredPage(filterKey: string) {
    const [state, setState] = useState({ filterKey, page: 1 });
    const changed = state.filterKey !== filterKey;
    if (changed) setState({ filterKey, page: 1 });
    const page = changed ? 1 : state.page;
    const setPage = (next: number) => setState({ filterKey, page: next });
    return [page, setPage] as const;
}
