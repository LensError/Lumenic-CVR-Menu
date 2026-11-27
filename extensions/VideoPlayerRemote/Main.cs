using System;
using System.Linq;
using System.Reflection;
using System.Runtime.CompilerServices;
using MelonLoader;
using UnityEngine;

[assembly: MelonInfo(typeof(VideoPlayerRemote.VideoPlayerRemoteMod), "VideoPlayerRemote", "1.0.0", "LensError")]
[assembly: MelonGame(null, "ChilloutVR")]
[assembly: MelonOptionalDependencies("LumenicMenu", "BTKUILib")]

namespace VideoPlayerRemote
{
    public class VideoPlayerRemoteMod : MelonMod
    {
        internal static MelonLogger.Instance Log;
        internal const string IconModName = "VideoRemote";
        internal const string TabIconName = "video_remote";

        private float _nextUiRefreshTime;
        private bool _usingLumenic;
        private bool _usingBtkUi;
        private bool _cleanedUp;

        public override void OnInitializeMelon()
        {
            Log = LoggerInstance;
            GameEvents.Init(HarmonyInstance);
        }

        public override void OnLateInitializeMelon()
        {
            Settings.Init();
            LibraryStore.Load();
            VideoPlayerTracker.Init();
            LocalScreenManager.Init();
            HudScreenManager.Init();

            if (MelonBase.RegisteredMelons.Any(m => m.Info.Name == "LumenicMenu"))
            {
                _usingLumenic = LumenicRemoteUI.TryInit();
            }

            if (!_usingLumenic)
            {
                if (MelonBase.RegisteredMelons.Any(m => m.Info.Name == "BTKUILib"))
                {
                    _usingBtkUi = TryInitBtkUi();
                }
                else
                {
                    Log?.Warning("Neither LumenicMenu nor BTKUILib is installed; VideoPlayerRemote UI will remain inactive.");
                }
            }
        }

        public override void OnUpdate()
        {
            if (Time.unscaledTime < _nextUiRefreshTime)
                return;

            _nextUiRefreshTime = Time.unscaledTime + 0.5f;
            VideoPlayerTracker.Tick();
            LocalScreenManager.Tick();
            HudScreenManager.Tick();
            if (_usingLumenic)
                LumenicRemoteUI.Tick();
            else if (_usingBtkUi)
                TickBtkUi();
        }

        public override void OnLateUpdate()
        {
            LocalScreenManager.LateUpdateTick();
            HudScreenManager.LateUpdateTick();
        }

        public override void OnDeinitializeMelon() => Cleanup();
        public override void OnApplicationQuit() => Cleanup();

        private void Cleanup()
        {
            if (_cleanedUp)
                return;
            _cleanedUp = true;

            if (_usingLumenic)
            {
                LumenicRemoteUI.Shutdown();
                _usingLumenic = false;
            }
            else if (_usingBtkUi)
            {
                ShutdownBtkUi();
                _usingBtkUi = false;
            }

            HudScreenManager.Shutdown();
            LocalScreenManager.Shutdown();
            VideoPlayerTracker.Shutdown();
            LibraryStore.Shutdown();
            GameEvents.Shutdown();
        }

        [MethodImpl(MethodImplOptions.NoInlining)]
        private static bool TryInitBtkUi()
        {
            try
            {
                PrepareBtkIcons();
                RemoteUI.Init();
                return true;
            }
            catch (Exception ex)
            {
                Log?.Warning($"Failed to initialize BTKUILib UI: {ex.Message}");
                return false;
            }
        }

        [MethodImpl(MethodImplOptions.NoInlining)]
        private static void TickBtkUi() => RemoteUI.Tick();

        [MethodImpl(MethodImplOptions.NoInlining)]
        private static void ShutdownBtkUi() => RemoteUI.Shutdown();

        [MethodImpl(MethodImplOptions.NoInlining)]
        private static void PrepareBtkIcons()
        {
            try
            {
                if (BTKUILib.QuickMenuAPI.DoesIconExist(IconModName, TabIconName))
                    return;

                using var stream = Assembly.GetExecutingAssembly()
                    .GetManifestResourceStream("VideoPlayerRemote.resources.video_remote.png");
                if (stream != null)
                {
                    BTKUILib.QuickMenuAPI.PrepareIcon(IconModName, TabIconName, stream);
                }
            }
            catch (Exception ex)
            {
                Log?.Warning($"Failed to prepare QuickMenu icon: {ex.Message}");
            }
        }
    }
}
