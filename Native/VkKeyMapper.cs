using cohtml.Net;

namespace LumenicMenu.Native;

/// <summary>
/// Translates Windows VK_* codes from Cohtml KeyEvent into Lumenic logical key IDs.
/// Ordinary printable characters are intentionally omitted here and routed via OnCharEvent
/// to prevent double-typing inside Lumenic's TextEditState.
/// </summary>
internal static class VkKeyMapper
{
    public const uint ModShift = 1u << 0;
    public const uint ModCtrl = 1u << 1;
    public const uint ModAlt = 1u << 2;
    public const uint ModSuper = 1u << 3;

    private const uint KeyTab = 1;
    private const uint KeyEnter = 2;
    private const uint KeySpace = 3;
    private const uint KeyEscape = 4;
    private const uint KeyBackspace = 5;
    private const uint KeyDelete = 6;
    private const uint KeyInsert = 7;
    private const uint KeyLeft = 8;
    private const uint KeyRight = 9;
    private const uint KeyUp = 10;
    private const uint KeyDown = 11;
    private const uint KeyHome = 12;
    private const uint KeyEnd = 13;
    private const uint KeyPageUp = 14;
    private const uint KeyPageDown = 15;
    private const uint KeyF1 = 16;
    private const uint KeyF2 = 17;
    private const uint KeyF3 = 18;
    private const uint KeyF4 = 19;
    private const uint KeyF5 = 20;
    private const uint KeyF6 = 21;
    private const uint KeyF7 = 22;
    private const uint KeyF8 = 23;
    private const uint KeyF9 = 24;
    private const uint KeyF10 = 25;
    private const uint KeyF11 = 26;
    private const uint KeyF12 = 27;
    private const uint KeyCharBase = 0x1000;

    public static uint PackModifiers(EventModifiersState mods)
    {
        if (mods == null)
            return 0u;

        uint flags = 0u;
        if (mods.IsShiftDown) flags |= ModShift;
        if (mods.IsCtrlDown) flags |= ModCtrl;
        if (mods.IsAltDown) flags |= ModAlt;
        if (mods.IsMetaDown) flags |= ModSuper;
        return flags;
    }

    public static bool TryMapKeyDown(int vkCode, uint modFlags, bool isTextInputFocused, out uint lumenicKey)
    {
        lumenicKey = 0;

        // Non-printable navigation, editing, and function keys.
        switch (vkCode)
        {
            case 0x08: lumenicKey = KeyBackspace; return true; // VK_BACK
            case 0x09: lumenicKey = KeyTab; return true;       // VK_TAB
            case 0x0D: lumenicKey = KeyEnter; return true;     // VK_RETURN
            case 0x1B: lumenicKey = KeyEscape; return true;    // VK_ESCAPE
            case 0x20:
                // When text input is focused, OnCharEvent already commits ' '.
                // Forwarding Key::Space too would insert two spaces.
                if (!isTextInputFocused)
                {
                    lumenicKey = KeySpace;
                    return true;
                }
                return false;
            case 0x21: lumenicKey = KeyPageUp; return true;    // VK_PRIOR
            case 0x22: lumenicKey = KeyPageDown; return true;  // VK_NEXT
            case 0x23: lumenicKey = KeyEnd; return true;       // VK_END
            case 0x24: lumenicKey = KeyHome; return true;      // VK_HOME
            case 0x25: lumenicKey = KeyLeft; return true;      // VK_LEFT
            case 0x26: lumenicKey = KeyUp; return true;        // VK_UP
            case 0x27: lumenicKey = KeyRight; return true;     // VK_RIGHT
            case 0x28: lumenicKey = KeyDown; return true;      // VK_DOWN
            case 0x2D: lumenicKey = KeyInsert; return true;    // VK_INSERT
            case 0x2E: lumenicKey = KeyDelete; return true;    // VK_DELETE
            case 0x70: lumenicKey = KeyF1; return true;
            case 0x71: lumenicKey = KeyF2; return true;
            case 0x72: lumenicKey = KeyF3; return true;
            case 0x73: lumenicKey = KeyF4; return true;
            case 0x74: lumenicKey = KeyF5; return true;
            case 0x75: lumenicKey = KeyF6; return true;
            case 0x76: lumenicKey = KeyF7; return true;
            case 0x77: lumenicKey = KeyF8; return true;
            case 0x78: lumenicKey = KeyF9; return true;
            case 0x79: lumenicKey = KeyF10; return true;
            case 0x7A: lumenicKey = KeyF11; return true;
            case 0x7B: lumenicKey = KeyF12; return true;
        }

        // Ctrl / Alt / Super shortcut chords (e.g. Ctrl+A, Ctrl+C, Ctrl+V, Ctrl+X, Ctrl+Z, Ctrl+Y).
        bool hasShortcutMod = (modFlags & (ModCtrl | ModAlt | ModSuper)) != 0;
        if (hasShortcutMod && vkCode >= 0x41 && vkCode <= 0x5A)
        {
            char lower = (char)('a' + (vkCode - 0x41));
            lumenicKey = KeyCharBase + lower;
            return true;
        }

        return false;
    }
}
