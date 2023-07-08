#version 450
#extension GL_EXT_nonuniform_qualifier : enable

layout (location=0) out vec4 theColour;

layout (location=0) in vec2 uv;
layout (location=1) in vec3 normal;
layout (location=2) in vec3 worldpos;
layout (location=3) in vec3 camera_coordinates;

layout (location=4) in vec3 color;
layout (location=5) in flat uint albedo_map;
layout (location=6) in float metallic;
layout (location=7) in flat uint metallic_map;
layout (location=8) in float roughness;
layout (location=9) in flat uint roughness_map;
layout (location=10) in float normal_t;
layout (location=11) in flat uint normal_t_map;
layout (location=12) in float ao;
layout (location=13) in flat uint ao_map;

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

vec3 getNormalFromMap();
float DistributionGGX(vec3 N, vec3 H, float roughness);
float GeometrySchlickGGX(float NdotV, float roughness);
float GeometrySmith(vec3 N, vec3 V, vec3 L, float roughness);
vec3 fresnelSchlick(float cosTheta, vec3 F0);

void main(){ // TODO: Multiple lights bug
    int number_directional = int(sbo.num_directional);
    int number_point = int(sbo.num_point);

    vec3 c_albedo = pow(texture(texturesamplers[albedo_map], uv).rgb * color, vec3(2.2));
    float c_metallic = (texture(texturesamplers[metallic_map], uv) * metallic).r;
    float c_roughness = (texture(texturesamplers[roughness_map], uv) * roughness).r;
    float c_ao = (texture(texturesamplers[ao_map], uv) * ao).r;

    vec3 N = getNormalFromMap();
    vec3 V = normalize(camera_coordinates - worldpos);

    vec3 F0 = vec3(0.04); 
    F0 = mix(F0, c_albedo, c_metallic);

    vec3 Lo = vec3(0.0);

    for (int i = 0; i < number_directional; i++) {
        vec3 direction = sbo.data[2 * i];
        vec3 illuminance = sbo.data[2 * i + 1];
        DirectionalLight dlight = DirectionalLight(normalize(direction), illuminance);
    }

    for (int i = 0; i < number_point; i++) {
        vec3 position = sbo.data[2 * i + 2 * number_point];
        vec3 luminous_flux = sbo.data[2 * i + 1 + 2 * number_point];
        PointLight plight = PointLight(position, luminous_flux);

        vec3 L = normalize(plight.position - worldpos);
        vec3 H = normalize(V + L);
        float distance = length(plight.position - worldpos);
        float attenuation = 1.0 / (distance * distance);
        vec3 radiance = plight.luminous_flux * attenuation;

        float NDF = DistributionGGX(N, H, c_roughness);       
        float G = GeometrySmith(N, V, L, c_roughness); 
        vec3 F = fresnelSchlick(max(dot(H, V), 0.0), F0);

        vec3 numerator = NDF * G * F; 
        float denominator = 4.0 * max(dot(N, V), 0.0) * max(dot(N, L), 0.0) + 0.0001;
        vec3 specular = numerator / denominator;

        vec3 kS = F;
        vec3 kD = vec3(1.0) - kS;
        kD *= 1.0 - c_metallic;	
        
        float NdotL = max(dot(N, L), 0.0);        
        Lo += (kD * c_albedo / PI + specular) * radiance * NdotL;
    }

    vec3 ambient = vec3(0.03) * c_albedo * c_ao;
    vec3 color = ambient + Lo;  

    color = color / (color + vec3(1.0));
    color = pow(color, vec3(1.0/2.2)); 

    theColour = vec4(color, 1.0);
}

vec3 fresnelSchlick(float cosTheta, vec3 F0){
    return F0 + (1.0 - F0) * pow(clamp(1.0 - cosTheta, 0.0, 1.0), 5.0);
}  

float DistributionGGX(vec3 N, vec3 H, float roughness){
    float a      = roughness*roughness;
    float a2     = a*a;
    float NdotH  = max(dot(N, H), 0.0);
    float NdotH2 = NdotH*NdotH;
	
    float num   = a2;
    float denom = (NdotH2 * (a2 - 1.0) + 1.0);
    denom = PI * denom * denom;
	
    return num / denom;
}

float GeometrySchlickGGX(float NdotV, float roughness){
    float r = (roughness + 1.0);
    float k = (r*r) / 8.0;

    float num   = NdotV;
    float denom = NdotV * (1.0 - k) + k;
	
    return num / denom;
}

float GeometrySmith(vec3 N, vec3 V, vec3 L, float roughness){
    float NdotV = max(dot(N, V), 0.0);
    float NdotL = max(dot(N, L), 0.0);
    float ggx2  = GeometrySchlickGGX(NdotV, roughness);
    float ggx1  = GeometrySchlickGGX(NdotL, roughness);
	
    return ggx1 * ggx2;
}

vec3 getNormalFromMap(){
    vec3 tangentNormal = texture(texturesamplers[normal_t_map], uv).xyz * 2.0 - 1.0;

    vec3 Q1  = dFdx(worldpos);
    vec3 Q2  = dFdy(worldpos);
    vec2 st1 = dFdx(uv);
    vec2 st2 = dFdy(uv);

    vec3 N = normalize(normal);
    vec3 T = normalize(Q1*st2.t - Q2*st1.t);
    vec3 B = -normalize(cross(N, T));
    mat3 TBN = mat3(T, B, N);

    return normalize(TBN * tangentNormal);
}