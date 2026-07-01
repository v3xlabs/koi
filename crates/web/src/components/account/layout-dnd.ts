import type { Account, AccountLayout } from "#/api/account";
import { normalizeGroupId } from "#/api/account";

export type DragItem =
  | { type: "group"; group_identity: number; }
  | { type: "account"; account_identity: number; group_id?: number; };

export type Insertion =
  | { kind: "before-account"; account_identity: number; group_id?: number; }
  | { kind: "after-account"; account_identity: number; group_id?: number; }
  | { kind: "group-end"; group_id?: number; }
  | { kind: "before-group"; group_identity: number; };

const sortByOrder = <T extends { display_order: number; }>(items: T[]) =>
    items.toSorted((a, b) => a.display_order - b.display_order);

export const accountsForGroup = (accounts: Account[], group_id?: number) => {
    const normalizedGroupId = normalizeGroupId(group_id);

    return sortByOrder(accounts.filter(
        account => normalizeGroupId(account.group_id) === normalizedGroupId,
    ));
};

const parseGroupAttr = (value: string | null | undefined) => {
    if (!value || value === "ungrouped") return undefined;

    return normalizeGroupId(Number(value));
};

export const insertionKey = (insertion: Insertion | null) => {
    if (!insertion) return "";

    switch (insertion.kind) {
        case "before-account": {
            return `before:${insertion.account_identity}:${insertion.group_id ?? "ungrouped"}`;
        }
        case "after-account": {
            return `after:${insertion.account_identity}:${insertion.group_id ?? "ungrouped"}`;
        }
        case "group-end": {
            return `end:${insertion.group_id ?? "ungrouped"}`;
        }
        case "before-group": {
            return `group:${insertion.group_identity}`;
        }
    }
};

export const findInsertion = (x: number, y: number, drag: DragItem): Insertion | null => {
    const el = document.elementFromPoint(x, y);

    if (!el) return null;

    const accountRow = el.closest("[data-drop-account]");

    if (accountRow && drag.type === "account") {
        const account_identity = Number(accountRow.dataset.dropAccount);

        if (account_identity === drag.account_identity) return null;

        const rect = accountRow.getBoundingClientRect();
        const group_id = parseGroupAttr(accountRow.dataset.dropGroup);

        if (y < rect.top + rect.height / 2) {
            return { kind: "before-account", account_identity, group_id };
        }

        return { kind: "after-account", account_identity, group_id };
    }

    const groupHeader = el.closest("[data-drop-group-header]");

    if (groupHeader) {
        const headerAttr = groupHeader.dataset.dropGroupHeader;
        const isUngroupedHeader = headerAttr === "ungrouped";
        const group_identity = isUngroupedHeader
            ? undefined
            : normalizeGroupId(Number(headerAttr));

        if (drag.type === "group") {
            if (isUngroupedHeader || group_identity === drag.group_identity) return null;

            return { kind: "before-group", group_identity: group_identity! };
        }

        if (drag.type === "account") {
            return { kind: "group-end", group_id: group_identity };
        }
    }

    const groupZone = el.closest("[data-drop-group-zone]");

    if (groupZone && drag.type === "account") {
        return {
            kind: "group-end",
            group_id: parseGroupAttr(groupZone.dataset.dropGroupZone),
        };
    }

    return null;
};

export const reorderGroups = (
    layout: AccountLayout,
    sourceId: number,
    targetId: number,
): AccountLayout => {
    const ordered = sortByOrder([...layout.groups]);
    const sourceIndex = ordered.findIndex(group => group.group_identity === sourceId);
    const targetIndex = ordered.findIndex(group => group.group_identity === targetId);

    if (sourceIndex === -1 || targetIndex === -1 || sourceIndex === targetIndex) {
        return layout;
    }

    const [moved] = ordered.splice(sourceIndex, 1);

    ordered.splice(targetIndex, 0, moved);

    return {
        ...layout,
        groups: ordered.map((group, index) => ({ ...group, display_order: index })),
    };
};

const moveAccountToIndex = (
    layout: AccountLayout,
    accountId: number,
    group_id: number | undefined,
    index: number,
): AccountLayout => {
    const normalizedGroupId = normalizeGroupId(group_id);
    const moving = layout.accounts.find(account => account.account_identity === accountId);

    if (!moving) return layout;

    const others = layout.accounts.filter(account => account.account_identity !== accountId);
    const bucket = accountsForGroup(others, normalizedGroupId);
    const insertIndex = Math.min(Math.max(index, 0), bucket.length);
    const nextBucket = [...bucket];

    nextBucket.splice(insertIndex, 0, { ...moving, group_id: normalizedGroupId });

    const bucketIds = new Set(nextBucket.map(account => account.account_identity));

    return {
        ...layout,
        accounts: layout.accounts.map((account) => {
            if (!bucketIds.has(account.account_identity)) return account;

            const order = nextBucket.findIndex(item => item.account_identity === account.account_identity);

            return {
                ...account,
                group_id: normalizedGroupId,
                display_order: order,
            };
        }),
    };
};

