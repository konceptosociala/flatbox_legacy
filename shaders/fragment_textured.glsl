#version 450
#extension GL_EXT_nonuniform_qualifier : enable

layout (location=0) out vec4 theColour;

layout (location=0) in vec2 uv;
layout (location=1) flat in uint texture_id;

layout (set=1, binding=0) uniform sampler2D texturesamplers[];



void main(){
	theColour = texture(texturesamplers[texture_id], uv);
}
