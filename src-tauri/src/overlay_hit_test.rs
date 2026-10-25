//! Logical button bounds and physical desktop cursor conversion. No hooks,
//! input simulation or game access: only the overlay's cursor hit policy.
use serde::Deserialize;

#[derive(Debug, Clone, Copy, Deserialize)]
pub struct ActionBounds {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl ActionBounds {
    pub fn valid(self, viewport_width: f64, viewport_height: f64) -> bool {
        [
            self.x,
            self.y,
            self.width,
            self.height,
            viewport_width,
            viewport_height,
        ]
        .iter()
        .all(|value| value.is_finite())
            && self.x >= 0.0
            && self.y >= 0.0
            && self.width >= 1.0
            && self.width <= 420.0
            && self.height >= 1.0
            && self.height <= 100.0
            && self.x + self.width <= viewport_width
            && self.y + self.height <= viewport_height
    }

    pub fn contains_cursor(self, cursor: (f64, f64), origin: (f64, f64), scale: f64) -> bool {
        if !scale.is_finite() || scale <= 0.0 {
            return false;
        }
        let x = (cursor.0 - origin.0) / scale;
        let y = (cursor.1 - origin.1) / scale;
        x >= self.x && x < self.x + self.width && y >= self.y && y < self.y + self.height
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn limits_renderer_to_a_small_finite_rectangle_inside_the_viewport() {
        let rect = ActionBounds {
            x: 10.0,
            y: 30.0,
            width: 220.0,
            height: 36.0,
        };
        assert!(rect.valid(960.0, 150.0));
        for bad in [
            ActionBounds {
                width: 960.0,
                ..rect
            },
            ActionBounds { x: -1.0, ..rect },
            ActionBounds {
                height: f64::NAN,
                ..rect
            },
            ActionBounds { y: 149.0, ..rect },
        ] {
            assert!(!bad.valid(960.0, 150.0));
        }
    }
    #[test]
    fn handles_scaled_and_negative_monitor_coordinates_and_excludes_other_content() {
        let rect = ActionBounds {
            x: 10.0,
            y: 30.0,
            width: 220.0,
            height: 36.0,
        };
        let origin = (-1920.0, 100.0);
        assert!(rect.contains_cursor((-1903.5, 146.5), origin, 1.5));
        assert!(!rect.contains_cursor((-1920.0, 100.0), origin, 1.5));
        assert!(!rect.contains_cursor((-1575.0, 146.5), origin, 1.5));
        assert!(!rect.contains_cursor((-1903.5, 146.5), origin, 0.0));
    }
}
