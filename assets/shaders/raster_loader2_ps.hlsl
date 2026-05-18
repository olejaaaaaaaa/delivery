struct PushConstants {
    float speed;
    float time;
    float2 resolution;
    float2 mouse_pos;
};

[[vk::push_constant]] PushConstants push;

struct PSInput {
    float4 position : SV_POSITION;
    float2 uv       : TEXCOORD0;
};

#define S smoothstep

float4 Line(float2 uv, float speed, float height, float3 col) {
    uv.y += S(1.0, 0.0, abs(uv.x)) * cos(push.time * speed + uv.x * height) * 0.2;
    return float4(S(0.06 * S(0.0, 1.0, abs(uv.x)), 0.0, abs(uv.y) - 0.004) * col, 1.0) *  S(0.0, 0.0, abs(uv.x));
}

float4 main(PSInput input) : SV_TARGET {
    float2 uv = (input.uv * push.resolution - 0.5 * push.resolution) / push.resolution.y;

    float2 delta = uv - push.mouse_pos;
    float dist   = length(delta) / 0.1;

    float4 O = float4(0.0, 0.0, 0.0, 0.0);
    for (float i = 0.0; i <= 5.0; i += 1.0) {
        float t = i / 5.0;
        float2 warped = uv;
        warped.y += (push.mouse_pos.y - uv.y) * 0.9 * S(1.0, 0.0, dist);
        O += Line(warped, 1.0 + t, 4.0 + t, float3(0.1 + t * 0.3, 0.0, 0.4 + t * 0.6));
    }
    float glow = (0.2 / (dist + 0.01));
    O += float4(glow * 0.5, glow * 0.2, glow, 0.0);
    return O;
}