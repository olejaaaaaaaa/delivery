[[vk::binding(0, 0)]] Texture2D Textures[];
[[vk::binding(1, 0)]] SamplerState tex_sampler;

[[vk::binding(0, 1)]] TextureCube sky;

struct PSInput {
    float4 position : SV_POSITION;
    float3 dir : TEXCOORD0;
};

// float3 dir_to_cube_uv(float3 d) {
//     float3 ad = abs(d);
//     float face;
//     float2 uv;

//     if (ad.x >= ad.y && ad.x >= ad.z) {
//         face = d.x > 0 ? 0 : 1;
//         uv = d.x > 0 ? float2(-d.z, -d.y) / ad.x
//                      : float2( d.z, -d.y) / ad.x;
//     } else if (ad.y >= ad.x && ad.y >= ad.z) {
//         face = d.y > 0 ? 2 : 3;
//         uv = d.y > 0 ? float2( d.x,  d.z) / ad.y
//                      : float2( d.x, -d.z) / ad.y;
//     } else {
//         face = d.z > 0 ? 4 : 5;
//         uv = d.z > 0 ? float2( d.x, -d.y) / ad.z
//                      : float2(-d.x, -d.y) / ad.z;
//     }

//     uv = uv * 0.5 + 0.5;
//     return float3(uv, face);
// }

// float4 main(PSInput input) : SV_TARGET {
//     float3 uvw = dir_to_cube_uv(normalize(input.dir));
    
//     int3 coord = int3(
//         (int)(uvw.x * 64.0),
//         (int)(uvw.y * 64.0),
//         (int)(uvw.z)          
//     );
//     coord = clamp(coord, int3(0,0,0), int3(63, 63, 5));
    
//     float3 sky_color = sky.Load(int4(coord, 0)).rgb;
//     return float4(sky_color, 1.0);
// }

// float4 main(PSInput input) : SV_TARGET {
//     float3 dir = normalize(input.dir);
//     float t = dir.y * 0.5 + 0.5;
//     float3 color = lerp(float3(0.3, 2.0, 1.0), float3(1.0, 1.0, 1.0), t);
//     return float4(color, 1.0);
// }

float4 main(PSInput input) : SV_TARGET {
    float3 dir = normalize(input.dir);
    // dir.z = -dir.z;
    // dir.y = -dir.y;
    // dir.x = -dir.x;
    float3 color = sky.Sample(tex_sampler, dir).rgb;
     // ACES tonemap
    color = (color * (2.51 * color + 0.03)) / (color * (2.43 * color + 0.59) + 0.14);
    return float4(color, 1.0);
}