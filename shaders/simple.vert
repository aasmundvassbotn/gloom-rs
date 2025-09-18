#version 430 core

layout (location = 0) in vec4 position;
layout (location = 1) in vec4 color;

uniform mat4 uTransform;

out vec4 vColor;

void main() {
    gl_Position = uTransform * position;
    vColor = color;
}