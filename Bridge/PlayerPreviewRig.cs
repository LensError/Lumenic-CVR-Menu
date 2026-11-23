using System;
using ABI.CCK.Components;
using ABI_RC.Core.IO;
using ABI_RC.Core.Networking;
using ABI_RC.Core.Player;
using ABI_RC.Core.Savior;
using ABI_RC.Core.Util.AnimatorManager;
using ABI_RC.Systems.UI.UILib;
using MelonLoader;
using UnityEngine;

namespace LumenicMenu.Bridge;

/// <summary>
/// Isolated Layer-12 studio turntable camera rig for previewing remote players' avatars
/// (and any instance-loaded avatar in the Content Browser) into a Linear ARGBHalf RenderTexture.
/// Positioned far away from CVR's AvatarSettingsPreview so the two Layer-12 cameras never overlap.
/// </summary>
internal sealed class PlayerPreviewRig : MonoBehaviour
{
    private const int PreviewLayerMask = 1 << 12; // CVRLayers.CameraOnly
    private const float FieldOfView = 40f;
    private const float FramingPadding = 1.2f;
    private const float MaxTopRatio = 1.6f;
    private const float DragDegrees = 360f;
    private const float DragResponse = 15f;
    private const float DragDamping = 3f;
    private const float DestroyDelaySecs = 5f;

    private static readonly Vector3 IsolatedRigPosition = new Vector3(250f, -5000f, 0f);

    private static PlayerPreviewRig _instance;

    private readonly AvatarAnimatorManager _animatorManager = new AvatarAnimatorManager();

    private Camera _camera;
    private Transform _cameraTransform;
    private Light _light;
    private Transform _avatarHolder;
    private RenderTexture _cameraTexture;

    private CVRAvatar _sourceAvatar;
    private CVRAvatar _previewAvatar;
    private Renderer[] _previewRenderers;
    private string _activeTarget = string.Empty;
    private bool _rendering;
    private bool _isLoading;
    private float _inactiveSince = -1f;

    private float _dragAngle;
    private float _dragVelocity;
    private float _dragDelta;
    private bool _isDragging;

    public static bool HasInstance => _instance != null;

    public static PlayerPreviewRig Instance
    {
        get
        {
            if (_instance == null)
            {
                var go = new GameObject("LumenicPlayerPreviewRig");
                go.transform.position = IsolatedRigPosition;
                DontDestroyOnLoad(go);
                _instance = go.AddComponent<PlayerPreviewRig>();
            }
            return _instance;
        }
    }

    public RenderTexture Texture => _rendering && _previewAvatar != null ? _cameraTexture : null;

    public static bool IsLocalTarget(string target)
    {
        if (string.IsNullOrEmpty(target) || target == "local")
            return true;

        if (target.StartsWith("player:", StringComparison.Ordinal))
        {
            string uid = target.Substring("player:".Length);
            return IsLocalUserId(uid);
        }

        if (target.StartsWith("avatar:", StringComparison.Ordinal))
        {
            string aid = target.Substring("avatar:".Length);
            string localGuid = MetaPort.Instance?.currentAvatarGuid;
            return !string.IsNullOrEmpty(localGuid)
                && string.Equals(aid, localGuid, StringComparison.OrdinalIgnoreCase);
        }

        return false;
    }

    public static bool IsLocalUserId(string userId)
    {
        if (string.IsNullOrEmpty(userId))
            return true;
        if (userId == PlayerSetup.PlayerLocalId)
            return true;
        if (!string.IsNullOrEmpty(AuthManager.UserId)
            && string.Equals(userId, AuthManager.UserId, StringComparison.OrdinalIgnoreCase))
            return true;
        string ownerId = MetaPort.Instance?.ownerId;
        return !string.IsNullOrEmpty(ownerId)
            && string.Equals(userId, ownerId, StringComparison.OrdinalIgnoreCase);
    }

