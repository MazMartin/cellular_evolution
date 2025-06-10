fn circle_sdf (pos: vec2<f32>) -> f32 {
    return length(pos) - 1.0;
}

fn fmod(x: f32, y: f32) -> f32 {
    return x - y * floor(x / y);
}

fn regular_polygon_sdf(n: u32, p: vec2<f32>) -> f32 {
    let pi = 3.141592653589793;
    let angle = atan2(p.y, p.x);
    let radius = length(p);

    let angle_per_side = 2.0 * pi / f32(n);
    let normalized_angle = fmod(angle + 2.0 * pi, 2.0 * pi);
    let sector_angle = abs(fmod(normalized_angle, angle_per_side) - angle_per_side * 0.5);
    let edge_dist = radius * cos(sector_angle) - cos(angle_per_side * 0.5);

    return edge_dist;
}


fn star_sdf(n: u32, inner_radius: f32, p: vec2<f32>) -> f32 {
    let pi = 3.141592653589793;
    let angle = atan2(p.y, p.x);
    let radius = length(p);

    let total_points = 2u * n;
    let angle_step = 2.0 * pi / f32(total_points);

    let normalized_angle = fmod(angle + 2.0 * pi, 2.0 * pi);

    // Figure out which point we're near (outer or inner)
    let point_index = u32(floor(normalized_angle / angle_step));
    let is_inner = (point_index % 2u) == 1u;
    // Pick radius: outer = 1.0, inner = inner_radius
    let r = select(1.0, inner_radius, is_inner);
    // Get angle offset into current "wedge"
    let local_angle = normalized_angle - f32(point_index) * angle_step;
    let half_step = angle_step * 0.5;
    // Signed distance approximation
    let edge_dist = radius * cos(local_angle - half_step) - r * cos(half_step);
    return edge_dist;
}