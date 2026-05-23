struct InstanceInput {
    @location(1) position: vec2<f32>,
    @location(2) color: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
    @location(1) uv: vec2<f32>,
};

@vertex
fn vs_main( @builtin(vertex_index) in_vertex_index: u32, instance: InstanceInput) -> VertexOutput {
    // [-1.0, 1.0] screen coords

    var pos = array<vec2<f32>, 4>(
        vec2<f32>(-0.5, -0.5),
        vec2<f32>( 0.5, -0.5),
        vec2<f32>(-0.5,  0.5),
        vec2<f32>( 0.5,  0.5)
    );
    
    var out: VertexOutput;
    out.clip_position = vec4<f32>(instance.position + (pos[in_vertex_index] * 0.04), 0.0, 1.0);
    out.color = instance.color;
    out.uv = pos[in_vertex_index] + 0.5;
    
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(in.color, 1.0);
}