    public static void ResolvePlayer(string userId, out PlayerBase player, out string status)
    {
        player = null;
        status = string.Empty;

        if (IsLocalUserId(userId))
        {
            player = PlayerSetup.Instance;
        }
        else if (CVRPlayerManager.Instance != null
            && CVRPlayerManager.Instance.UserIdToPlayerEntity.TryGetValue(userId, out CVRPlayerEntity entity))
        {
            player = entity?.PuppetMaster;
        }

        if (player == null)
        {
            status = "Player Not in World";
            return;
        }
        if (player.IsAvatarBlocked || player.IsAvatarBlockedAlt)
        {
            status = "Avatar Blocked";
            return;
        }
        if (player.IsAvatarHidden && !player.IsLocalPlayer)
        {
            status = "Avatar Hidden";
            return;
        }
        if (player.IsAvatarLoading || !player.IsAvatarLoaded || player.AvatarDescriptor == null)
        {
            status = "Loading Avatar...";
        }
    }

    public static string QuerySelectedQuickMenuPlayerStatus()
    {
        string uuid = QuickMenuAPI.SelectedPlayerID;
        if (string.IsNullOrEmpty(uuid)
            && QuickMenuAPI.SelectedPlayerName != null
            && QuickMenuAPI.SelectedPlayerName == AuthManager.Username)
        {
            uuid = PlayerSetup.PlayerLocalId;
        }
        ResolvePlayer(uuid, out _, out string status);
        return status;
    }

    public static bool CanPreviewPlayer(string userId)
    {
        try
        {
            ResolvePlayer(userId, out PlayerBase player, out string status);
            return player != null && string.IsNullOrEmpty(status) && player.AvatarDescriptor != null;
        }
        catch
        {
            return false;
        }
    }

    public static bool ResolveLoadedAvatar(string avatarId, out PlayerBase wearer)
    {
        wearer = null;
        if (string.IsNullOrWhiteSpace(avatarId))
            return false;

        try
        {
            PlayerSetup local = PlayerSetup.Instance;
            if (local != null
                && local.IsAvatarLoaded
                && !local.IsAvatarLoading
                && local.AvatarDescriptor != null
                && !local.IsAvatarBlocked
                && !local.IsAvatarBlockedAlt
                && string.Equals(MetaPort.Instance?.currentAvatarGuid, avatarId, StringComparison.OrdinalIgnoreCase))
            {
                wearer = local;
                return true;
            }

            var players = CVRPlayerManager.Instance?.NetworkPlayers;
            if (players != null)
            {
                for (int i = 0; i < players.Count; i++)
                {
                    CVRPlayerEntity ent = players[i];
                    PuppetMaster pm = ent?.PuppetMaster;
                    if (pm == null || !pm.IsAvatarLoaded || pm.IsAvatarLoading || pm.AvatarDescriptor == null)
                        continue;
                    if (pm.IsAvatarBlocked || pm.IsAvatarBlockedAlt || pm.IsAvatarHidden)
                        continue;

                    string wornId = ent.ContentMetadata.AssetId ?? pm.AvatarMetadata.AssetId;
                    if (!string.IsNullOrEmpty(wornId)
                        && string.Equals(wornId, avatarId, StringComparison.OrdinalIgnoreCase))
                    {
                        wearer = pm;
                        return true;
                    }
                }
            }
        }
        catch
        {
            // Ignore transient player list mutations.
        }
        return false;
    }

    public static bool CanPreviewAvatar(string avatarId)
    {
        return ResolveLoadedAvatar(avatarId, out _);
    }

    public static void DisposeInstance()
    {
        if (_instance != null)
        {
            _instance.CleanupRig();
            Destroy(_instance.gameObject);
            _instance = null;
        }
    }

    private void Awake()
    {
        _avatarHolder = new GameObject("AvatarHolder").transform;
        _avatarHolder.SetParent(transform, worldPositionStays: false);

        _cameraTransform = new GameObject("PreviewCamera").transform;
        _cameraTransform.SetParent(transform, worldPositionStays: false);
        _cameraTransform.localRotation = Quaternion.Euler(0f, 180f, 0f);

        _camera = _cameraTransform.gameObject.AddComponent<Camera>();
        _camera.enabled = false;
        _camera.clearFlags = CameraClearFlags.Color;
        _camera.backgroundColor = Color.clear;
        _camera.cullingMask = PreviewLayerMask;
        _camera.fieldOfView = FieldOfView;
        _camera.nearClipPlane = 0.01f;
        _camera.farClipPlane = 100f;
        _camera.allowHDR = false;
        _camera.allowMSAA = false;
        _camera.useOcclusionCulling = false;

        _cameraTexture = new RenderTexture(512, 768, 24, RenderTextureFormat.ARGBHalf, RenderTextureReadWrite.Linear)
        {
            name = "LumenicPlayerPreviewRT"
        };
        _camera.targetTexture = _cameraTexture;

        var lightGo = new GameObject("PreviewLight");
        lightGo.transform.SetParent(transform, worldPositionStays: false);
        lightGo.transform.localRotation = Quaternion.Euler(35f, 150f, 0f);
        _light = lightGo.AddComponent<Light>();
        _light.enabled = false;
        _light.type = LightType.Directional;
        _light.cullingMask = PreviewLayerMask;
    }

