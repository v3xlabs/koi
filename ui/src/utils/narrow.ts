import { Accessor } from "solid-js";

export const narrow = <A, B extends A>(accessor: Accessor<A | undefined>, guard: (v: A) => v is B): B | null => {
    const val = accessor();

    if (!val) {
        return null;
    }

    if (guard(val)) {
        return val;
    }

    return null;
};
