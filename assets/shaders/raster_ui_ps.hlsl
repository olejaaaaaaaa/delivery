[[vk::binding(0, 0)]] Texture2D Textures[];
[[vk::binding(1, 0)]] SamplerState tex_sampler;

struct PSInput {
    float4 position : SV_POSITION;
    float2 uv    : TEXCOORD0; 
};

float4 main(PSInput input) : SV_TARGET {
    return Textures[0].Sample(tex_sampler, input.uv);
}