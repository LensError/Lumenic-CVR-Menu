using System;
using LumenicMenu.Api;
using UnityEngine;

namespace LumenicUiShowcase;

// Procedural textures demonstrating RegisterRawRgba and RegisterTexture2D.
// Keys prefixed with "ext:" are pinned in Lumenic's texture cache so browsing worlds/avatars never evicts them.
internal static class ShowcaseTextures
{
    public const string RawRgbaKey = "ext:showcase_gradient";
    public const string UnityTexKey = "ext:showcase_checker";
    public const string FallbackDemoKey = "ext:showcase_missing_demo";

    public static void RegisterAll(int seed = 0)
    {
        UploadRawRgbaGradient(seed);
        UploadUnityTexture2D(seed);
    }

    // Raw RGBA upload: top-left origin, thread-safe, no Unity Texture2D allocation required.
    public static void UploadRawRgbaGradient(int seed)
    {
        const uint w = 160;
        const uint h = 90;
        byte[] rgba = new byte[w * h * 4];
        float phase = seed * 0.85f;

        for (uint y = 0; y < h; y++)
        {
            float v = y / (float)(h - 1);
            for (uint x = 0; x < w; x++)
            {
                float u = x / (float)(w - 1);
                float wave = 0.5f + 0.5f * (float)Math.Sin((u * 6.0f) + phase + (v * 2.5f));
                byte r = (byte)(35 + 165 * u * wave);
                byte g = (byte)(90 + 130 * (1f - v) * (0.4f + 0.6f * wave));
                byte b = (byte)(155 + 95 * v);

                int idx = (int)((y * w + x) * 4);
                rgba[idx] = r;
                rgba[idx + 1] = g;
                rgba[idx + 2] = b;
                rgba[idx + 3] = 255;
            }
        }

        LumenicApi.RegisterRawRgba(RawRgbaKey, w, h, rgba, ExtSurface.Both);
    }

    // Unity Texture2D upload: must run on Unity's main thread; Lumenic flips Y automatically.
    public static void UploadUnityTexture2D(int seed)
    {
        const int size = 128;
        var tex = new Texture2D(size, size, TextureFormat.RGBA32, false);
        try
        {
            var pixels = new Color32[size * size];
            Color32 cA = seed % 2 == 0 ? new Color32(28, 36, 52, 255) : new Color32(34, 28, 52, 255);
            Color32 cB = seed % 2 == 0 ? new Color32(56, 189, 248, 255) : new Color32(52, 211, 153, 255);
            Color32 ring = new Color32(248, 250, 252, 255);

            float cx = size * 0.5f;
            float cy = size * 0.5f;

            for (int y = 0; y < size; y++)
            {
                for (int x = 0; x < size; x++)
                {
                    bool check = ((x / 16) + (y / 16)) % 2 == 0;
                    float dx = x - cx;
                    float dy = y - cy;
                    float dist = (float)Math.Sqrt(dx * dx + dy * dy);

                    Color32 col = check ? cA : cB;
                    if (dist >= 34f && dist <= 40f)
                        col = ring;

                    pixels[y * size + x] = col;
                }
            }

            tex.SetPixels32(pixels);
            tex.Apply(false, false);
            LumenicApi.RegisterTexture2D(UnityTexKey, tex, ExtSurface.Both);
        }
        finally
        {
            UnityEngine.Object.Destroy(tex);
        }
    }
}
