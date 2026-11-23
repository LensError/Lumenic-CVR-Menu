using System;
using System.Diagnostics;
using System.Runtime.InteropServices;
using ABI_RC.Core.UI;
using MelonLoader;
using UnityEngine;

namespace LumenicMenu.Native;

/// <summary>
/// Manages per-menu Texture2D, pinned RGBA8 readback buffer, dynamic ViewTextureMips blitting,
/// pointer hover/leave tracking, refresh-rate throttling, and Stopwatch profiling.
/// </summary>
internal sealed class MenuSurfaceHost : IDisposable
{
    public readonly uint SurfaceId;
    public readonly string Name;
    private readonly float _referenceLogicalHeight;

    private Texture2D _texture;
    private byte[] _pixelBuffer;
    private GCHandle _pinnedBuffer;
    private bool _hasPinnedBuffer;

    private int _width;
    private int _height;
    private float _scale;
    private bool _linearTexture;
    private bool _needsFirstBlit = true;
    private RenderTexture _lastBlittedMips;
    private bool _liveOverlayActive;
    private Rect _liveOverlayNormRect;
    private Material _overlayBlendMat;

    private float _lastRenderUnscaledTime;
    private float _lastPerfLogUnscaledTime;
    private int _renderedFramesSinceLog;
    private int _idleFramesSinceLog;
    private readonly Stopwatch _sw = new Stopwatch();

    public bool HitThisFrame { get; set; }
    public bool HasDiscreteInput { get; set; }
    public bool WasHovered { get; private set; }
    public bool IsTextFocused { get; private set; }
    public bool PointerOverUi { get; private set; }
    public bool WasGrabAborted { get; set; }

    public int Width => _width;
    public int Height => _height;

    public float LastStepMs { get; private set; }
    public float LastReadbackMs { get; private set; }
    public float LastUploadMs { get; private set; }
    public float LastBlitMs { get; private set; }

    public MenuSurfaceHost(uint surfaceId, string name, float referenceLogicalHeight)
    {
        SurfaceId = surfaceId;
        Name = name;
        _referenceLogicalHeight = Mathf.Max(320f, referenceLogicalHeight);
    }

    public float ComputeEffectiveScale(int physicalHeight)
    {
        float baseScale = Mathf.Clamp(physicalHeight / _referenceLogicalHeight, 0.5f, 4.0f);
        return Mathf.Clamp(baseScale * Settings.UiScale, 0.5f, 4.0f);
    }

    public bool IsLiveOverlayActive => _liveOverlayActive;

    public void SetLiveOverlay(bool active, float normX, float normY, float normW, float normH)
    {
        bool nextActive = active && normW > 0.001f && normH > 0.001f;
        Rect nextRect = nextActive
            ? new Rect(
                Mathf.Clamp01(normX),
                Mathf.Clamp01(normY),
                Mathf.Clamp(normW, 0.001f, 1f),
                Mathf.Clamp(normH, 0.001f, 1f))
            : default;

        if (_liveOverlayActive != nextActive || _liveOverlayNormRect != nextRect)
        {
            _liveOverlayActive = nextActive;
            _liveOverlayNormRect = nextRect;
            _needsFirstBlit = true;
        }
    }

    public void ClearLiveOverlay()
    {
        if (_liveOverlayActive)
        {
            _liveOverlayActive = false;
            _liveOverlayNormRect = default;
            _needsFirstBlit = true;
        }
    }

    public void RequestForceRedraw()
    {
        _needsFirstBlit = true;
    }

    public void InvalidateNativeSurface()
    {
        _scale = 0f;
        _needsFirstBlit = true;
    }

    public void ResetTransientInteraction(NativeBridge bridge)
    {
        HitThisFrame = false;
        HasDiscreteInput = false;
        WasHovered = false;
        IsTextFocused = false;
        PointerOverUi = false;
        WasGrabAborted = false;

        if (bridge != null && bridge.IsHealthy)
        {
            bridge.PointerCancel(SurfaceId);
            bridge.PointerLeave(SurfaceId);
            bridge.ClearFocus(SurfaceId);
        }
    }

