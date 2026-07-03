pub fn responsive_columns(window_width: f32, breakpoints: &[(f32, usize)]) -> usize {
    breakpoints
        .iter()
        .filter(|(min_w, _)| *min_w <= window_width)
        .max_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal))
        .map(|(_, cols)| *cols)
        .unwrap_or(1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn below_all_breakpoints_returns_1() {
        assert_eq!(responsive_columns(500.0, &[(600.0, 2), (900.0, 3)]), 1);
    }

    #[test]
    fn matches_first_breakpoint() {
        assert_eq!(responsive_columns(700.0, &[(600.0, 2), (900.0, 3)]), 2);
    }

    #[test]
    fn matches_second_breakpoint() {
        assert_eq!(responsive_columns(1000.0, &[(600.0, 2), (900.0, 3)]), 3);
    }

    #[test]
    fn empty_breakpoints_returns_1() {
        assert_eq!(responsive_columns(700.0, &[]), 1);
    }

    #[test]
    fn unsorted_breakpoints_work() {
        assert_eq!(responsive_columns(1000.0, &[(900.0, 3), (600.0, 2)]), 3);
        assert_eq!(responsive_columns(700.0, &[(900.0, 3), (600.0, 2)]), 2);
    }
}
