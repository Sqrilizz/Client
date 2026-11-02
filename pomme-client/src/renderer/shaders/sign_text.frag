#version 450

#include "fog.glsl"

layout(set = 1, binding = 0) uniform texture2DArray font_tex;
layout(set = 1, binding = 1) uniform sampler font_tex_sampler;

layout(location = 0) in vec2 v_uv;
layout(location = 1) in float v_layer;
layout(location = 2) in vec4 v_color;
layout(location = 3) in float v_fog;
layout(location = 4) in vec3 v_fog_color;

layout(location = 0) out vec4 out_color;

void main() {
    float coverage = texture(sampler2DArray(font_tex, font_tex_sampler), vec3(v_uv, v_layer)).r;
    if (coverage < 0.05) discard;
    vec4 color = vec4(v_color.rgb, v_color.a * coverage);
    color.rgb = apply_fog(color.rgb, v_fog, v_fog_color);
    out_color = color;
}
