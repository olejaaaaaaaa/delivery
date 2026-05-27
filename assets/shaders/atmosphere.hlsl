struct FrameConstants
{
    float4 sky_ambient;
    float4 sun_color_multiplier;
    float3 sun_direction; 
    float  pre_exposure;
};
[[vk::push_constant]] FrameConstants frame_constants;

[[vk::binding(0)]] RWTexture2DArray<float4> output_tex;

#define PI                  3.14159265359
#define INFINITY            (1.0 / 0.0)
#define PLANET_RADIUS       6371000.0
#define PLANET_CENTER       float3(0.0, -PLANET_RADIUS, 0.0)
#define ATMOSPHERE_HEIGHT   100000.0
#define RAYLEIGH_HEIGHT     (ATMOSPHERE_HEIGHT * 0.08)
#define MIE_HEIGHT          (ATMOSPHERE_HEIGHT * 0.012)

static const float3 C_RAYLEIGH = float3(5.802, 13.558, 33.100) * 1e-6;
static const float3 C_MIE      = float3(3.996,  3.996,  3.996) * 1e-6;
static const float3 C_OZONE    = float3(0.650,  1.881,  0.085) * 1e-6;

#define ATMOSPHERE_DENSITY  1.0
#define EXPOSURE            5.0

float2 SphereIntersection(float3 rayStart, float3 rayDir, float3 sphereCenter, float sphereRadius)
{
    rayStart -= sphereCenter;
    float a = dot(rayDir, rayDir);
    float b = 2.0 * dot(rayStart, rayDir);
    float c = dot(rayStart, rayStart) - sphereRadius * sphereRadius;
    float d = b * b - 4.0 * a * c;
    if (d < 0.0) return float2(-1.0, -1.0);
    d = sqrt(d);
    return float2(-b - d, -b + d) / (2.0 * a);
}

float2 PlanetIntersection(float3 rayStart, float3 rayDir)
{
    return SphereIntersection(rayStart, rayDir, PLANET_CENTER, PLANET_RADIUS);
}

float2 AtmosphereIntersection(float3 rayStart, float3 rayDir)
{
    return SphereIntersection(rayStart, rayDir, PLANET_CENTER, PLANET_RADIUS + ATMOSPHERE_HEIGHT);
}

float PhaseRayleigh(float costh)
{
    return 3.0 * (1.0 + costh * costh) / (16.0 * PI);
}

float PhaseMie(float costh, float g)
{
    g = min(g, 0.9381);
    float k = 1.55 * g - 0.55 * g * g * g;
    float kcosth = k * costh;
    return (1.0 - k * k) / ((4.0 * PI) * (1.0 - kcosth) * (1.0 - kcosth));
}

float AtmosphereHeight(float3 positionWS)
{
    return distance(positionWS, PLANET_CENTER) - PLANET_RADIUS;
}

float DensityRayleigh(float h) { return exp(-max(0.0, h / RAYLEIGH_HEIGHT)); }
float DensityMie     (float h) { return exp(-max(0.0, h / MIE_HEIGHT)); }
float DensityOzone   (float h) { return max(0.0, 1.0 - abs(h - 25000.0) / 15000.0); }

float3 AtmosphereDensity(float h)
{
    return float3(DensityRayleigh(h), DensityMie(h), DensityOzone(h));
}

float3 Absorb(float3 opticalDepth)
{
    return exp(-(opticalDepth.x * C_RAYLEIGH + opticalDepth.y * C_MIE * 1.1 + opticalDepth.z * C_OZONE) * ATMOSPHERE_DENSITY);
}

float3 IntegrateOpticalDepth(float3 rayStart, float3 rayDir)
{
    float2 intersection = AtmosphereIntersection(rayStart, rayDir);
    float  rayLength    = intersection.y;

    const int sampleCount = 8;
    float     stepSize    = rayLength / sampleCount;

    float3 opticalDepth = float3(0.0, 0.0, 0.0);

    for (int i = 0; i < sampleCount; i++)
    {
        float3 localPosition = rayStart + rayDir * (i + 0.5) * stepSize;
        float  localHeight   = AtmosphereHeight(localPosition);
        opticalDepth += AtmosphereDensity(localHeight) * stepSize;
    }

    return opticalDepth;
}