    public bool StepAndBlit(
        NativeBridge bridge,
        CohtmlControlledView view,
        float dtSecs,
        out bool hasActions,
        out bool fatalError,
        RenderTexture liveOverlay = null)
    {
        hasActions = false;
        fatalError = false;

        if (bridge == null || !bridge.IsHealthy || view == null || view.ViewTextureMips == null)
            return false;

        int targetW = Math.Max(1, view.Width);
        int targetH = Math.Max(1, view.Height);
        float targetScale = ComputeEffectiveScale(targetH);
        bool targetLinear = Settings.LinearTextureMode;

        // Leave-check runs in OnLateUpdate after ControllerRay.LateUpdate.
        if (WasHovered && !HitThisFrame)
        {
            bridge.PointerLeave(SurfaceId);
        }
        WasHovered = HitThisFrame;
        HitThisFrame = false;

        bool sizeOrScaleChanged = targetW != _width || targetH != _height || Mathf.Abs(targetScale - _scale) > 0.001f;
        bool texFormatChanged = _texture == null || targetLinear != _linearTexture || targetW != _width || targetH != _height;

        if (sizeOrScaleChanged)
        {
            if (!bridge.Resize(SurfaceId, (uint)targetW, (uint)targetH, targetScale))
            {
                fatalError = true;
                return false;
            }
            _scale = targetScale;
            _needsFirstBlit = true;
        }

        if (texFormatChanged)
        {
            ReallocateBuffers(targetW, targetH, targetLinear);
            _needsFirstBlit = true;
        }

        if (!ReferenceEquals(view.ViewTextureMips, _lastBlittedMips))
        {
            _needsFirstBlit = true;
        }

        // Respect MaxRefreshHz cap for continuous hover/drag/scroll/animation frames,
        // but never delay discrete clicks, releases, pokes, or keyboard events.
        float now = Time.unscaledTime;
        float minInterval = 1.0f / Settings.MaxRefreshHz;
        bool throttled = !_needsFirstBlit && !HasDiscreteInput && (now - _lastRenderUnscaledTime) < minInterval;
        if (throttled)
        {
            return false;
        }

        HasDiscreteInput = false;
        float effectiveDt = _lastRenderUnscaledTime > 0f
            ? Mathf.Clamp(now - _lastRenderUnscaledTime, 0.001f, 0.25f)
            : Mathf.Clamp(dtSecs, 0.001f, 0.25f);
        _sw.Restart();
        uint flags = bridge.Step(SurfaceId, effectiveDt, _needsFirstBlit);
        _sw.Stop();
        float stepMs = (float)_sw.Elapsed.TotalMilliseconds;

        if ((flags & NativeBridge.StepFlagErrorOrPanic) != 0)
        {
            fatalError = true;
            return false;
        }

        IsTextFocused = (flags & NativeBridge.StepFlagTextFocused) != 0;
        PointerOverUi = (flags & NativeBridge.StepFlagPointerOverUi) != 0;
        hasActions = (flags & NativeBridge.StepFlagHasActions) != 0;

        bool drawOverlay = _liveOverlayActive && liveOverlay != null;
        if ((flags & NativeBridge.StepFlagRendered) == 0 && !_needsFirstBlit)
        {
            if (drawOverlay && _texture != null)
            {
                RenderTexture overlayMips = view.ViewTextureMips;
                if (overlayMips != null)
                {
                    _sw.Restart();
                    BlitSurfaceAndOverlay(overlayMips, liveOverlay);
                    _sw.Stop();
                    LastBlitMs = (float)_sw.Elapsed.TotalMilliseconds;
                    _lastRenderUnscaledTime = now;
                    _lastBlittedMips = overlayMips;
                }
                return true;
            }

            _idleFramesSinceLog++;
            MaybeLogPerf(now);
            return false;
        }

        _sw.Restart();
        bool readOk = bridge.ReadPixels(SurfaceId, _pinnedBuffer.AddrOfPinnedObject(), _pixelBuffer.Length);
        _sw.Stop();
        float readMs = (float)_sw.Elapsed.TotalMilliseconds;

        if (!readOk)
        {
            fatalError = true;
            return false;
        }

        _sw.Restart();
        _texture.LoadRawTextureData(_pinnedBuffer.AddrOfPinnedObject(), _pixelBuffer.Length);
        _texture.Apply(updateMipmaps: false, makeNoLongerReadable: false);
        _sw.Stop();
        float uploadMs = (float)_sw.Elapsed.TotalMilliseconds;

        // Always read view.ViewTextureMips dynamically (RecreateViewTexture can replace it at runtime).
        RenderTexture mips = view.ViewTextureMips;
        _sw.Restart();
        if (mips != null)
        {
            BlitSurfaceAndOverlay(mips, drawOverlay ? liveOverlay : null);
            _lastBlittedMips = mips;
        }
        _sw.Stop();
        float blitMs = (float)_sw.Elapsed.TotalMilliseconds;

        LastStepMs = stepMs;
        LastReadbackMs = readMs;
        LastUploadMs = uploadMs;
        LastBlitMs = blitMs;
        _lastRenderUnscaledTime = now;
        _needsFirstBlit = false;
        _renderedFramesSinceLog++;
        MaybeLogPerf(now);

        return true;
    }

