#version 450
#extension GL_EXT_nonuniform_qualifier : enable

layout (location=0) out vec4 theColour;

layout (location=0) in vec2 uv;
layout (location=1) in vec3 normal;
layout (location=2) in vec3 worldpos;
layout (location=3) in vec3 camera_coordinates;

layout (location=4) in vec3 color;
layout (location=5) in flat uint albedo_map;

layout (set=1, binding=0) uniform sampler2D texturesamplers[];

readonly layout (set=2, binding=0) buffer StorageBufferObject {
    float num_directional;
    float num_point;
    vec3 data[];
} sbo;

struct DirectionalLight{
    vec3 direction_to_light;
    vec3 irradiance;
};

struct PointLight{
    vec3 position;
    vec3 luminous_flux;
};

const float PI = 3.1415926535897932;

void main(){
    theColour = vec4(pow(texture(texturesamplers[albedo_map], uv).rgb * color, vec3(2.2)), 1.0);
}