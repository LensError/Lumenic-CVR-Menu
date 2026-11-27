using System;
using ABI.CCK.Components;
using ABI_RC.Core.Player;
using UnityEngine;
using UnityEngine.Rendering;

namespace VideoPlayerRemote
{
    internal static class HudScreenManager
    {
        private static readonly int GuiZTestProp = Shader.PropertyToID("unity_GUIZTestMode");

        private static GameObject _root;
        private static Transform _displayQuad;
        private static MeshRenderer _displayRenderer;
        private static Material _videoMaterial;
        private static Texture _lastBoundTexture;
        private static bool _snapped;
        private static bool _creationFailed;

        public static bool IsActive => _root != null && _root.activeSelf;

        public static void Init()
        {
            GameEvents.OnWorldLeave += HandleWorldLeave;
            VideoPlayerTracker.OnSelectedPlayerChanged += HandleSelectedChanged;

            if (Settings.HudEnabled)
                EnsureCreated();
        }

        public static void Shutdown()
        {
            GameEvents.OnWorldLeave -= HandleWorldLeave;
            VideoPlayerTracker.OnSelectedPlayerChanged -= HandleSelectedChanged;
            DestroyHud();
        }

        public static void SetEnabled(bool enabled)
        {
            Settings.HudEnabled = enabled;
            if (enabled)
            {
                EnsureCreated();
                ApplyVisualSettings();
                RefreshFeed();
                _snapped = false;
            }
            else
            {
                DestroyHud();
            }
        }

        public static void ApplyVisualSettings()
        {
            if (_root == null)
                return;

            float width = Settings.HudScale;
            float height = width * (9f / 16f);

            if (_displayQuad != null)
            {
                _displayQuad.localScale = new Vector3(width, height, 1f);
                _displayQuad.localPosition = Vector3.zero;
            }

            int zTest = Settings.HudOverdraw
                ? (int)CompareFunction.Always
                : (int)CompareFunction.LessEqual;

            float alpha = Settings.HudOpacity;

            if (_videoMaterial != null)
            {
                if (_videoMaterial.HasProperty(GuiZTestProp))
                    _videoMaterial.SetInt(GuiZTestProp, zTest);
                _videoMaterial.renderQueue = Settings.HudOverdraw ? 4000 : 3000;
                _videoMaterial.color = _lastBoundTexture != null
                    ? new Color(1f, 1f, 1f, alpha)
                    : new Color(0.03f, 0.04f, 0.06f, alpha * 0.7f);
            }
        }

        public static void Tick()
        {
            if (!Settings.HudEnabled || _creationFailed)
                return;

            if (_root == null)
                EnsureCreated();

            RefreshFeed();
        }

        public static void LateUpdateTick()
        {
            if (_root == null || !Settings.HudEnabled || PlayerSetup.Instance == null || PlayerSetup.Instance.activeCam == null)
                return;

            Transform cam = PlayerSetup.Instance.activeCam.transform;

            Vector3 localOffset = new Vector3(Settings.HudOffsetX, Settings.HudOffsetY, Settings.HudDistance);
            Vector3 targetPos = cam.position + cam.rotation * localOffset;
            Quaternion targetRot = Quaternion.LookRotation(cam.forward, cam.up);

            if (!_snapped || !Settings.HudSmoothFollow || Vector3.SqrMagnitude(_root.transform.position - targetPos) > 4f)
            {
                _root.transform.SetPositionAndRotation(targetPos, targetRot);
                _snapped = true;
                return;
            }

            float t = 1f - Mathf.Exp(-10f * Time.unscaledDeltaTime);
            _root.transform.position = Vector3.Lerp(_root.transform.position, targetPos, t);
            _root.transform.rotation = Quaternion.Slerp(_root.transform.rotation, targetRot, t);
        }

        private static void EnsureCreated()
        {
            if (_root != null)
                return;

            try
            {
                _root = new GameObject("VideoRemote_HudScreen");
                _root.layer = 15; // CVRLayers.UIInternal - excludes HUD from mirrors and PortableCamera
                UnityEngine.Object.DontDestroyOnLoad(_root);

                Shader uiShader = Shader.Find("UI/Default")
                                  ?? Shader.Find("Sprites/Default")
                                  ?? Shader.Find("Unlit/Texture")
                                  ?? Shader.Find("Standard");

                _videoMaterial = new Material(uiShader);

                var quad = GameObject.CreatePrimitive(PrimitiveType.Quad);
                quad.name = "HudDisplayQuad";
                quad.layer = 15;
                quad.transform.SetParent(_root.transform, false);
                var quadCol = quad.GetComponent<Collider>();
                if (quadCol != null)
                    UnityEngine.Object.Destroy(quadCol);
                _displayQuad = quad.transform;
                _displayRenderer = quad.GetComponent<MeshRenderer>();
                _displayRenderer.sharedMaterial = _videoMaterial;
                _displayRenderer.shadowCastingMode = ShadowCastingMode.Off;
                _displayRenderer.receiveShadows = false;

                _lastBoundTexture = null;
                _snapped = false;
                ApplyVisualSettings();
                RefreshFeed();
            }
            catch (Exception ex)
            {
                _creationFailed = true;
                VideoPlayerRemoteMod.Log?.Error($"Failed to create HUD screen: {ex}");
                DestroyHud();
            }
        }

        private static void RefreshFeed()
        {
            if (_root == null || _videoMaterial == null)
                return;

            var target = VideoPlayerTracker.SelectedPlayer;
            Texture tex = null;
            if (VideoPlayerTracker.IsValid(target))
            {
                tex = target.ProjectionTexture;
                if (tex == null)
                {
                    try { tex = target.VideoPlayer?.GetVideoTexture(); }
                    catch { tex = null; }
                }
            }

            if (tex == _lastBoundTexture)
                return;

            _lastBoundTexture = tex;
            _videoMaterial.mainTexture = tex;
            ApplyVisualSettings();
        }

        private static void HandleWorldLeave()
        {
            _lastBoundTexture = null;
            if (_videoMaterial != null)
                _videoMaterial.mainTexture = null;
            _snapped = false;
            ApplyVisualSettings();
        }

        private static void HandleSelectedChanged(CVRVideoPlayer _)
        {
            RefreshFeed();
        }

        private static void DestroyHud()
        {
            if (_videoMaterial != null)
                UnityEngine.Object.Destroy(_videoMaterial);
            if (_root != null)
                UnityEngine.Object.Destroy(_root);

            _root = null;
            _displayQuad = null;
            _displayRenderer = null;
            _videoMaterial = null;
            _lastBoundTexture = null;
            _snapped = false;
        }
    }
}