    private void MaybeLogPerf(float now)
    {
        if (!Settings.PerfLogging || (now - _lastPerfLogUnscaledTime) < 1.0f)
            return;

        _lastPerfLogUnscaledTime = now;
        float total = LastStepMs + LastReadbackMs + LastUploadMs + LastBlitMs;
        MelonLogger.Msg(
            $"[LumenicMenu][Perf][{Name} {_width}x{_height} @{_scale:F2}x] rendered={_renderedFramesSinceLog} idle={_idleFramesSinceLog} | step={LastStepMs:F2}ms read={LastReadbackMs:F2}ms upload={LastUploadMs:F2}ms blit={LastBlitMs:F2}ms total={total:F2}ms");
        _renderedFramesSinceLog = 0;
        _idleFramesSinceLog = 0;
    }

    public static void RestoreVanillaFrame(CohtmlControlledView view)
    {
        if (view == null)
            return;

        // Trigger a single vanilla Cohtml redraw without latching ForceRender forever.
        view.m_HasDrawn = false;
        view.m_ShouldRedraw = true;

        RenderTexture src = view.ViewTexture;
        RenderTexture dst = view.ViewTextureMips;
        if (src != null && dst != null)
        {
            Graphics.Blit(src, dst);
            dst.GenerateMips();
        }
    }

    private void ReallocateBuffers(int width, int height, bool linear)
    {
        ReleaseBuffers();

        _width = width;
        _height = height;
        _linearTexture = linear;

        _texture = new Texture2D(width, height, TextureFormat.RGBA32, mipChain: false, linear: linear)
        {
            name = $"LumenicMenu_{Name}_{width}x{height}",
            wrapMode = TextureWrapMode.Clamp,
            filterMode = FilterMode.Bilinear
        };

        _pixelBuffer = new byte[width * height * 4];
        _pinnedBuffer = GCHandle.Alloc(_pixelBuffer, GCHandleType.Pinned);
        _hasPinnedBuffer = true;
    }

    private void ReleaseBuffers()
    {
        if (_hasPinnedBuffer)
        {
            _pinnedBuffer.Free();
            _hasPinnedBuffer = false;
        }
        _pixelBuffer = null;

        if (_texture != null)
        {
            UnityEngine.Object.Destroy(_texture);
            _texture = null;
        }
        if (_overlayBlendMat != null)
        {
            UnityEngine.Object.Destroy(_overlayBlendMat);
            _overlayBlendMat = null;
        }
        _lastBlittedMips = null;
    }

