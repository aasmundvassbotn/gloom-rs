#version 430 core

layout (location = 0) in vec4 position;
layout (location = 1) in vec4 color;
layout (location = 2) in vec3 normal;

uniform mat4 model;
uniform mat4 modelViewProj;

out vec4 vColor;
out vec3 vNormal;

void main() {
    gl_Position = modelViewProj * model * position;
    vColor = color;

    mat3 normalMatrix = mat3(model);
    vNormal = normalize(normalMatrix * normal);
}