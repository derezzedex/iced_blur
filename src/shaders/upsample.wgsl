@group(0) @binding(0) var u_texture: texture_2d<f32>;
@group(0) @binding(1) var u_sampler: sampler;

struct VertexInput {
    @builtin(vertex_index) vertex_index: u32,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

fn vertex_position(vertex_index: u32) -> vec2<f32> {
    // #: 0 1 2 3 4 5
    // x: 1 1 0 0 0 1
    // y: 1 0 0 0 1 1
    return vec2<f32>((vec2(1u, 2u) + vertex_index) % vec2(6u) < vec2(3u));
}

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var uv = vertex_position(input.vertex_index);

    var out: VertexOutput;

    out.uv = uv;
    out.position = vec4<f32>(uv * vec2(2.0, -2.0) + vec2(-1.0, 1.0), 0.0, 1.0);

    return out;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    var texel_size = vec2<f32>(1) / vec2<f32>(textureDimensions(u_texture));
    var texel = texel_size;

    const sample = vec2<f32>(-1.0, 1.0);
    const sample2 = vec2<f32>(0.0, 2.0);

    return (1.0 / 6.0) * (
                    textureSample(u_texture, u_sampler, input.uv+texel*sample.xx) +
                    textureSample(u_texture, u_sampler, input.uv+texel*sample.yx) +
                    textureSample(u_texture, u_sampler, input.uv+texel*sample.xy) +
                    textureSample(u_texture, u_sampler, input.uv+texel*sample.yy)
                ) + 
                (1.0 / 12.0) * (
                    textureSample(u_texture, u_sampler, input.uv+texel*sample2.xy)+
                    textureSample(u_texture, u_sampler, input.uv-texel*sample2.xy)+
                    textureSample(u_texture, u_sampler, input.uv+texel*sample2.yx)+
                    textureSample(u_texture, u_sampler, input.uv-texel*sample2.yx)
                );
}