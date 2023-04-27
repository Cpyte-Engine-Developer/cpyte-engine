#version 450

layout(location = 0) in vec3 in_pos;
layout(location = 1) in vec3 in_color;
layout(location = 2) in vec2 in_texture_coords;

layout(location = 0) out vec3 out_color;
layout(location = 1) out vec2 out_texture_coords;

layout(binding = 0) uniform UBO {
	mat4 view;
	mat4 projection;
} ubo;

layout(push_constant) uniform PushConstants {
	mat4 model;
} pc;

void main() {
    gl_Position = ubo.projection * ubo.view * pc.model * vec4(in_pos, 1.0);
    out_color = in_color;
	out_texture_coords = in_texture_coords;
}
