#version 430 core

layout (location = 0) in vec4 position;
layout (location = 1) in vec4 color;
layout (location = 2) in vec3 normal;

uniform mat4 uTransform;

out vec4 vColor;

void main() {
    gl_Position = uTransform * position;

    vec3 lightDirection = normalize(vec3(0.8, -0.5, 0.6));

    float ndotl = max(0.0, dot(normalize(normal), -lightDirection));

    vColor = color * ndotl; // scales RGBA by diffuse term
}