#version 330 core

in VS_OUTPUT {
    vec4 Colour;
} IN;

out vec4 Colour;

void main()
{
    Colour = IN.Colour;
}
