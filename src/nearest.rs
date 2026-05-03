//! Palette nearest-color search, mirroring culori 4.0.2's `nearest.js`.
//!
//! ```js
//! const nearest = (colors, metric = differenceEuclidean(), accessor = d => d) => {
//!     let arr = colors.map((c, idx) => ({ color: accessor(c), i: idx }));
//!     return (color, n = 1, τ = Infinity) => {
//!         if (isFinite(n)) {
//!             n = Math.max(1, Math.min(n, arr.length - 1));
//!         }
//!         arr.forEach(c => { c.d = metric(color, c.color); });
//!         return arr.sort((a, b) => a.d - b.d).slice(0, n).filter(c => c.d < τ).map(c => colors[c.i]);
//!     };
//! };
//! ```
//!
//! The Rust translation: `nearest(palette, metric)` returns a closure
//! `Fn(&Color, usize) -> Vec<Color>` that ranks palette entries by their
//! distance under `metric` (defaulting to Euclidean in RGB) and slices off
//! the closest `n`. We use `usize::MAX` as the "Infinity" sentinel — pass
//! it to receive every sorted color.

use crate::difference::difference_euclidean;
use crate::Color;

type Metric = Box<dyn Fn(&Color, &Color) -> f64>;

/// Returns a closure that finds the `n` nearest colors in `palette` to a
/// query color, ordered by ascending distance under `metric`.
///
/// Pass `metric = None` to use the default Euclidean distance over `rgb`,
/// matching culori's default for `nearest(colors)`.
///
/// `n` is clamped to `palette.len() - 1` when finite, matching culori's
/// `Math.max(1, Math.min(n, arr.length - 1))`. To get every color sorted
/// by distance, pass `n = usize::MAX`. Returns an empty vector when the
/// palette is empty.
///
/// ```rust
/// use culors::{nearest, parse};
///
/// let palette: Vec<_> = ["red", "green", "blue"]
///     .iter()
///     .map(|s| parse(s).unwrap())
///     .collect();
/// let find = nearest(palette, None);
/// let target = parse("#fa0000").unwrap();
/// let hits = find(&target, 1);
/// assert_eq!(hits.len(), 1);
/// ```
pub fn nearest(
    palette: Vec<Color>,
    metric: Option<Metric>,
) -> impl Fn(&Color, usize) -> Vec<Color> {
    let metric: Metric = metric.unwrap_or_else(|| Box::new(difference_euclidean("rgb")));
    move |query, n| {
        let len = palette.len();
        if len == 0 {
            return Vec::new();
        }
        // Clamp finite `n` the same way culori does.
        let take = if n == usize::MAX {
            len
        } else {
            n.clamp(1, len.saturating_sub(1).max(1))
        };
        let mut indexed: Vec<(usize, f64)> = palette
            .iter()
            .enumerate()
            .map(|(i, c)| (i, metric(query, c)))
            .collect();
        // Stable sort to preserve insertion order on ties (matches V8's
        // Array.prototype.sort semantics for equal keys, which is what
        // culori observes).
        indexed.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
        indexed
            .into_iter()
            .take(take)
            .map(|(i, _)| palette[i])
            .collect()
    }
}
