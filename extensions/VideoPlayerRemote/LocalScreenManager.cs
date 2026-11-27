using System;
using System.Collections.Generic;
using ABI.CCK.Components;
using ABI_RC.Core.Player;
using UnityEngine;

namespace VideoPlayerRemote
{
    internal class LocalScreenInstance
    {
        public int Id { get; }
        public GameObject Root { get; }
        public CVRPickupObject Pickup { get; }
        public BoxCollider Collider { get; }
        public Transform DisplayQuad { get; }
        public Transform Bezel { get; }
        public MeshRenderer DisplayRenderer { get; }
        public Material VideoMaterial { get; }
        public Material IdleMaterial { get; }
        public Material BezelMaterial { get; }

        public float WidthScale { get; private set; }
        public bool GrabLocked { get; private set; }
        public bool FollowSelectedPlayer { get; set; }
        public CVRVideoPlayer PinnedPlayer { get; set; }
        public bool PinToLocalSpace { get; private set; }
        public bool FollowPlayspace { get; private set; }

        private Texture _lastBoundTexture;
        private bool _trackingInitialized;
        private bool _wasHeld;
        private Vector3 _localPinnedPos;
        private Quaternion _localPinnedRot = Quaternion.identity;
        private Vector3 _lastPlayspaceOffset;

        public LocalScreenInstance(
            int id,
            GameObject root,
            CVRPickupObject pickup,
            BoxCollider collider,
            Transform displayQuad,
            Transform bezel,
            MeshRenderer displayRenderer,
            Material videoMaterial,
            Material idleMaterial,
            Material bezelMaterial,
            float initialScale,
            bool followSelected,
            CVRVideoPlayer pinnedPlayer)
        {
            Id = id;
            Root = root;
            Pickup = pickup;
            Collider = collider;
            DisplayQuad = displayQuad;
            Bezel = bezel;
            DisplayRenderer = displayRenderer;
            VideoMaterial = videoMaterial;
            IdleMaterial = idleMaterial;
            BezelMaterial = bezelMaterial;
            FollowSelectedPlayer = followSelected;
            PinnedPlayer = pinnedPlayer;

            SetScale(initialScale);
            RefreshFeed();
        }

        public bool IsAlive => Root != null;

        public CVRVideoPlayer EffectiveTarget =>
            FollowSelectedPlayer ? VideoPlayerTracker.SelectedPlayer : PinnedPlayer;

        public string DisplayLabel
        {
            get
            {
                var target = EffectiveTarget;
                string source = FollowSelectedPlayer
                    ? "Auto"
                    : (VideoPlayerTracker.IsValid(target) ? VideoPlayerTracker.GetShortName(target) : "Unbound");
                string tags = "";
                if (PinToLocalSpace) tags += " [Local]";
                if (FollowPlayspace) tags += " [Playspace]";
                if (GrabLocked) tags += " [Locked]";
                return $"Screen #{Id} ({WidthScale:F1}m, {source}){tags}";
            }
        }

        public void SetScale(float widthMeters)
        {
            WidthScale = Mathf.Clamp(widthMeters, 0.3f, 4.0f);
            float height = WidthScale * (9f / 16f);
            float bezelPad = Mathf.Clamp(WidthScale * 0.03f, 0.02f, 0.08f);

            if (DisplayQuad != null)
            {
                DisplayQuad.localScale = new Vector3(WidthScale, height, 1f);
                DisplayQuad.localPosition = Vector3.zero;
            }

            if (Bezel != null)
            {
                Bezel.localScale = new Vector3(WidthScale + bezelPad, height + bezelPad, 0.022f);
                Bezel.localPosition = new Vector3(0f, 0f, 0.014f);
            }

            if (Collider != null)
            {
                Collider.size = new Vector3(WidthScale + bezelPad, height + bezelPad, 0.05f);
                Collider.center = new Vector3(0f, 0f, 0.01f);
            }
        }

        public void SetGrabLocked(bool locked)
        {
            GrabLocked = locked;
            if (Pickup != null)
                Pickup.enabled = !locked;
            if (Collider != null)
                Collider.enabled = !locked;
        }

        public void SetPinToLocalSpace(bool pin)
        {
            PinToLocalSpace = pin;
            ResnapshotTracking();
        }

        public void SetFollowPlayspace(bool follow)
        {
            FollowPlayspace = follow;
            ResnapshotTracking();
        }