float3 IntegrateScattering(float3 rayStart, float3 rayDir, float rayLength,
                            float3 lightDir, float3 lightColor,
                            out float3 transmittance)
{
    float rayHeight = AtmosphereHeight(rayStart);
    float sampleDistributionExponent = 1.0 + clamp(1.0 - rayHeight / ATMOSPHERE_HEIGHT, 0.0, 1.0) * 8.0;

    float2 intersection = AtmosphereIntersection(rayStart, rayDir);
    rayLength = min(rayLength, intersection.y);
    if (intersection.x > 0.0)
    {
        rayStart  += rayDir * intersection.x;
        rayLength -= intersection.x;
    }

    float costh  = dot(rayDir, lightDir);
    float phaseR = PhaseRayleigh(costh);
    float phaseM = PhaseMie(costh, 0.85);

    const int sampleCount = 6;

    float3 opticalDepth = float3(0.0, 0.0, 0.0);
    float3 rayleigh     = float3(0.0, 0.0, 0.0);
    float3 mie          = float3(0.0, 0.0, 0.0);
    float  prevRayTime  = 0.0;

    for (int i = 0; i < sampleCount; i++)
    {
        float  rayTime  = pow((float)i / sampleCount, sampleDistributionExponent) * rayLength;
        float  stepSize = rayTime - prevRayTime;

        float3 localPosition = rayStart + rayDir * rayTime;
        float  localHeight   = AtmosphereHeight(localPosition);
        float3 localDensity  = AtmosphereDensity(localHeight);

        opticalDepth += localDensity * stepSize;

        float3 viewTransmittance  = Absorb(opticalDepth);
        float3 opticalDepthLight  = IntegrateOpticalDepth(localPosition, lightDir);
        float3 lightTransmittance = Absorb(opticalDepthLight);

        rayleigh += viewTransmittance * lightTransmittance * phaseR * localDensity.x * stepSize;
        mie      += viewTransmittance * lightTransmittance * phaseM * localDensity.y * stepSize;

        prevRayTime = rayTime;
    }

    transmittance = Absorb(opticalDepth);
    return (rayleigh * C_RAYLEIGH + mie * C_MIE) * lightColor * frame_constants.pre_exposure;
}

float3 atmosphere_default(float3 wi, float3 light_dir)
{
    float3 transmittance;
    float3 scatter = IntegrateScattering(
        float3(0.0, 0.0, 0.0),
        wi,
        INFINITY,
        light_dir,
        float3(1.0, 1.0, 1.0),
        transmittance);

    return (frame_constants.sky_ambient.rgb
          + frame_constants.sun_color_multiplier.rgb * scatter)
         * frame_constants.pre_exposure;
}

static const float3x3 CUBE_MAP_FACE_ROTATIONS[6] = {
    float3x3(0,0,-1, 0,-1,0, -1,0,0),
    float3x3(0,0,1,  0,-1,0,  1,0,0),
    float3x3(1,0,0,  0,0,-1,  0,1,0),
    float3x3(1,0,0,  0,0,1,   0,-1,0),
    float3x3(1,0,0,  0,-1,0,  0,0,-1),
    float3x3(-1,0,0, 0,-1,0,  0,0,1),
};

[numthreads(8, 8, 1)]
void main(in uint3 px : SV_DispatchThreadID)
{
    uint   face = px.z;
    float2 uv   = ((float2)px.xy + 0.5) / 64.0;
    float3 dir  = normalize(mul(CUBE_MAP_FACE_ROTATIONS[face], float3(uv * 2.0 - 1.0, -1.0)));

    float3 color = atmosphere_default(dir, frame_constants.sun_direction);
    output_tex[px] = float4(color, 1.0);
}