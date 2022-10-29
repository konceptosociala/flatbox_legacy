#version 450

layout (location=0) in vec3 position;
layout (location=1) in vec3 position_offset;
layout (location=2) in vec3 colour;

layout (location=0) out vec4 colourdata_for_the_fragmentshader;

void main() {
    gl_Position = vec4(position + position_offset, 1.0);
    colourdata_for_the_fragmentshader = vec4(colour, 1.0);
}
