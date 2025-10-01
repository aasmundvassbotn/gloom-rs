#version 430 core

in vec4 vColor;
in vec3 vNormal;

out vec4 outColor;

void main() {
    vec3 lightDirection = normalize(vec3(0.8, -0.5, 0.6));
    float ndotl = max(0.0, dot(vNormal, -lightDirection));

    vec3 lit = vColor.rgb * ndotl;
    outColor = vec4(lit, vColor.a);
}
