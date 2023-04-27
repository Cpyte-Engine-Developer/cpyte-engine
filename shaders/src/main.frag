#version 450

layout(location = 0) in vec3 in_color;
layout(location = 1) in vec2 in_texture_coords;

layout(location = 0) out vec4 out_color;

layout(binding = 1) uniform sampler2D textureSampler;

void main() {
    out_color = texture(textureSampler, in_texture_coords);
}