        public void ResnapshotTracking()
        {
            if (!IsAlive || PlayerSetup.Instance == null || PlayerSetup.Instance.activeCam == null)
            {
                _trackingInitialized = false;
                return;
            }

            Transform playerRoot = PlayerSetup.Instance.transform;
            Vector3 playspaceOffset = GetPlayspaceOffset(playerRoot);
            Vector3 basePos = FollowPlayspace ? (Root.transform.position - playspaceOffset) : Root.transform.position;
            _localPinnedPos = playerRoot.InverseTransformPoint(basePos);
            _localPinnedRot = Quaternion.Inverse(playerRoot.rotation) * Root.transform.rotation;
            _lastPlayspaceOffset = playspaceOffset;
            _trackingInitialized = true;
            _wasHeld = false;
        }

        public void UpdateTracking()
        {
            if (!IsAlive || (!PinToLocalSpace && !FollowPlayspace) || PlayerSetup.Instance == null || PlayerSetup.Instance.activeCam == null)
                return;

            Transform playerRoot = PlayerSetup.Instance.transform;
            Vector3 playspaceOffset = GetPlayspaceOffset(playerRoot);

            bool isHeld = Pickup != null && (Pickup.IsGrabbedByMe || Pickup.ControllerRay != null);
            if (isHeld)
            {
                _wasHeld = true;
                _lastPlayspaceOffset = playspaceOffset;
                return;
            }

            if (_wasHeld || !_trackingInitialized)
            {
                ResnapshotTracking();
                return;
            }

            if (PinToLocalSpace)
            {
                Vector3 baseWorldPos = playerRoot.TransformPoint(_localPinnedPos);
                Root.transform.position = FollowPlayspace ? (baseWorldPos + playspaceOffset) : baseWorldPos;
                Root.transform.rotation = playerRoot.rotation * _localPinnedRot;
            }
            else if (FollowPlayspace)
            {
                Vector3 delta = playspaceOffset - _lastPlayspaceOffset;
                Root.transform.position += delta;
            }

            _lastPlayspaceOffset = playspaceOffset;
        }

        private static Vector3 GetPlayspaceOffset(Transform playerRoot)
        {
            Transform cam = PlayerSetup.Instance.activeCam.transform;
            return cam.position - playerRoot.position;
        }

        public void BringToPlayer()
        {
            if (Root == null || PlayerSetup.Instance == null)
                return;

            if (Pickup != null && Pickup.ControllerRay != null)
                Pickup.ControllerRay.DropObject(force: true);

            Transform cam = PlayerSetup.Instance.activeCam != null
                ? PlayerSetup.Instance.activeCam.transform
                : PlayerSetup.Instance.transform;

            Vector3 forwardFlat = Vector3.ProjectOnPlane(cam.forward, Vector3.up);
            if (forwardFlat.sqrMagnitude < 0.001f)
                forwardFlat = cam.forward;
            forwardFlat.Normalize();

            float distance = Mathf.Clamp(WidthScale * 1.15f, 1.1f, 3.2f);
            Root.transform.position = cam.position + forwardFlat * distance;
            Root.transform.rotation = Quaternion.LookRotation(forwardFlat, Vector3.up);
            ResnapshotTracking();
        }

        public void RefreshFeed()
        {
            if (!IsAlive || DisplayRenderer == null)
                return;

            var target = EffectiveTarget;
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

            if (tex == _lastBoundTexture && DisplayRenderer.sharedMaterial != null)
                return;

            _lastBoundTexture = tex;
            if (tex != null && VideoMaterial != null)
            {
                VideoMaterial.mainTexture = tex;
                DisplayRenderer.sharedMaterial = VideoMaterial;
            }
            else if (IdleMaterial != null)
            {
                DisplayRenderer.sharedMaterial = IdleMaterial;
            }
        }

        public void Destroy()
        {
            if (VideoMaterial != null)
                UnityEngine.Object.Destroy(VideoMaterial);
            if (IdleMaterial != null)
                UnityEngine.Object.Destroy(IdleMaterial);
            if (BezelMaterial != null)
                UnityEngine.Object.Destroy(BezelMaterial);
            if (Root != null)
                UnityEngine.Object.Destroy(Root);
        }
    }

    internal static class LocalScreenManager
    {
        private static readonly List<LocalScreenInstance> _screens = new List<LocalScreenInstance>();
        private static int _nextScreenId = 1;

        public static IReadOnlyList<LocalScreenInstance> Screens => _screens;
        public static event Action OnScreensChanged;

        public static void Init()
        {
            GameEvents.OnWorldLeave += HandleWorldLeave;
            VideoPlayerTracker.OnSelectedPlayerChanged += HandleSelectedPlayerChanged;
        }

