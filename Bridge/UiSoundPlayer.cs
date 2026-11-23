using System;
using ABI_RC.Core.Savior;
using UnityEngine;

namespace LumenicMenu.Bridge;

/// <summary>
/// Synthesizes and plays Lumenic UI hover/click AudioClips in memory:
/// - "Glass" (Palette A default): minimal VisionOS-style crystal tap.
/// - "SoftSub" (Palette D option): warm low-fatigue VR sub-drop thump.
/// - "Vanilla": falls back to ChilloutVR's built-in InterfaceAudio.
/// </summary>
internal static class UiSoundPlayer
{
    private const int SampleRate = 48000;
    private const float HoverCooldownSecs = 0.035f;

    private static GameObject _hostObj;
    private static AudioSource _source;

    private static AudioClip _glassHover;
    private static AudioClip _glassClick;
    private static AudioClip _glassSelect;

    private static AudioClip _subHover;
    private static AudioClip _subClick;
    private static AudioClip _subSelect;

    private static float _lastHoverTime = -10f;

    public static bool TryPlay(string soundName)
    {
        if (!Settings.UiClickSounds || string.IsNullOrEmpty(soundName))
            return false;

        string pack = Settings.UiSoundPack?.Trim() ?? "Glass";
        if (string.Equals(pack, "Vanilla", StringComparison.OrdinalIgnoreCase))
        {
            // Suppress custom hover in Vanilla mode; let clicks fall through to CVR InterfaceAudio.
            return string.Equals(soundName, "Hover", StringComparison.OrdinalIgnoreCase);
        }

        EnsureInitialized();
        if (_source == null)
            return false;

        float volume = ResolveUiVolume();
        if (volume <= 0.001f)
            return true;

        bool useSub = string.Equals(pack, "SoftSub", StringComparison.OrdinalIgnoreCase);

        if (string.Equals(soundName, "Hover", StringComparison.OrdinalIgnoreCase))
        {
            float now = Time.unscaledTime;
            if ((now - _lastHoverTime) < HoverCooldownSecs)
                return true;
            _lastHoverTime = now;
            AudioClip clip = useSub ? _subHover : _glassHover;
            if (clip != null)
                _source.PlayOneShot(clip, volume * 0.68f);
            return true;
        }

        if (string.Equals(soundName, "Click", StringComparison.OrdinalIgnoreCase))
        {
            AudioClip clip = useSub ? _subClick : _glassClick;
            if (clip != null)
                _source.PlayOneShot(clip, volume * 0.90f);
            return true;
        }

        if (string.Equals(soundName, "Select", StringComparison.OrdinalIgnoreCase))
        {
            AudioClip clip = useSub ? _subSelect : _glassSelect;
            if (clip != null)
                _source.PlayOneShot(clip, volume * 0.85f);
            return true;
        }

        return false;
    }

    public static void Dispose()
    {
        DestroyClip(ref _glassHover);
        DestroyClip(ref _glassClick);
        DestroyClip(ref _glassSelect);
        DestroyClip(ref _subHover);
        DestroyClip(ref _subClick);
        DestroyClip(ref _subSelect);

        if (_hostObj != null)
        {
            UnityEngine.Object.Destroy(_hostObj);
            _hostObj = null;
            _source = null;
        }
    }

    private static void DestroyClip(ref AudioClip clip)
    {
        if (clip != null)
        {
            UnityEngine.Object.Destroy(clip);
            clip = null;
        }
    }

    private static void EnsureInitialized()
    {
        if (_source != null && _glassHover != null && _subHover != null)
            return;

        if (_hostObj == null)
        {
            _hostObj = new GameObject("LumenicUiSoundPlayer");
            UnityEngine.Object.DontDestroyOnLoad(_hostObj);
        }

        if (_source == null)
        {
            _source = _hostObj.GetComponent<AudioSource>() ?? _hostObj.AddComponent<AudioSource>();
            _source.playOnAwake = false;
            _source.loop = false;
            _source.spatialBlend = 0f;
            _source.bypassEffects = true;
            _source.bypassListenerEffects = true;
            _source.bypassReverbZones = true;
            _source.ignoreListenerPause = true;
        }

        _glassHover ??= BuildClip("LumenicGlassHover", SynthesizeGlassHover());
        _glassClick ??= BuildClip("LumenicGlassClick", SynthesizeGlassClick());
        _glassSelect ??= BuildClip("LumenicGlassSelect", SynthesizeGlassSelect());

        _subHover ??= BuildClip("LumenicSubHover", SynthesizeSubHover());
        _subClick ??= BuildClip("LumenicSubClick", SynthesizeSubClick());
        _subSelect ??= BuildClip("LumenicSubSelect", SynthesizeSubSelect());
    }

    private static float ResolveUiVolume()
    {
        try
        {
            var settings = MetaPort.Instance?.settings;
            if (settings == null)
                return 0.75f;

            float master = Mathf.Clamp01(settings.GetSettingsInt("AudioMaster", 100) / 100f);
            float uiVol = Mathf.Clamp01(settings.GetSettingsInt("AudioUi", 100) / 100f);
            return Mathf.Clamp01(master * uiVol * 0.85f);
        }
        catch
        {
            return 0.75f;
        }
    }

    private static float EnvAd(float t, float attack, float decay)
    {
        if (t < attack)
            return t / Mathf.Max(attack, 1e-6f);
        return Mathf.Exp(-(t - attack) / Mathf.Max(decay, 1e-6f));
    }

