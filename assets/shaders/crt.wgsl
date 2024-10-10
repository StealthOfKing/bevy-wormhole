// curved crt post process effect

#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput

@group(0) @binding(0) var screen_texture: texture_2d<f32>;
@group(0) @binding(1) var texture_sampler: sampler;
struct PostProcessSettings {
    time: f32,
}
@group(0) @binding(2) var<uniform> settings: PostProcessSettings;

fn curved_transform(in: FullscreenVertexOutput, curvature: vec2<f32>) -> vec2<f32> {
    var uv = in.uv * 2.0 - 1.0;
    let offset = abs(uv.yx) / curvature;
    uv = uv + uv * offset * offset;
    uv = uv * 0.5 + 0.5;
    return uv;
}

fn scanline_intensity(uv: f32, resolution: f32, opacity: f32) -> vec4<f32> {
    var intensity = sin(uv * resolution * 3.14159265359 * 2.0);
    intensity = ((0.5 * intensity) + 0.5) * 0.9 + 0.1;
    return vec4<f32>(vec3<f32>(pow(intensity, opacity)), 1.0);
}

fn vignette_intensity(uv: vec2<f32>, resolution: vec2<f32>, opacity: f32, roundness: f32) -> vec4<f32> {
    var intensity = uv.x * uv.y * (1.0 - uv.x) * (1.0 - uv.y);
    return vec4<f32>(vec3<f32>(clamp(pow((resolution.x / roundness) * intensity, opacity), 0.0, 1.0)), 1.0);
}

fn syncline_intensity(y: f32, size: f32, speed: f32) -> f32 {
    let position = (settings.time % speed) / speed;
    let intensity = size * (y - position) - y + 1;
    if (intensity < 0.0 || intensity > 1.0) {
        return 0.0;
    } else {
        return pow(intensity, 2.0);
    }
}

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let curvature = vec2<f32>(8.0, 8.0);
    let uv = curved_transform(in, curvature);
    if uv.x < 0 || uv.y < 0 || uv.x > 1 || uv.y > 1 {
        return vec4<f32>(0.0, 0.0, 0.0, 1.0);
    } else {
        let resolution = vec2<f32>(480.0, 960.0) * 0.75; // externalise, derive from screen resolution
        var colour = textureSample(screen_texture, texture_sampler, uv);
        colour *= vignette_intensity(uv, resolution, 1.0, 1.0);
        colour *= scanline_intensity(uv.x, resolution.y, 1.0); // vertical
        colour *= scanline_intensity(uv.y, resolution.x, 1.0); // horizontal
        let syncline = syncline_intensity(uv.y, 6.0, 5.0) * 0.5;
        let warmup = 1.0 - 1.0 / (1.0 * settings.time + 1.0); // reciprocal brightening over time
        colour *= vec4<f32>(vec3<f32>(3.0 + syncline), 1.0) * warmup; // brightness
        return colour;
    }
}
