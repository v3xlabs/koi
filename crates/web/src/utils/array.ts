export const moveItem = <T>(items: T[], from: number, to: number) => {
    if (from < 0 || from >= items.length) return items;

    const destination = Math.min(Math.max(to, 0), items.length - 1);

    if (from === destination) return items;

    const next = [...items];
    const [item] = next.splice(from, 1);

    next.splice(destination, 0, item);

    return next;
};
