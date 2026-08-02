export interface MergeGroupOrderOptions {
    preserveUnknown?: boolean;
}

export const mergeGroupOrder = (
    existing: string[],
    childrenIds: string[],
    options: MergeGroupOrderOptions = {},
) => {
    const valid = new Set(childrenIds);
    const retained = options.preserveUnknown
        ? Array.from(new Set(existing))
        : existing.filter((id) => valid.has(id));
    const retainedSet = new Set(retained);
    const missing = childrenIds
        .filter((id) => !retainedSet.has(id))
        .sort((a, b) => a.localeCompare(b));
    const order = [...retained, ...missing];
    const changed = existing.length !== order.length || existing.some((id, index) => id !== order[index]);
    return { order, changed };
};
