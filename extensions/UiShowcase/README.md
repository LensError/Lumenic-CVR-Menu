# LumenicUiShowcase

Reference MelonLoader mod demonstrating every `LumenicMenu.Api` surface, `UiBuilder` widget, procedural texture upload, overlay, section override, and global theme patch in one small project.

## File Map
- `UiShowcaseMod.cs`: Soft-dependency setup (`MelonOptionalDependencies` + `NoInlining` boundary) and unload cleanup (`LumenicApi.UnregisterMod`).
- `ShowcaseUi.cs`: Main Menu tab, Quick Menu tab, shared Extensions Hub entry (`LumenicHubs.Mods`), `HeaderActions`, `PushSubPage`/`PopSubPage`, and lifecycle events (`OnReady`, `OnQuickMenuToggled`, `OnMainMenuToggled`).
- `ShowcasePages.cs`: Interactive demos for all `UiBuilder` controls, layout/styling primitives (`ExtStyle`, `ExtDropdownStyle`), media widgets, modals/keyboard/toasts, and opt-in `LumenicApi.Sections` + `LumenicApi.Theme` toggles.
- `ShowcaseTextures.cs`: `RegisterRawRgba` and `RegisterTexture2D` procedural texture uploads.

## Build
```powershell
dotnet build extensions/UiShowcase/LumenicUiShowcase.csproj -c Release
```
The compiled `LumenicUiShowcase.dll` copies automatically to `$(CVRPath)/Mods/` when `CVRPath` is set.
