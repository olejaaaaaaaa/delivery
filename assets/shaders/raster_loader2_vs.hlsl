struct VSInput {
    uint vertex_id : SV_VertexID;
};

struct VSOutput {
    float4 position : SV_POSITION;
    float2 uv       : TEXCOORD0;
};

VSOutput main(VSInput input) {
    float2 positions[3] = {
        float2(-1.0, -1.0),
        float2( 3.0, -1.0),
        float2(-1.0,  3.0)
    };
    float2 uvs[3] = {
        float2(0.0, 0.0),
        float2(2.0, 0.0),
        float2(0.0, 2.0)
    };

    VSOutput output;
    output.position = float4(positions[input.vertex_id], 0.0, 1.0);
    output.uv = uvs[input.vertex_id];
    return output;
}