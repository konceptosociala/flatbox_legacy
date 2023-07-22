#version 450

// VERTEX
layout (location=0) in vec3 position;
layout (location=1) in vec3 normal;
layout (location=2) in vec2 texcoord;
// INSTANCE
layout (location=3) in vec3 color;
layout (location=4) in uint albedo_map;
layout (location=5) in int blank;

layout (set=0, binding=0) uniform UniformBufferObject {
    mat4 view_matrix;
    mat4 projection_matrix;
} ubo;

layout( push_constant ) uniform PushConstants {
    mat4 model_matrix;
    mat4 inverse_model_matrix;
} pc;

layout (location=0) out vec2 uv;
layout (location=1) out vec3 out_normal;
layout (location=2) out vec3 out_worldpos;
layout (location=3) out vec3 out_camera_coordinates;

layout (location=4) out vec3 out_color;
layout (location=5) out uint out_albedo_map;
layout (location=6) out int out_blank;

void main() {
    vec4 worldpos = pc.model_matrix * vec4(position, 1.0);
    gl_Position = ubo.projection_matrix * ubo.view_matrix * worldpos;
    gl_PointSize = 20.0;

    uv = texcoord;
    out_normal = transpose(mat3(pc.inverse_model_matrix)) * normal;
    out_worldpos = vec3(worldpos);
    out_camera_coordinates =
        - ubo.view_matrix[3][0] * vec3 (ubo.view_matrix[0][0],ubo.view_matrix[1][0],ubo.view_matrix[2][0])
        - ubo.view_matrix[3][1] * vec3 (ubo.view_matrix[0][1],ubo.view_matrix[1][1],ubo.view_matrix[2][1])
        - ubo.view_matrix[3][2] * vec3 (ubo.view_matrix[0][2],ubo.view_matrix[1][2],ubo.view_matrix[2][2]);

    out_color = color;
    out_albedo_map = albedo_map;
    out_blank = blank;
}
