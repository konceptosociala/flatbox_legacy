#version 450

layout (location=0) in vec4 position;
layout (location=1) in float size;
layout (location=2) in vec4 colour;

layout (location=0) out vec4 colourdata_for_the_fragmentshader;

void main() {
    gl_PointSize = size;
    gl_Position = position;
    colourdata_for_the_fragmentshader = colour;
}
