//! Express physical movement in a temporary placement-facing coordinate system.
//! Wire X is left-positive. Changing yaw without rebasing these vectors makes the
//! server simulate a different walk from the client's unchanged camera and position.
pub(super) fn rebase([left, forward]: [f32; 2], physical_yaw: f32, reported_yaw: f32) -> [f32; 2] {
    if left == 0. && forward == 0. {
        return [0.; 2];
    }
    let (s, c) = (physical_yaw - reported_yaw).to_radians().sin_cos();
    [left * c - forward * s, left * s + forward * c]
}

#[cfg(test)]
mod tests {
    use super::rebase;
    fn world([l, f]: [f32; 2], yaw: f32) -> [f32; 2] {
        let (s, c) = yaw.to_radians().sin_cos();
        [l * c - f * s, l * s + f * c]
    }
    #[test]
    fn every_placement_turn_preserves_the_walk_in_world_space() {
        for yaw in [-180., -90., -32., 0., 45., 90., 179.] {
            for sent in [-180., -90., 0., 90., 180.] {
                for input in [[0., 1.], [1., 0.], [-1., -1.], [0.3, 0.7], [0., 0.]] {
                    let a = world(input, yaw);
                    let b = world(rebase(input, yaw, sent), sent);
                    assert!((a[0] - b[0]).abs() < 0.00001 && (a[1] - b[1]).abs() < 0.00001);
                }
            }
        }
    }
}
