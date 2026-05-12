struct VSInput {
    float3 position : POSITION;
    float3 color    : COLOR0;
};

struct VSOutput {
    float4 position : SV_POSITION;
    float4 color    : COLOR0;
};

VSOutput main(VSInput input) {
    VSOutput output;
    output.position = float4(input.position, 1.0);
    output.color    = float4(input.color, 1.0);
    return output;
}