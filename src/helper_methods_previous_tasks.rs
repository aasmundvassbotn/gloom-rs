
// I just assume the color red, and i just use z = 0. So just in xy-plane
fn vertices_and_indices_circle(cx: f32, cy: f32, r: f32) -> (Vec<f32>, Vec<u32>){
    let segments: usize = 16;

    // x, y, z, r, g, b
    let mut vertices = Vec::<f32>::with_capacity((segments + 1) * 6);
    let mut indices  = Vec::<u32>::with_capacity(segments * 3);

    // Center
    vertices.extend_from_slice(&[cx, cy, 0.0, 1.0, 0.0, 0.0]);

    // Constr circlepoints with maths
    let two_pi = std::f32::consts::PI * 2.0;
    for i in 0..segments {
        let t = i as f32 * two_pi / segments as f32;
        let x = cx + r * t.cos();
        let y = cy + r * t.sin();
        vertices.extend_from_slice(&[x, y, 0.0, 1.0, 0.0, 0.0]);
    }

    // Indices
    for i in 0..segments {
        let i1 = (i as u32) + 1;
        let i2 = ((i as u32 + 1) % segments as u32) + 1;
        indices.extend_from_slice(&[0, i1, i2]);
    }

    (vertices, indices)
}
