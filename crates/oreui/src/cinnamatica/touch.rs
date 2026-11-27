//! iOS touch-control configuration drawn from the guest definitions.
use super::*;
impl Window<'_> {
    /// Configuration rows are declared by the iOS guest, including their titles.
    pub(super) fn touch_controls(
        &self,
        canvas: &mut Canvas<'_>,
        card: Bounds,
        top: f32,
        hits: &mut Hits,
    ) -> Result<(), PaintError> {
        let pad = canvas.r(PAD);
        let left = card[0] + pad;
        let right = card[2] - pad;
        let y = self.heading(canvas, "bw_touch", [left, top], right - left)?;
        let width = (right - left - pad) * 0.5;
        let keys: Vec<_> = self
            .input
            .controls
            .iter()
            .filter_map(|control| match control {
                Control::Toggle { id, .. } => id
                    .strip_prefix(SETTING_PREFIX)
                    .filter(|key| key.starts_with("touch-")),
                _ => None,
            })
            .collect();
        let rows = keys.len().div_ceil(2).max(1);
        let row_height = canvas.r(ROW);
        for (index, key) in keys.into_iter().enumerate() {
            let x = left + (index / rows) as f32 * (width + pad);
            let row_y = y + (index % rows) as f32 * row_height;
            self.switch_row(canvas, key, [x, row_y, x + width, row_y + row_height], hits)?;
        }
        Ok(())
    }
}
