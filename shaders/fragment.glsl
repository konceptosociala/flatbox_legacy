#version 450

layout (location=0) out vec4 theColour;

layout (location=0) in vec3 colour_in;


void main(){
	theColour=vec4(colour_in,1.0);
}
