/// Keep `scroll` fixed while `selected` stays inside the viewport; adjust only
/// when the selection would move above or below the visible window.
pub fn ensure_visible(scroll: &mut usize, selected: usize, len: usize, height: usize) {
    if len == 0 || height == 0 {
        *scroll = 0;
        return;
    }

    if len <= height {
        *scroll = 0;
        return;
    }

    if selected < *scroll {
        *scroll = selected;
    } else if selected >= *scroll + height {
        *scroll = selected + 1 - height;
    }

    if *scroll + height > len {
        *scroll = len - height;
    }
}

pub fn visible_window(len: usize, scroll: usize, height: usize) -> (usize, usize) {
    if len == 0 || height == 0 {
        return (0, 0);
    }
    if len <= height {
        return (0, len);
    }

    let start = scroll.min(len - height);
    (start, start + height)
}