    // Palette A: Minimal VisionOS-style crystal hover tick (32 ms)
    private static float[] SynthesizeGlassHover()
    {
        int n = (int)(SampleRate * 0.032f);
        float[] samples = new float[n];
        const float twoPi = 2f * Mathf.PI;
        float ph = 0f;
        for (int i = 0; i < n; i++)
        {
            float t = (float)i / SampleRate;
            float e = EnvAd(t, 0.0015f, 0.0055f);
            float f = 1650f + 220f * Mathf.Exp(-t / 0.004f);
            ph += twoPi * f / SampleRate;
            samples[i] = (Mathf.Sin(ph) * 0.70f + Mathf.Sin(ph * 1.5f) * 0.25f) * e * 0.35f;
        }
        return Normalize(samples);
    }

    // Palette A: Minimal VisionOS-style damped glass click (65 ms)
    private static float[] SynthesizeGlassClick()
    {
        int n = (int)(SampleRate * 0.065f);
        float[] samples = new float[n];
        const float twoPi = 2f * Mathf.PI;
        float ph = 0f;
        for (int i = 0; i < n; i++)
        {
            float t = (float)i / SampleRate;
            float e1 = EnvAd(t, 0.0010f, 0.010f);
            float e2 = EnvAd(t, 0.0005f, 0.003f);
            float f = 920f + 680f * Mathf.Exp(-t / 0.0045f);
            ph += twoPi * f / SampleRate;
            samples[i] = Mathf.Sin(ph) * e1 * 0.65f + Mathf.Sin(twoPi * 2180f * t) * e2 * 0.28f;
        }
        return Normalize(samples);
    }

    // Palette A variant: Brighter harmonic ping for secondary/confirm cues (68 ms)
    private static float[] SynthesizeGlassSelect()
    {
        int n = (int)(SampleRate * 0.068f);
        float[] samples = new float[n];
        const float twoPi = 2f * Mathf.PI;
        float ph = 0f;
        for (int i = 0; i < n; i++)
        {
            float t = (float)i / SampleRate;
            float e1 = EnvAd(t, 0.0010f, 0.011f);
            float f = 1140f + 560f * Mathf.Exp(-t / 0.005f);
            ph += twoPi * f / SampleRate;
            samples[i] = (Mathf.Sin(ph) * 0.65f + Mathf.Sin(ph * 1.5f) * 0.25f) * e1;
        }
        return Normalize(samples);
    }

    // Palette D: Warm low-fatigue VR hover tap (30 ms)
    private static float[] SynthesizeSubHover()
    {
        int n = (int)(SampleRate * 0.030f);
        float[] samples = new float[n];
        const float twoPi = 2f * Mathf.PI;
        float ph = 0f;
        for (int i = 0; i < n; i++)
        {
            float t = (float)i / SampleRate;
            float e = EnvAd(t, 0.0025f, 0.0065f);
            float f = 520f + 180f * Mathf.Exp(-t / 0.004f);
            ph += twoPi * f / SampleRate;
            samples[i] = Mathf.Sin(ph) * e * 0.36f;
        }
        return Normalize(samples);
    }

    // Palette D: Warm low-fatigue VR pitch-dropped sub thump (68 ms)
    private static float[] SynthesizeSubClick()
    {
        int n = (int)(SampleRate * 0.068f);
        float[] samples = new float[n];
        const float twoPi = 2f * Mathf.PI;
        float ph = 0f;
        for (int i = 0; i < n; i++)
        {
            float t = (float)i / SampleRate;
            float e = EnvAd(t, 0.0012f, 0.013f);
            float f = 185f + 390f * Mathf.Exp(-t / 0.0055f);
            ph += twoPi * f / SampleRate;
            samples[i] = (Mathf.Sin(ph) * 0.80f + Mathf.Sin(ph * 2f) * 0.12f) * e * 0.72f;
        }
        return Normalize(samples);
    }

    // Palette D variant: Warm upward double-pulse for confirm cues (72 ms)
    private static float[] SynthesizeSubSelect()
    {
        int n = (int)(SampleRate * 0.072f);
        float[] samples = new float[n];
        const float twoPi = 2f * Mathf.PI;
        float ph = 0f;
        for (int i = 0; i < n; i++)
        {
            float t = (float)i / SampleRate;
            float e = EnvAd(t, 0.0015f, 0.014f);
            float f = 260f + 320f * Mathf.Exp(-t / 0.006f);
            ph += twoPi * f / SampleRate;
            samples[i] = (Mathf.Sin(ph) * 0.78f + Mathf.Sin(ph * 1.5f) * 0.18f) * e * 0.70f;
        }
        return Normalize(samples);
    }

    private static float[] Normalize(float[] samples)
    {
        float peak = 0f;
        for (int i = 0; i < samples.Length; i++)
        {
            float a = Mathf.Abs(samples[i]);
            if (a > peak) peak = a;
        }
        if (peak > 0.85f)
        {
            float scale = 0.85f / peak;
            for (int i = 0; i < samples.Length; i++)
                samples[i] *= scale;
        }
        return samples;
    }

    private static AudioClip BuildClip(string name, float[] samples)
    {
        AudioClip clip = AudioClip.Create(name, samples.Length, 1, SampleRate, stream: false);
        clip.SetData(samples, 0);
        return clip;
    }
}