    private void BlitSurfaceAndOverlay(RenderTexture mips, RenderTexture liveOverlay)
    {
        // Flip V on GPU: OpenGL bottom-up Texture2D -> CVR MenuFX D3D11 top-down ViewTextureMips.
        Graphics.Blit(_texture, mips, new Vector2(1f, -1f), new Vector2(0f, 1f));

        if (liveOverlay != null && liveOverlay.width > 0 && liveOverlay.height > 0)
        {
            // Compute aspect-fit in logical display space (_width x _height) because ViewTextureMips
            // is allocated as NextPowerOfTwo (e.g. 4096x2048 for 2560x1440), which is non-square pixels.
            float dispW = Mathf.Max(1f, _width);
            float dispH = Mathf.Max(1f, _height);
            float boxW = _liveOverlayNormRect.width * dispW;
            float boxH = _liveOverlayNormRect.height * dispH;

            float texAspect = (float)liveOverlay.width / liveOverlay.height;
            float boxAspect = boxW / Mathf.Max(1f, boxH);

            // Portrait avatar cards are often taller/narrower than the 512x768 (2:3) preview RT.
            // Center-crop up to ~40% of the empty horizontal side padding so the avatar fills the
            // card vertically instead of looking tiny with a huge gap above its head.
            float fitW = boxW;
            float fitH = boxH;
            float srcX = 0f;
            float srcW = 1f;
            float srcY = 0f;
            float srcH = 1f;

            if (boxAspect > texAspect)
            {
                if (texAspect > 1.2f)
                {
                    // Landscape overlay (e.g. 16:9 WorldPano): Cover-crop vertically.
                    srcH = texAspect / boxAspect;
                    srcY = (1f - srcH) * 0.5f;
                }
                else
                {
                    fitW = boxH * texAspect;
                }
            }
            else if (boxAspect < texAspect)
            {
                // Tall/narrow box: fill height by cropping horizontal transparent margin (clamped so wide avatars don't clip).
                float neededSrcW = boxAspect / texAspect;
                float clampedSrcW = Mathf.Clamp(neededSrcW, 0.58f, 1f);
                srcW = clampedSrcW;
                srcX = (1f - clampedSrcW) * 0.5f;
                float effectiveTexAspect = texAspect * clampedSrcW;
                if (boxAspect < effectiveTexAspect)
                {
                    fitH = boxW / effectiveTexAspect;
                }
            }

            float normDrawX = _liveOverlayNormRect.x + ((boxW - fitW) * 0.5f) / dispW;
            float normDrawY = _liveOverlayNormRect.y + ((boxH - fitH) * 0.5f) / dispH;
            float normDrawW = fitW / dispW;
            float normDrawH = fitH / dispH;

            float drawX = normDrawX * mips.width;
            float drawY = normDrawY * mips.height;
            float drawW = normDrawW * mips.width;
            float drawH = normDrawH * mips.height;

            // ViewTextureMips is stored V-flipped for CVR MenuFX: mirror both destination Y and source V.
            float flippedDrawY = mips.height - (drawY + drawH);
            Rect srcRect = new Rect(srcX, 1f - srcY, srcW, -srcH);

            if (_overlayBlendMat == null)
            {
                Shader sh = Shader.Find("UI/Default") ?? Shader.Find("Sprites/Default");
                if (sh != null)
                {
                    _overlayBlendMat = new Material(sh) { name = "LumenicAvatarPreviewBlend" };
                }
            }

            RenderTexture prevActive = RenderTexture.active;
            bool prevSrgbWrite = GL.sRGBWrite;
            try
            {
                RenderTexture.active = mips;
                GL.sRGBWrite = mips.sRGB;
                GL.PushMatrix();
                try
                {
                    GL.LoadPixelMatrix(0f, mips.width, mips.height, 0f);
                    Graphics.DrawTexture(
                        new Rect(drawX, flippedDrawY, drawW, drawH),
                        liveOverlay,
                        srcRect,
                        0,
                        0,
                        0,
                        0,
                        Color.white,
                        _overlayBlendMat);
                }
                finally
                {
                    GL.PopMatrix();
                }
            }
            finally
            {
                GL.sRGBWrite = prevSrgbWrite;
                RenderTexture.active = prevActive;
            }
        }

        mips.GenerateMips();
    }

    public void Dispose()
    {
        ReleaseBuffers();
    }
}
