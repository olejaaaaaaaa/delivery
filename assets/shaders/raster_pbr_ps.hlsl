static const float PI = 3.14159265359;

[[vk::binding(0, 0)]] Texture2D Textures[];
[[vk::binding(1, 0)]] SamplerState tex_sampler;

[[vk::push_constant]] struct Push {
    uint tex_idx[8];    
    float user_data[24];       
} push;

struct VSOutput {
    float4 position  : SV_POSITION;
    float3 world_pos : TEXCOORD0;
    float3 normal    : TEXCOORD1; 
    float4 tangent   : TEXCOORD2;  
    float2 uv        : TEXCOORD3;
};

static const float3 LIGHT_POSITIONS[4] = {
    float3( 10.0,  10.0, 10.0),
    float3(-10.0,  10.0, 10.0),
    float3( 10.0, -10.0, 10.0),
    float3(-10.0, -10.0, 10.0)
};

static const float3 LIGHT_COLORS[4] = {
    float3(150.0, 300.0, 300.0),
    float3(300.0, 150.0, 300.0),
    float3(150.0, 300.0, 300.0),
    float3(300.0, 300.0, 150.0)
};

float3 getNormalFromMap(VSOutput input, Texture2D normal_tex) {
    float3 N = normalize(input.normal);
    float3 T = normalize(input.tangent.xyz);
    float3 B = cross(N, T) * input.tangent.w;
    float3x3 TBN = float3x3(T, B, N);

    float3 tangentNormal = normal_tex.Sample(tex_sampler, input.uv).rgb * 2.0 - 1.0;
    return normalize(mul(tangentNormal, TBN));
}

float DistributionGGX(float3 N, float3 H, float roughness) {
    float a  = roughness * roughness;
    float a2 = a * a;
    float NdotH  = max(dot(N, H), 0.0);
    float NdotH2 = NdotH * NdotH;
    float denom  = (NdotH2 * (a2 - 1.0) + 1.0);
    denom = PI * denom * denom;
    return a2 / max(denom, 0.0001);
}

float GeometrySchlickGGX(float NdotV, float roughness) {
    float r = roughness + 1.0;
    float k = (r * r) / 8.0;
    return NdotV / max(NdotV * (1.0 - k) + k, 0.0001);
}

float GeometrySmith(float3 N, float3 V, float3 L, float roughness) {
    float NdotV = max(dot(N, V), 0.0);
    float NdotL = max(dot(N, L), 0.0);
    return GeometrySchlickGGX(NdotV, roughness) * GeometrySchlickGGX(NdotL, roughness);
}

float3 fresnelSchlick(float cosTheta, float3 F0) {
    return F0 + (1.0 - F0) * pow(clamp(1.0 - cosTheta, 0.0, 1.0), 5.0);
}

float4 main(VSOutput input) : SV_TARGET {
    float3 albedo    = pow(Textures[push.tex_idx[0]].Sample(tex_sampler, input.uv).rgb, 2.2);
    float4 mr_sample = Textures[push.tex_idx[1]].Sample(tex_sampler, input.uv);
    float  ao        = Textures[push.tex_idx[2]].Sample(tex_sampler, input.uv).r;

    float metallic  = mr_sample.b;
    float roughness = mr_sample.g;

    float3 N = getNormalFromMap(input, Textures[push.tex_idx[3]]);
    float3 cam_pos = float3(push.user_data[0], push.user_data[1], push.user_data[2]);
    float3 V = normalize(cam_pos - input.world_pos);

    float3 F0 = lerp(float3(0.04, 0.04, 0.04), albedo, metallic);

    float3 Lo = float3(0.0, 0.0, 0.0);
    for (int i = 0; i < 4; ++i) {
        float3 L = normalize(LIGHT_POSITIONS[i] - input.world_pos);
        float3 H = normalize(V + L);
        float  distance    = length(LIGHT_POSITIONS[i] - input.world_pos);
        float  attenuation = 1.0 / (distance * distance);
        float3 radiance    = LIGHT_COLORS[i] * attenuation;

        float  NDF = DistributionGGX(N, H, roughness);
        float  G   = GeometrySmith(N, V, L, roughness);
        float3 F   = fresnelSchlick(max(dot(H, V), 0.0), F0);

        float3 numerator   = NDF * G * F;
        float  denominator = 4.0 * max(dot(N, V), 0.0) * max(dot(N, L), 0.0) + 0.0001;
        float3 specular    = numerator / denominator;

        float3 kS = F;
        float3 kD = (float3(1.0, 1.0, 1.0) - kS) * (1.0 - metallic);

        float NdotL = max(dot(N, L), 0.0);
        Lo += (kD * albedo / PI + specular) * radiance * NdotL;
    }

    float3 ambient = float3(0.003, 0.003, 0.003) * albedo * ao;
    float3 color   = ambient + Lo;
    color = color / (color + float3(1.0, 1.0, 1.0));
    color = pow(color, 1.0 / 2.2);

    return float4(color, 1.0);
}