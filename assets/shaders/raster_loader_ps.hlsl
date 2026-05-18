[[vk::binding(0, 0)]] Texture2D Textures[];
[[vk::binding(1, 0)]] SamplerState tex_sampler;

struct PushConstants {
    uint text_idx;
};

[[vk::push_constant]] PushConstants push;

struct PSInput {
    float4 position : SV_POSITION;
    float2 uv       : TEXCOORD0;
};

float4 main(PSInput input) : SV_TARGET {
    float4 tex_image = Textures[push.text_idx].Sample(tex_sampler, input.uv);
    return tex_image;
}