export const applyInsertion = (
    layout: AccountLayout,
    drag: DragItem,
    insertion: Insertion,
): AccountLayout => {
    switch (insertion.kind) {
        case "before-group": {
            if (drag.type !== "group") return layout;

            return reorderGroups(layout, drag.group_identity, insertion.group_identity);
        }
        case "group-end": {
            if (drag.type !== "account") return layout;

            return moveAccountToGroupEnd(layout, drag.account_identity, insertion.group_id);
        }
        case "before-account": {
            if (drag.type !== "account") return layout;

            return moveAccountBefore(layout, drag.account_identity, insertion.account_identity, insertion.group_id);
        }
        case "after-account": {
            if (drag.type !== "account") return layout;

            return moveAccountAfter(layout, drag.account_identity, insertion.account_identity, insertion.group_id);
        }
    }
};

const moveAccountToGroupEnd = (
    layout: AccountLayout,
    accountId: number,
    group_id: number | undefined,
): AccountLayout => {
    const normalizedGroupId = normalizeGroupId(group_id);
    const rest = layout.accounts.filter(account => account.account_identity !== accountId);
    const bucketSize = accountsForGroup(rest, normalizedGroupId).length;

    return moveAccountToIndex(layout, accountId, group_id, bucketSize);
};

const moveAccountBefore = (
    layout: AccountLayout,
    sourceId: number,
    targetId: number,
    group_id: number | undefined,
): AccountLayout => {
    const normalizedGroupId = normalizeGroupId(group_id);
    const rest = layout.accounts.filter(account => account.account_identity !== sourceId);
    const bucket = accountsForGroup(rest, normalizedGroupId);
    const targetIndex = bucket.findIndex(account => account.account_identity === targetId);

    if (targetIndex === -1) return layout;

    return moveAccountToIndex(layout, sourceId, group_id, targetIndex);
};

const moveAccountAfter = (
    layout: AccountLayout,
    sourceId: number,
    targetId: number,
    group_id: number | undefined,
): AccountLayout => {
    const normalizedGroupId = normalizeGroupId(group_id);
    const rest = layout.accounts.filter(account => account.account_identity !== sourceId);
    const bucket = accountsForGroup(rest, normalizedGroupId);
    const targetIndex = bucket.findIndex(account => account.account_identity === targetId);

    if (targetIndex === -1) return layout;

    return moveAccountToIndex(layout, sourceId, group_id, targetIndex + 1);
};

export const isDraggingItem = (drag: DragItem | null, candidate: DragItem) => {
    if (!drag || drag.type !== candidate.type) return false;

    if (drag.type === "account" && candidate.type === "account") {
        return drag.account_identity === candidate.account_identity;
    }

    if (drag.type === "group" && candidate.type === "group") {
        return drag.group_identity === candidate.group_identity;
    }

    return false;
};

export const matchesInsertionAccount = (
    insertion: Insertion | null,
    account_identity: number,
    group_id: number | undefined,
    edge: "before" | "after",
) => {
    if (!insertion) return false;

    const normalizedGroupId = normalizeGroupId(group_id);

    if (edge === "before" && insertion.kind === "before-account") {
        return insertion.account_identity === account_identity
          && normalizeGroupId(insertion.group_id) === normalizedGroupId;
    }

    if (edge === "after" && insertion.kind === "after-account") {
        return insertion.account_identity === account_identity
          && normalizeGroupId(insertion.group_id) === normalizedGroupId;
    }

    return false;
};

export const matchesInsertionGroupEnd = (
    insertion: Insertion | null,
    group_id: number | undefined,
) => (
    insertion?.kind === "group-end"
    && normalizeGroupId(insertion.group_id) === normalizeGroupId(group_id)
);

export const matchesInsertionGroupBefore = (
    insertion: Insertion | null,
    group_identity: number,
) => (
    insertion?.kind === "before-group"
    && insertion.group_identity === group_identity
);

export const expandGroupForInsertion = (insertion: Insertion | null): number | "ungrouped" | null => {
    if (!insertion) return null;

    if (insertion.kind === "group-end") {
        return insertion.group_id ?? "ungrouped";
    }

    if (insertion.kind === "before-account" || insertion.kind === "after-account") {
        return insertion.group_id ?? "ungrouped";
    }

    return null;
};