        public static void Shutdown()
        {
            GameEvents.OnWorldLeave -= HandleWorldLeave;
            VideoPlayerTracker.OnSelectedPlayerChanged -= HandleSelectedPlayerChanged;
            CleanupAll(notify: false);
            OnScreensChanged = null;
        }

        private static void HandleWorldLeave()
        {
            CleanupAll(notify: false);
        }

        public static void LateUpdateTick()
        {
            for (int i = 0; i < _screens.Count; i++)
            {
                if (_screens[i].IsAlive)
                    _screens[i].UpdateTracking();
            }
        }

        public static void Tick()
        {
            bool removedAny = false;
            for (int i = _screens.Count - 1; i >= 0; i--)
            {
                if (!_screens[i].IsAlive)
                {
                    _screens.RemoveAt(i);
                    removedAny = true;
                }
                else
                {
                    _screens[i].RefreshFeed();
                }
            }

            if (removedAny)
                OnScreensChanged?.Invoke();
        }

        public static LocalScreenInstance SpawnScreen(CVRVideoPlayer target = null, float? scale = null, bool? followSelected = null)
        {
            try
            {
                int id = _nextScreenId++;
                float width = scale ?? Settings.DefaultScreenScale;
                bool follow = followSelected ?? Settings.DefaultFollowSelected;
                var boundPlayer = target ?? VideoPlayerTracker.SelectedPlayer;

                var root = new GameObject($"VideoRemote_LocalScreen_{id}");

                var collider = root.AddComponent<BoxCollider>();
                collider.isTrigger = true;

                var pickup = root.AddComponent<CVRPickupObject>();
                pickup.moveMode = CVRPickupObject.MoveMode.Transform;
                pickup.gripType = CVRPickupObject.GripType.Free;
                pickup.updateWithPhysics = false;
                pickup.maximumGrabDistance = 10f;
                pickup.disallowTheft = true;
                pickup.autoHold = false;

                // Front video quad (Unity Quad faces -Z, so when root +Z points away from player, front faces player)
                var quad = GameObject.CreatePrimitive(PrimitiveType.Quad);
                quad.name = "DisplayQuad";
                quad.transform.SetParent(root.transform, false);
                var quadCol = quad.GetComponent<Collider>();
                if (quadCol != null)
                    UnityEngine.Object.Destroy(quadCol);

                // Dark matte bezel and backplate
                var bezel = GameObject.CreatePrimitive(PrimitiveType.Cube);
                bezel.name = "Bezel";
                bezel.transform.SetParent(root.transform, false);
                var bezelCol = bezel.GetComponent<Collider>();
                if (bezelCol != null)
                    UnityEngine.Object.Destroy(bezelCol);

                var bezelRenderer = bezel.GetComponent<MeshRenderer>();
                Shader colorShader = Shader.Find("Unlit/Color") ?? Shader.Find("Standard");
                var bezelMat = new Material(colorShader);
                bezelMat.color = new Color(0.08f, 0.09f, 0.12f, 1f);
                bezelRenderer.sharedMaterial = bezelMat;

                Shader texShader = Shader.Find("Unlit/Texture") ?? Shader.Find("Standard");
                var videoMat = new Material(texShader);
                var idleMat = new Material(colorShader);
                idleMat.color = new Color(0.03f, 0.04f, 0.06f, 1f);

                var displayRenderer = quad.GetComponent<MeshRenderer>();
                displayRenderer.sharedMaterial = idleMat;

                var instance = new LocalScreenInstance(
                    id,
                    root,
                    pickup,
                    collider,
                    quad.transform,
                    bezel.transform,
                    displayRenderer,
                    videoMat,
                    idleMat,
                    bezelMat,
                    width,
                    follow,
                    boundPlayer);

                instance.BringToPlayer();
                _screens.Add(instance);
                OnScreensChanged?.Invoke();
                return instance;
            }
            catch (Exception ex)
            {
                VideoPlayerRemoteMod.Log?.Error($"Failed to spawn local screen: {ex}");
                return null;
            }
        }

        public static void DespawnScreen(LocalScreenInstance screen)
        {
            if (screen == null)
                return;

            _screens.Remove(screen);
            screen.Destroy();
            OnScreensChanged?.Invoke();
        }

        public static void CleanupAll(bool notify = true)
        {
            for (int i = _screens.Count - 1; i >= 0; i--)
            {
                _screens[i].Destroy();
            }
            _screens.Clear();
            if (notify)
                OnScreensChanged?.Invoke();
        }

        private static void HandleSelectedPlayerChanged(CVRVideoPlayer player)
        {
            foreach (var screen in _screens)
            {
                if (screen.IsAlive && screen.FollowSelectedPlayer)
                    screen.RefreshFeed();
            }
        }
    }
}