    public void SyncTarget(string targetKey, bool active)
    {
        if (!active || string.IsNullOrEmpty(targetKey) || IsLocalTarget(targetKey))
        {
            SetRendering(false);
            return;
        }

        PlayerBase player = null;
        if (targetKey.StartsWith("player:", StringComparison.Ordinal))
        {
            ResolvePlayer(targetKey.Substring("player:".Length), out player, out string status);
            if (!string.IsNullOrEmpty(status))
                player = null;
        }
        else if (targetKey.StartsWith("avatar:", StringComparison.Ordinal))
        {
            ResolveLoadedAvatar(targetKey.Substring("avatar:".Length), out player);
        }

        CVRAvatar source = player?.AvatarDescriptor;
        if (source == null)
        {
            SetRendering(false);
            return;
        }

        _inactiveSince = -1f;
        bool targetChanged = _activeTarget != targetKey || !ReferenceEquals(_sourceAvatar, source);
        if (targetChanged)
        {
            _activeTarget = targetKey;
            _sourceAvatar = source;
            DestroyPreviewAvatar();
            SpawnPreviewAvatar(source, player.AnimatorManager);
        }

        SetRendering(true);
    }

    public void Rotate(float delta, bool isDragging)
    {
        if (float.IsNaN(delta) || float.IsInfinity(delta))
            return;

        _isDragging = isDragging;
        _dragDelta += (0f - delta) * DragDegrees;
    }

    private void SetRendering(bool state)
    {
        if (_rendering == state)
            return;

        _rendering = state;
        _camera.enabled = state && _previewAvatar != null;
        _light.enabled = state && _previewAvatar != null;
        if (!state)
        {
            _isDragging = false;
            _dragDelta = 0f;
            _inactiveSince = Time.unscaledTime;
            if (_cameraTexture != null)
                _cameraTexture.Release();
        }
    }

    private void SpawnPreviewAvatar(CVRAvatar sourceAvatar, AvatarAnimatorManager sourceAnimator)
    {
        if (_isLoading || sourceAvatar == null || CVRObjectLoader.Instance == null)
            return;

        _isLoading = true;
        StartCoroutine(CVRObjectLoader.Instance.InstantiateAvatarPreview(sourceAvatar, _avatarHolder, previewAvatar =>
        {
            OnPreviewInstantiated(sourceAvatar, sourceAnimator, previewAvatar);
        }));
    }

    private void OnPreviewInstantiated(CVRAvatar sourceAvatar, AvatarAnimatorManager sourceAnimator, CVRAvatar previewAvatar)
    {
        _isLoading = false;
        if (previewAvatar == null)
            return;

        if (!_rendering || !ReferenceEquals(sourceAvatar, _sourceAvatar))
        {
            Destroy(previewAvatar.gameObject);
            if (_rendering && _sourceAvatar != null)
            {
                SpawnPreviewAvatar(_sourceAvatar, sourceAnimator);
            }
            return;
        }

        _previewAvatar = previewAvatar;
        _previewRenderers = previewAvatar.GetComponentsInChildren<Renderer>(includeInactive: true);
        _dragAngle = 0f;
        _dragVelocity = 0f;
        _dragDelta = 0f;

        try
        {
            Animator anim = previewAvatar.GetComponent<Animator>();
            if (anim != null)
            {
                anim.enabled = true;
                anim.cullingMode = AnimatorCullingMode.AlwaysAnimate;
                anim.applyRootMotion = false;
                _animatorManager.SetupManager(anim, previewAvatar, isLocalPlayer: false);
                _animatorManager.ResetCoreParameters();
                _animatorManager.IsLocal = true;
                _animatorManager.Grounded = true;
                _animatorManager.IsAnimatorEnabled = true;

                if (sourceAnimator != null)
                {
                    foreach (string key in sourceAnimator.Parameters.Keys)
                    {
                        if (sourceAnimator.GetParameter(key, out float fVal))
                            _animatorManager.SetParameter(key, fVal);
                        else if (sourceAnimator.GetParameter(key, out int iVal))
                            _animatorManager.SetParameter(key, iVal);
                        else if (sourceAnimator.GetParameter(key, out bool bVal))
                            _animatorManager.SetParameter(key, bVal);
                    }
                }
            }
        }
        catch (Exception ex)
        {
            MelonLogger.Warning($"[LumenicMenu] PlayerPreviewRig animator setup warning: {ex.Message}");
        }

        _camera.enabled = _rendering;
        _light.enabled = _rendering;
        FrameAvatar();
    }

