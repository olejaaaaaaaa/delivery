struct PushConstants {
    float4x4 inv_view_proj;
};

[[vk::push_constant]] PushConstants pc;

struct VSInput {
    uint vertex_id : SV_VertexID;
};

struct VSOutput {
    float4 position : SV_POSITION;
    float3 dir      : TEXCOORD0;
};

VSOutput main(VSInput input) {
    float2 positions[3] = {
        float2(-1.0, -1.0),
        float2( 3.0, -1.0),
        float2(-1.0,  3.0)
    };
    float2 pos2d = positions[input.vertex_id];
    float4 pos = float4(pos2d, 1.0, 1.0);
    float4 world = mul(pc.inv_view_proj, pos);

    VSOutput output;
    output.position = float4(pos2d, 0.0, 1.0);
    output.position.z = output.position.w;
    output.dir = world.xyz / world.w;
    return output;
}
