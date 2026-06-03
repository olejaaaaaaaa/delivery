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

    // Фон: #413A31 в два раза темнее → примерно #201D19 (RGB: 32, 29, 25)
    float3 back_color = float3(1.0 / 255.0, 1.0 / 255.0, 1.0 / 255.0);
    
    // Другие линии: #591812 (RGB: 89, 24, 18)
    float3 other_lines_color = float3(140.0 / 255.0, 35.0 / 255.0, 8.0 / 255.0);

    float4 O = float4(back_color, 1.0);  // Фон

    for (float i = 0.0; i <= 4.0; i += 1.0) {
        float t = i / 5.0;
        float2 warped = uv;
        warped.x += (push.mouse_pos.x - uv.x) * 0.9 * S(1.0, 0.0, dist);
        float4 linee = Line(warped, 1.0 + t, 4.0 + t, other_lines_color);
        
        // Накопление: новая линия смешивается с тем, что уже есть
        O.rgb += linee.rgb;
        O.a = 1.0;
    }

    // Основная линия: rgb(227, 89, 34) (RGB: 213, 88, 38)
    float3 main_color = float3(247.0 / 255.0, 109.0 / 255.0, 54.0 / 255.0);

    float t = 1.0 / 5.0;
    uv.x += (push.mouse_pos.x - uv.x) * 0.9 * S(1.0, 0.0, dist);
    O += Line(uv, 1.0 + t, 4.0 + t, main_color);

    float glow = saturate(0.015 / (dist * dist + 0.01));
    O += float4(glow * main_color.x, glow * main_color.y, glow * main_color.z, 0.0);
    
    return O;
}