    private void LateUpdate()
    {
        if (!_rendering)
        {
            if (_previewAvatar != null && _inactiveSince > 0f && (Time.unscaledTime - _inactiveSince) >= DestroyDelaySecs)
            {
                DestroyPreviewAvatar();
                _sourceAvatar = null;
                _activeTarget = string.Empty;
                _inactiveSince = -1f;
            }
            return;
        }

        if (_previewAvatar == null)
            return;

        float dt = Mathf.Max(Time.unscaledDeltaTime, 0.0001f);
        if (_isDragging)
        {
            _dragVelocity = Mathf.Lerp(_dragVelocity, _dragDelta / dt, 1f - Mathf.Exp(-DragResponse * dt));
        }
        else
        {
            _dragAngle += _dragVelocity * dt;
            _dragVelocity *= Mathf.Exp(-DragDamping * dt);
        }
        _dragAngle += _dragDelta;
        _dragDelta = 0f;

        _avatarHolder.localRotation = Quaternion.Euler(0f, _dragAngle, 0f);
        FrameAvatar();
    }

    private void FrameAvatar()
    {
        if (_previewAvatar == null)
            return;

        CVRAvatar.RestMeasurements rest = _previewAvatar.restMeasurements;
        float scaleRatio = rest.IsValid && rest.Scale.y > 0.0001f
            ? (_previewAvatar.transform.localScale.y / rest.Scale.y)
            : 1f;
        float eyeHeight = Mathf.Max(_previewAvatar.viewPosition.y * scaleRatio, 0.01f);
        float maxTop = 0f;
        if (_previewRenderers != null)
        {
            for (int i = 0; i < _previewRenderers.Length; i++)
            {
                Renderer r = _previewRenderers[i];
                if (r != null && r.enabled && r.gameObject.activeInHierarchy)
                {
                    maxTop = Mathf.Max(maxTop, GetTop(r) - transform.position.y);
                }
            }
        }

        float framedH = eyeHeight * Mathf.Clamp(maxTop / eyeHeight, 1.084f, MaxTopRatio);
        float z = framedH * FramingPadding * 0.5f / Mathf.Tan(FieldOfView * 0.5f * Mathf.Deg2Rad);
        _cameraTransform.localPosition = new Vector3(0f, framedH * 0.5f, z);
    }

    private static float GetTop(Renderer r)
    {
        if (r is MeshRenderer)
            return r.bounds.max.y;
        if (r is SkinnedMeshRenderer smr && smr.sharedMesh != null)
        {
            Bounds b = smr.sharedMesh.bounds;
            Vector4 row = smr.transform.localToWorldMatrix.GetRow(1);
            return Vector3.Dot(row, b.center)
                + row.w
                + Mathf.Abs(row.x) * b.extents.x
                + Mathf.Abs(row.y) * b.extents.y
                + Mathf.Abs(row.z) * b.extents.z;
        }
        return float.MinValue;
    }

    private void DestroyPreviewAvatar()
    {
        if (_camera != null) _camera.enabled = false;
        if (_light != null) _light.enabled = false;
        _animatorManager.ResetManager();
        if (_previewAvatar != null)
        {
            Destroy(_previewAvatar.gameObject);
            _previewAvatar = null;
        }
        _previewRenderers = null;
    }

    private void CleanupRig()
    {
        SetRendering(false);
        DestroyPreviewAvatar();
        if (_camera != null) _camera.targetTexture = null;
        if (_cameraTexture != null)
        {
            _cameraTexture.Release();
            Destroy(_cameraTexture);
            _cameraTexture = null;
        }
    }

    private void OnDestroy()
    {
        CleanupRig();
        if (_instance == this)
            _instance = null;
    }
}
