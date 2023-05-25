#version 450
#extension GL_EXT_nonuniform_qualifier : enable

layout (location=0) out vec4 theColour;

layout (location=0) out vec2 uv;
layout (location=1) out vec3 normal;
layout (location=2) out vec3 worldpos;
layout (location=3) out vec3 camera_coordinates;

layout (location=4) out vec3 color;
layout (location=5) out uint albedo_map;
layout (location=6) out float metallic;
layout (location=7) out uint metallic_map;
layout (location=8) out float roughness;
layout (location=9) out uint roughness_map;
layout (location=19) out float normal_t;
layout (location=11) out uint normal_t_map;

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
	vec3 lightColor = vec3(1.0);	
    
    float ambientStrength = 0.1;
    vec3 ambient = ambientStrength * lightColor;
  	
    // diffuse 
    vec3 norm = normalize(normal);
    vec3 lightDir = normalize(sbo.data[0]);
    float diff = max(dot(norm, lightDir), 0.0);
    vec3 diffuse = diff * lightColor;
    
    // specular
    float specularStrength = 0.5;
    vec3 viewDir = normalize(camera_coordinates - worldpos);
    vec3 reflectDir = reflect(-lightDir, norm);  
    float spec = pow(max(dot(viewDir, reflectDir), 0.0), 256);
    vec3 specular = specularStrength * spec * lightColor;  
        
    vec3 result = (ambient + diffuse + specular) * vec3(texture(texturesamplers[albedo_map], uv));
    theColour = vec4(result, 1.0);
}
