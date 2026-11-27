using System.Linq;
using System.Runtime.CompilerServices;
using LumenicMenu.Api;
using MelonLoader;

[assembly: MelonInfo(typeof(LumenicUiShowcase.UiShowcaseMod), "LumenicUiShowcase", "1.0.0", "LensError")]
[assembly: MelonGame("ChilloutVR", "ChilloutVR")]
[assembly: MelonOptionalDependencies("LumenicMenu")]

namespace LumenicUiShowcase;

// Entry point demonstrating safe optional integration with LumenicMenu.
// All LumenicMenu.Api calls live in [MethodImpl(MethodImplOptions.NoInlining)] helpers
// so the assembly still loads cleanly when LumenicMenu is not installed.
public sealed class UiShowcaseMod : MelonMod
{
    private ShowcaseUi _ui;
    private bool _lumenicBound;

    public override void OnLateInitializeMelon()
    {
        if (MelonBase.RegisteredMelons.Any(m => m.Info.Name == "LumenicMenu"))
        {
            InitializeLumenic();
        }
        else
        {
            MelonLogger.Warning("[UiShowcase] LumenicMenu not found; showcase UI disabled.");
        }
    }

    [MethodImpl(MethodImplOptions.NoInlining)]
    private void InitializeLumenic()
    {
        if (!LumenicApi.IsAvailable)
            return;

        _ui = new ShowcaseUi();
        _ui.Register();
        _lumenicBound = true;
        MelonLogger.Msg("[UiShowcase] Registered Main Menu tab, Quick Menu tab, and Extensions Hub entry.");
    }

    public override void OnDeinitializeMelon() => Cleanup();
    public override void OnApplicationQuit() => Cleanup();

    private void Cleanup()
    {
        if (!_lumenicBound)
            return;

        _lumenicBound = false;
        CleanupLumenic();
    }

    [MethodImpl(MethodImplOptions.NoInlining)]
    private void CleanupLumenic()
    {
        _ui?.Dispose();
        _ui = null;
        LumenicApi.UnregisterMod(ShowcaseUi.ModId);
    }
}
