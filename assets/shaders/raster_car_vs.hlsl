struct Transform {
    float4x4 model;
    float4x4 normal;
};

[[vk::binding(0, 1)]] ConstantBuffer<Transform> transform;

struct Camera {
    float4x4 view;
    float4x4 proj;
    float3   pos;
};

[[vk::binding(0, 2)]] ConstantBuffer<Camera> camera;

[[vk::push_constant]] struct Push {
    uint tex_idx[8];    
    float user_data[24];       
} push;

struct VSInput {
    float4 position    : POSITION;  
    float4 normal      : NORMAL; 
    float2 uv          : TEXCOORD0;
    float4 base_color  : COLOR0;  
    float4 tangent     : TANGENT;
};

struct VSOutput {
    float4 base_color: COLOR0;  
    float4 position  : SV_POSITION;
    float3 world_pos : TEXCOORD0;
    float3 normal    : TEXCOORD1; 
    float4 tangent   : TEXCOORD2;  
    float2 uv        : TEXCOORD3;
};

VSOutput main(VSInput input) {
    VSOutput output;

    float4 world_pos = mul(transform.model, input.position);
    output.position  = mul(camera.proj, mul(camera.view, world_pos));
    output.world_pos = world_pos.xyz;
    output.normal    = mul((float3x3)transform.normal, input.normal.xyz);
    output.tangent   = mul(transform.model, input.tangent);
    output.uv        = input.uv;
    output.base_color = input.base_color;

    return output;
}