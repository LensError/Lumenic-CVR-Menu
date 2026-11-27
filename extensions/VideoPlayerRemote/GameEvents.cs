using System;
using System.Reflection;
using ABI.CCK.Components;
using ABI_RC.Core.IO;
using ABI_RC.Systems.GameEventSystem;
using HarmonyLib;

namespace VideoPlayerRemote
{
    internal static class GameEvents
    {
        private static HarmonyLib.Harmony _harmony;

        public static event Action<CVRVideoPlayer> OnVideoPlayerRegistered;
        public static event Action<CVRVideoPlayer> OnVideoPlayerUnregistered;
        public static event Action OnVideoPlayersCleared;
        public static event Action<CVRVideoPlayer> OnVideoFinishedPlaying;
        public static event Action<CVRVideoPlayer, string, bool> OnVideoUrlChanged;
        public static event Action<CVRVideoPlayer> OnVideoUrlCleared;
        public static event Action OnWorldLeave;

        public static void Init(HarmonyLib.Harmony harmony)
        {
            _harmony = harmony;
            try
            {
                CVRGameEventSystem.World.OnWorldUnload.AddListener(HandleWorldUnload);

                PatchPostfix(harmony, typeof(CVRWorld), nameof(CVRWorld.RegisterVideoPlayer),
                    typeof(GameEvents), nameof(Postfix_RegisterVideoPlayer));

                PatchPostfix(harmony, typeof(CVRWorld), nameof(CVRWorld.UnregisterVideoPlayer),
                    typeof(GameEvents), nameof(Postfix_UnregisterVideoPlayer));

                PatchPostfix(harmony, typeof(CVRWorld), nameof(CVRWorld.ClearRegisteredVideoPlayers),
                    typeof(GameEvents), nameof(Postfix_ClearRegisteredVideoPlayers));

                PatchPostfix(harmony, typeof(CVRVideoPlayer), "FinishedPlaying",
                    typeof(GameEvents), nameof(Postfix_FinishedPlaying));

                PatchPostfix(harmony, typeof(CVRVideoPlayer), nameof(CVRVideoPlayer.SetVideoUrl),
                    typeof(GameEvents), nameof(Postfix_SetVideoUrl));

                PatchPostfix(harmony, typeof(CVRVideoPlayer), nameof(CVRVideoPlayer.ClearVideoUrl),
                    typeof(GameEvents), nameof(Postfix_ClearVideoUrl));
            }
            catch (Exception ex)
            {
                VideoPlayerRemoteMod.Log?.Error($"Failed to initialize Harmony patches: {ex}");
            }
        }

        public static void Shutdown()
        {
            try
            {
                CVRGameEventSystem.World.OnWorldUnload.RemoveListener(HandleWorldUnload);
            }
            catch { }

            try
            {
                _harmony?.UnpatchSelf();
            }
            catch (Exception ex)
            {
                VideoPlayerRemoteMod.Log?.Warning($"Failed to unpatch Harmony: {ex.Message}");
            }
            _harmony = null;
            OnVideoPlayerRegistered = null;
            OnVideoPlayerUnregistered = null;
            OnVideoPlayersCleared = null;
            OnVideoFinishedPlaying = null;
            OnVideoUrlChanged = null;
            OnVideoUrlCleared = null;
            OnWorldLeave = null;
        }

        private static void HandleWorldUnload(string _, CVRObjectLoader.WorldLoadIntent __)
        {
            try
            {
                OnWorldLeave?.Invoke();
            }
            catch (Exception ex)
            {
                VideoPlayerRemoteMod.Log?.Error($"Error in HandleWorldUnload: {ex.Message}");
            }
        }

        private static void PatchPostfix(
            HarmonyLib.Harmony harmony,
            Type targetType,
            string targetMethodName,
            Type patchType,
            string patchMethodName)
        {
            MethodInfo target = AccessTools.Method(targetType, targetMethodName);
            MethodInfo patch = AccessTools.Method(patchType, patchMethodName);
            if (target == null || patch == null)
            {
                VideoPlayerRemoteMod.Log?.Warning($"Could not find target/patch method for {targetType.Name}.{targetMethodName}");
                return;
            }
            harmony.Patch(target, postfix: new HarmonyMethod(patch));
        }

        private static void Postfix_RegisterVideoPlayer(CVRVideoPlayer videoPlayer)
        {
            try
            {
                if (videoPlayer != null)
                    OnVideoPlayerRegistered?.Invoke(videoPlayer);
            }
            catch (Exception ex)
            {
                VideoPlayerRemoteMod.Log?.Error($"Error in Postfix_RegisterVideoPlayer: {ex.Message}");
            }
        }

        private static void Postfix_UnregisterVideoPlayer(CVRVideoPlayer videoPlayer)
        {
            try
            {
                if (videoPlayer != null)
                    OnVideoPlayerUnregistered?.Invoke(videoPlayer);
            }
            catch (Exception ex)
            {
                VideoPlayerRemoteMod.Log?.Error($"Error in Postfix_UnregisterVideoPlayer: {ex.Message}");
            }
        }

        private static void Postfix_ClearRegisteredVideoPlayers()
        {
            try
            {
                OnVideoPlayersCleared?.Invoke();
            }
            catch (Exception ex)
            {
                VideoPlayerRemoteMod.Log?.Error($"Error in Postfix_ClearRegisteredVideoPlayers: {ex.Message}");
            }
        }

        private static void Postfix_FinishedPlaying(CVRVideoPlayer __instance)
        {
            try
            {
                if (__instance != null)
                    OnVideoFinishedPlaying?.Invoke(__instance);
            }
            catch (Exception ex)
            {
                VideoPlayerRemoteMod.Log?.Error($"Error in Postfix_FinishedPlaying: {ex.Message}");
            }
        }

        private static void Postfix_SetVideoUrl(CVRVideoPlayer __instance, string url, bool fromNetwork)
        {
            try
            {
                if (__instance != null && !string.IsNullOrEmpty(url))
                    OnVideoUrlChanged?.Invoke(__instance, url, fromNetwork);
            }
            catch (Exception ex)
            {
                VideoPlayerRemoteMod.Log?.Error($"Error in Postfix_SetVideoUrl: {ex.Message}");
            }
        }

        private static void Postfix_ClearVideoUrl(CVRVideoPlayer __instance)
        {
            try
            {
                if (__instance != null)
                    OnVideoUrlCleared?.Invoke(__instance);
            }
            catch (Exception ex)
            {
                VideoPlayerRemoteMod.Log?.Error($"Error in Postfix_ClearVideoUrl: {ex.Message}");
            }
        }
    }
}
