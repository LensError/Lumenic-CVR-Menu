using System;
using System.IO;
using System.Reflection;
using System.Runtime.InteropServices;
using System.Security.Cryptography;
using System.Text;
using MelonLoader;
using MelonLoader.Utils;

namespace LumenicMenu.Native;

/// <summary>
/// Dynamic Win32 loader and FFI wrapper for lumenic_cvr_ui.dll.
/// Must be called strictly on Unity's main thread to match WGL context thread affinity.
/// </summary>
internal sealed class NativeBridge : IDisposable
{
    public const uint ExpectedAbiVersion = 5;
    public const uint SurfaceMain = 0;
    public const uint SurfaceQuick = 1;

    public const uint StepFlagRendered = 1u << 0;
    public const uint StepFlagTextFocused = 1u << 1;
    public const uint StepFlagPointerOverUi = 1u << 2;
    public const uint StepFlagHasActions = 1u << 3;
    public const uint StepFlagErrorOrPanic = 1u << 31;

    [DllImport("kernel32.dll", CharSet = CharSet.Unicode, SetLastError = true)]
    private static extern IntPtr LoadLibraryW(string lpFileName);

    [DllImport("kernel32.dll", CharSet = CharSet.Ansi, SetLastError = true)]
    private static extern IntPtr GetProcAddress(IntPtr hModule, string procName);

    [DllImport("kernel32.dll", SetLastError = true)]
    private static extern bool FreeLibrary(IntPtr hModule);

    [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
    private delegate uint AbiVersionFn();

    [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
    private delegate UIntPtr LastErrorFn(IntPtr dstPtr, UIntPtr dstCap);

    [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
    private delegate IntPtr CreateFn(uint mmW, uint mmH, uint qmW, uint qmH, float scale);

    [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
    private delegate void DestroyFn(IntPtr handle);

    [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
    [return: MarshalAs(UnmanagedType.I1)]
    private delegate bool ResizeFn(IntPtr handle, uint surfaceId, uint w, uint h, float scale);

    [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
    private delegate void PointerMoveFn(IntPtr handle, uint surfaceId, float x, float y);

    [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
    private delegate void PointerButtonFn(IntPtr handle, uint surfaceId, uint button);

    [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
    private delegate void PointerScrollFn(IntPtr handle, uint surfaceId, float dx, float dy);

    [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
    private delegate void SurfaceOnlyFn(IntPtr handle, uint surfaceId);

    [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
    private delegate void KeyDownFn(IntPtr handle, uint surfaceId, uint keyCode, uint modFlags);

    [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
    private delegate void TextInputFn(IntPtr handle, uint surfaceId, IntPtr utf8Ptr, UIntPtr len);

    [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
    [return: MarshalAs(UnmanagedType.I1)]
    private delegate bool CommitVirtualKeyboardFn(
        IntPtr handle,
        uint surfaceId,
        IntPtr fieldIdPtr,
        UIntPtr fieldIdLen,
        IntPtr textPtr,
        UIntPtr textLen);

    [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
    [return: MarshalAs(UnmanagedType.I1)]
    private delegate bool UploadImageBytesFn(
        IntPtr handle,
        uint surfaceId,
        IntPtr keyPtr,
        UIntPtr keyLen,
        IntPtr bytesPtr,
        UIntPtr bytesLen);

    [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
    [return: MarshalAs(UnmanagedType.I1)]
    private delegate bool UploadRgbaTextureFn(
        IntPtr handle,
        uint surfaceId,
        IntPtr keyPtr,
        UIntPtr keyLen,
        uint width,
        uint height,
        IntPtr rgbaPtr,
        UIntPtr rgbaLen);

    [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
    [return: MarshalAs(UnmanagedType.I1)]
    private delegate bool RegisterFontFn(
        IntPtr namePtr,
        UIntPtr nameLen,
        IntPtr bytesPtr,
        UIntPtr bytesLen);

    [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
    [return: MarshalAs(UnmanagedType.I1)]
    private delegate bool SetDefaultFontFamilyFn(IntPtr namePtr, UIntPtr nameLen);

    [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
    [return: MarshalAs(UnmanagedType.I1)]
    private delegate bool PushStateJsonFn(IntPtr handle, IntPtr utf8Ptr, UIntPtr len);

    [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
    [return: MarshalAs(UnmanagedType.I1)]
    private delegate bool PushTelemetryFn(
        IntPtr handle,
        int fps,
        int gamePing,
        int voicePing,
        IntPtr clockPtr,
        UIntPtr clockLen,
        IntPtr datePtr,
        UIntPtr dateLen);

    [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
    private delegate uint StepFn(IntPtr handle, uint surfaceId, float dtSecs, [MarshalAs(UnmanagedType.I1)] bool forceRedraw);

    [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
    [return: MarshalAs(UnmanagedType.I1)]
    private delegate bool ReadPixelsFn(IntPtr handle, uint surfaceId, IntPtr dstPtr, UIntPtr dstLen);

    [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
    [return: MarshalAs(UnmanagedType.I1)]
    private delegate bool PollActionsJsonFn(IntPtr handle, out IntPtr outPtr, out UIntPtr outLen);

    private IntPtr _module = IntPtr.Zero;
    private IntPtr _handle = IntPtr.Zero;
    private bool _disposed;

    private AbiVersionFn _abiVersion;
    private LastErrorFn _lastError;
    private CreateFn _create;
    private DestroyFn _destroy;
    private ResizeFn _resize;
    private PointerMoveFn _pointerMove;
    private PointerButtonFn _pointerDown;
    private PointerButtonFn _pointerUp;
    private PointerScrollFn _pointerScroll;
    private SurfaceOnlyFn _pointerLeave;
    private SurfaceOnlyFn _pointerCancel;
    private KeyDownFn _keyDown;
    private TextInputFn _textInput;
    private SurfaceOnlyFn _clearFocus;
    private CommitVirtualKeyboardFn _commitVirtualKeyboard;
    private UploadImageBytesFn _uploadImageBytes;
    private UploadRgbaTextureFn _uploadRgbaTexture;
    private RegisterFontFn _registerFont;
    private SetDefaultFontFamilyFn _setDefaultFontFamily;
    private PushStateJsonFn _pushStateJson;
    private PushStateJsonFn _pushExtensionsJson;
    private PushTelemetryFn _pushTelemetry;
    private StepFn _step;
    private ReadPixelsFn _readPixels;
    private PollActionsJsonFn _pollActionsJson;

    private byte[] _utf8Scratch = new byte[4096];

    public bool IsHealthy => !_disposed && _module != IntPtr.Zero && _handle != IntPtr.Zero;
    public string LastFailureReason { get; private set; } = string.Empty;

    public static string UserDataDir => Path.Combine(MelonEnvironment.UserDataDirectory, "LumenicMenu");
    public static string DefaultDllPath => Path.Combine(UserDataDir, "lumenic_cvr_ui.dll");

    public bool Load(uint mmW, uint mmH, uint qmW, uint qmH, float scale)
    {
        EnsureEmbeddedDllExtracted();
        if (!File.Exists(DefaultDllPath))
        {
            LastFailureReason = $"Native DLL not found at '{DefaultDllPath}'.";
            MelonLogger.Error($"[LumenicMenu] {LastFailureReason}");
            return false;
        }

        UnloadInternal();

        try
        {
            _module = LoadLibraryW(DefaultDllPath);
            if (_module == IntPtr.Zero)
            {
                int winErr = Marshal.GetLastWin32Error();
                LastFailureReason = $"LoadLibraryW failed for '{DefaultDllPath}' (Win32 error {winErr}).";
                MelonLogger.Error($"[LumenicMenu] {LastFailureReason}");
                UnloadInternal();
                return false;
            }

            if (!BindDelegates())
            {
                UnloadInternal();
                return false;
            }

            uint abi = _abiVersion();
            if (abi != ExpectedAbiVersion)
            {
                LastFailureReason = $"ABI mismatch: expected v{ExpectedAbiVersion}, got v{abi}.";
                MelonLogger.Error($"[LumenicMenu] {LastFailureReason}");
                UnloadInternal();
                return false;
            }

            RegisterChilloutVrFonts();

            _handle = _create(Math.Max(1u, mmW), Math.Max(1u, mmH), Math.Max(1u, qmW), Math.Max(1u, qmH), scale);
            if (_handle == IntPtr.Zero)
            {
                LastFailureReason = $"lumenic_cvr_create failed: {GetLastError()}";
                MelonLogger.Error($"[LumenicMenu] {LastFailureReason}");
                UnloadInternal();
                return false;
            }

            LastFailureReason = string.Empty;
            MelonLogger.Msg($"[LumenicMenu] Loaded native UI DLL from '{DefaultDllPath}' (ABI v{abi}).");
            return true;
        }
        catch (Exception ex)
        {
            LastFailureReason = $"Exception loading native DLL: {ex.Message}";
            MelonLogger.Error($"[LumenicMenu] {LastFailureReason}");
            UnloadInternal();
            return false;
        }
    }

    private static void EnsureEmbeddedDllExtracted()
    {
        const string resourceName = "lumenic_cvr_ui.dll";
        try
        {
            Assembly asm = typeof(NativeBridge).Assembly;
            using Stream resStream = asm.GetManifestResourceStream(resourceName);
            if (resStream == null)
            {
                MelonLogger.Warning($"[LumenicMenu] Embedded resource '{resourceName}' not found in assembly.");
                return;
            }

            byte[] embeddedHash;
            using (var sha = SHA256.Create())
            {
                embeddedHash = sha.ComputeHash(resStream);
            }

            if (File.Exists(DefaultDllPath))
            {
                try
                {
                    using var existingStream = File.OpenRead(DefaultDllPath);
                    using var sha = SHA256.Create();
                    byte[] existingHash = sha.ComputeHash(existingStream);
                    if (HashesEqual(embeddedHash, existingHash))
                        return;
                }
                catch
                {
                    // Re-extract below if existing file cannot be read.
                }
            }

            resStream.Position = 0;
            Directory.CreateDirectory(UserDataDir);
            string tmpPath = DefaultDllPath + ".tmp";
            using (var fs = new FileStream(tmpPath, FileMode.Create, FileAccess.Write, FileShare.None))
            {
                resStream.CopyTo(fs);
            }
            if (File.Exists(DefaultDllPath))
                File.Delete(DefaultDllPath);
            File.Move(tmpPath, DefaultDllPath);
            MelonLogger.Msg("[LumenicMenu] Extracted embedded lumenic_cvr_ui.dll.");
        }
        catch (Exception ex)
        {
            MelonLogger.Warning($"[LumenicMenu] Failed to extract embedded native DLL: {ex.Message}");
        }
    }

    private static bool HashesEqual(byte[] a, byte[] b)
    {
        if (a == null || b == null || a.Length != b.Length)
            return false;
        for (int i = 0; i < a.Length; i++)
        {
            if (a[i] != b[i])
                return false;
        }
        return true;
    }

    private bool BindDelegates()
    {
        _abiVersion = Bind<AbiVersionFn>("lumenic_cvr_abi_version");
        _lastError = Bind<LastErrorFn>("lumenic_cvr_last_error");
        _create = Bind<CreateFn>("lumenic_cvr_create");
        _destroy = Bind<DestroyFn>("lumenic_cvr_destroy");
        _resize = Bind<ResizeFn>("lumenic_cvr_resize");
        _pointerMove = Bind<PointerMoveFn>("lumenic_cvr_pointer_move");
        _pointerDown = Bind<PointerButtonFn>("lumenic_cvr_pointer_down");
        _pointerUp = Bind<PointerButtonFn>("lumenic_cvr_pointer_up");
        _pointerScroll = Bind<PointerScrollFn>("lumenic_cvr_pointer_scroll");
        _pointerLeave = Bind<SurfaceOnlyFn>("lumenic_cvr_pointer_leave");
        _pointerCancel = Bind<SurfaceOnlyFn>("lumenic_cvr_pointer_cancel");
        _keyDown = Bind<KeyDownFn>("lumenic_cvr_key_down");
        _textInput = Bind<TextInputFn>("lumenic_cvr_text_input");
        _clearFocus = Bind<SurfaceOnlyFn>("lumenic_cvr_clear_focus");
        _commitVirtualKeyboard = Bind<CommitVirtualKeyboardFn>("lumenic_cvr_commit_virtual_keyboard");
        _uploadImageBytes = Bind<UploadImageBytesFn>("lumenic_cvr_upload_image_bytes");
        _uploadRgbaTexture = Bind<UploadRgbaTextureFn>("lumenic_cvr_upload_rgba_texture");
        _registerFont = Bind<RegisterFontFn>("lumenic_cvr_register_font");
        _setDefaultFontFamily = Bind<SetDefaultFontFamilyFn>("lumenic_cvr_set_default_font_family");
        _pushStateJson = Bind<PushStateJsonFn>("lumenic_cvr_push_state_json");
        _pushExtensionsJson = Bind<PushStateJsonFn>("lumenic_cvr_push_extensions_json");
        _pushTelemetry = Bind<PushTelemetryFn>("lumenic_cvr_push_telemetry");
        _step = Bind<StepFn>("lumenic_cvr_step");
        _readPixels = Bind<ReadPixelsFn>("lumenic_cvr_read_pixels");
        _pollActionsJson = Bind<PollActionsJsonFn>("lumenic_cvr_poll_actions_json");

        return _abiVersion != null
            && _lastError != null
            && _create != null
            && _destroy != null
            && _resize != null
            && _pointerMove != null
            && _pointerDown != null
            && _pointerUp != null
            && _pointerScroll != null
            && _pointerLeave != null
            && _pointerCancel != null
            && _keyDown != null
            && _textInput != null
            && _clearFocus != null
            && _commitVirtualKeyboard != null
            && _uploadImageBytes != null
            && _registerFont != null
            && _setDefaultFontFamily != null
            && _pushStateJson != null
            && _pushTelemetry != null
            && _step != null
            && _readPixels != null
            && _pollActionsJson != null;
    }

    private T Bind<T>(string symbol) where T : Delegate
    {
        IntPtr proc = GetProcAddress(_module, symbol);
        if (proc == IntPtr.Zero)
        {
            LastFailureReason = $"Missing native export '{symbol}'.";
            MelonLogger.Error($"[LumenicMenu] {LastFailureReason}");
            return null;
        }
        return Marshal.GetDelegateForFunctionPointer<T>(proc);
    }

    public string GetLastError()
    {
        if (_lastError == null)
            return LastFailureReason;

        ulong needed = _lastError(IntPtr.Zero, UIntPtr.Zero).ToUInt64();
        if (needed == 0 || needed > 65536)
            return string.Empty;

        byte[] buf = new byte[(int)needed];
        GCHandle pin = GCHandle.Alloc(buf, GCHandleType.Pinned);
        try
        {
            _lastError(pin.AddrOfPinnedObject(), (UIntPtr)buf.Length);
            return Encoding.UTF8.GetString(buf);
        }
        finally
        {
            pin.Free();
        }
    }

    public bool Resize(uint surfaceId, uint width, uint height, float scale)
    {
        if (!IsHealthy) return false;
        return _resize(_handle, surfaceId, width, height, scale);
    }

    public void PointerMove(uint surfaceId, float x, float y)
    {
        if (!IsHealthy) return;
        _pointerMove(_handle, surfaceId, x, y);
    }

    public void PointerDown(uint surfaceId, uint button = 0)
    {
        if (!IsHealthy) return;
        _pointerDown(_handle, surfaceId, button);
    }

    public void PointerUp(uint surfaceId, uint button = 0)
    {
        if (!IsHealthy) return;
        _pointerUp(_handle, surfaceId, button);
    }

    public void PointerScroll(uint surfaceId, float dx, float dy)
    {
        if (!IsHealthy) return;
        _pointerScroll(_handle, surfaceId, dx, dy);
    }

    public void PointerLeave(uint surfaceId)
    {
        if (!IsHealthy) return;
        _pointerLeave(_handle, surfaceId);
    }

    public void PointerCancel(uint surfaceId)
    {
        if (!IsHealthy) return;
        _pointerCancel(_handle, surfaceId);
    }

    public void KeyDown(uint surfaceId, uint keyCode, uint modFlags)
    {
        if (!IsHealthy) return;
        _keyDown(_handle, surfaceId, keyCode, modFlags);
    }

    public void TextInput(uint surfaceId, string text)
    {
        if (!IsHealthy || string.IsNullOrEmpty(text)) return;
        byte[] utf8 = Encoding.UTF8.GetBytes(text);
        GCHandle pin = GCHandle.Alloc(utf8, GCHandleType.Pinned);
        try
        {
            _textInput(_handle, surfaceId, pin.AddrOfPinnedObject(), (UIntPtr)utf8.Length);
        }
        finally
        {
            pin.Free();
        }
    }

    public void ClearFocus(uint surfaceId)
    {
        if (!IsHealthy) return;
        _clearFocus(_handle, surfaceId);
    }

    public bool CommitVirtualKeyboard(uint surfaceId, string fieldId, string text)
    {
        if (!IsHealthy || string.IsNullOrEmpty(fieldId)) return false;
        byte[] fieldUtf8 = Encoding.UTF8.GetBytes(fieldId);
        byte[] textUtf8 = string.IsNullOrEmpty(text) ? Array.Empty<byte>() : Encoding.UTF8.GetBytes(text);
        GCHandle pinField = GCHandle.Alloc(fieldUtf8, GCHandleType.Pinned);
        GCHandle pinText = default;
        bool hasTextPin = false;
        try
        {
            IntPtr textPtr = IntPtr.Zero;
            if (textUtf8.Length > 0)
            {
                pinText = GCHandle.Alloc(textUtf8, GCHandleType.Pinned);
                hasTextPin = true;
                textPtr = pinText.AddrOfPinnedObject();
            }
            return _commitVirtualKeyboard(
                _handle,
                surfaceId,
                pinField.AddrOfPinnedObject(),
                (UIntPtr)fieldUtf8.Length,
                textPtr,
                (UIntPtr)textUtf8.Length);
        }
        finally
        {
            pinField.Free();
            if (hasTextPin) pinText.Free();
        }
    }

    private void RegisterChilloutVrFonts()
    {
        if (_registerFont == null || _setDefaultFontFamily == null)
            return;

        try
        {
            string fontsDir = Path.Combine(UnityEngine.Application.streamingAssetsPath, "Cohtml", "UIResources", "Fonts");
            if (!Directory.Exists(fontsDir))
                return;

            bool registeredAny = false;
            foreach ((string file, string family) in new[]
            {
                ("NotoSans.ttf", "Noto Sans"),
                ("NotoSans-Bold.ttf", "Noto Sans"),
                ("NotoSans-Italic.ttf", "Noto Sans"),
                ("NotoSansSymbols.ttf", string.Empty)
            })
            {
                string path = Path.Combine(fontsDir, file);
                if (!File.Exists(path))
                    continue;
                byte[] bytes = File.ReadAllBytes(path);
                if (bytes.Length == 0)
                    continue;
                if (RegisterFontBytes(family, bytes))
                    registeredAny = true;
            }

            if (registeredAny)
            {
                SetDefaultFontFamily("Noto Sans");
                MelonLogger.Msg("[LumenicMenu] Registered ChilloutVR Noto Sans fonts as default UI font.");
            }
        }
        catch (Exception ex)
        {
            MelonLogger.Warning($"[LumenicMenu] Failed to register CVR Noto Sans fonts: {ex.Message}");
        }
    }

    public bool RegisterFontBytes(string familyName, byte[] fontBytes)
    {
        if (_registerFont == null || fontBytes == null || fontBytes.Length == 0)
            return false;

        byte[] nameUtf8 = string.IsNullOrEmpty(familyName) ? null : Encoding.UTF8.GetBytes(familyName);
        GCHandle pinName = default;
        bool hasName = nameUtf8 != null && nameUtf8.Length > 0;
        if (hasName)
            pinName = GCHandle.Alloc(nameUtf8, GCHandleType.Pinned);
        GCHandle pinBytes = GCHandle.Alloc(fontBytes, GCHandleType.Pinned);
        try
        {
            return _registerFont(
                hasName ? pinName.AddrOfPinnedObject() : IntPtr.Zero,
                hasName ? (UIntPtr)nameUtf8.Length : UIntPtr.Zero,
                pinBytes.AddrOfPinnedObject(),
                (UIntPtr)fontBytes.Length);
        }
        finally
        {
            if (hasName) pinName.Free();
            pinBytes.Free();
        }
    }

    public bool SetDefaultFontFamily(string familyName)
    {
        if (_setDefaultFontFamily == null || string.IsNullOrEmpty(familyName))
            return false;

        byte[] nameUtf8 = Encoding.UTF8.GetBytes(familyName);
        GCHandle pinName = GCHandle.Alloc(nameUtf8, GCHandleType.Pinned);
        try
        {
            return _setDefaultFontFamily(pinName.AddrOfPinnedObject(), (UIntPtr)nameUtf8.Length);
        }
        finally
        {
            pinName.Free();
        }
    }

    public bool UploadImageBytes(uint surfaceId, string key, byte[] imageBytes)
    {
        if (!IsHealthy || string.IsNullOrEmpty(key) || imageBytes == null || imageBytes.Length == 0)
            return false;

        byte[] keyUtf8 = Encoding.UTF8.GetBytes(key);
        GCHandle pinKey = GCHandle.Alloc(keyUtf8, GCHandleType.Pinned);
        GCHandle pinBytes = GCHandle.Alloc(imageBytes, GCHandleType.Pinned);
        try
        {
            return _uploadImageBytes(
                _handle,
                surfaceId,
                pinKey.AddrOfPinnedObject(),
                (UIntPtr)keyUtf8.Length,
                pinBytes.AddrOfPinnedObject(),
                (UIntPtr)imageBytes.Length);
        }
        finally
        {
            pinKey.Free();
            pinBytes.Free();
        }
    }

    private byte[] EnsureScratchCapacity(int minBytes)
    {
        if (_utf8Scratch == null || _utf8Scratch.Length < minBytes)
        {
            int nextSize = Math.Max(minBytes, _utf8Scratch != null ? _utf8Scratch.Length * 2 : 4096);
            _utf8Scratch = new byte[nextSize];
        }
        return _utf8Scratch;
    }

    public bool UploadRgbaTexture(uint surfaceId, string key, uint width, uint height, byte[] rgbaBytes)
    {
        if (!IsHealthy || _uploadRgbaTexture == null || string.IsNullOrEmpty(key) || rgbaBytes == null || width == 0 || height == 0)
            return false;

        int expected = checked((int)(width * height * 4));
        if (rgbaBytes.Length < expected)
            return false;

        byte[] keyUtf8 = Encoding.UTF8.GetBytes(key);
        GCHandle pinKey = GCHandle.Alloc(keyUtf8, GCHandleType.Pinned);
        GCHandle pinRgba = GCHandle.Alloc(rgbaBytes, GCHandleType.Pinned);
        try
        {
            return _uploadRgbaTexture(
                _handle,
                surfaceId,
                pinKey.AddrOfPinnedObject(),
                (UIntPtr)keyUtf8.Length,
                width,
                height,
                pinRgba.AddrOfPinnedObject(),
                (UIntPtr)expected);
        }
        finally
        {
            pinKey.Free();
            pinRgba.Free();
        }
    }

    public bool PushExtensionsJson(string json)
    {
        if (!IsHealthy || _pushExtensionsJson == null || string.IsNullOrEmpty(json)) return false;
        int maxBytes = Encoding.UTF8.GetMaxByteCount(json.Length);
        byte[] buf = EnsureScratchCapacity(maxBytes);
        int byteCount = Encoding.UTF8.GetBytes(json, 0, json.Length, buf, 0);
        GCHandle pin = GCHandle.Alloc(buf, GCHandleType.Pinned);
        try
        {
            return _pushExtensionsJson(_handle, pin.AddrOfPinnedObject(), (UIntPtr)byteCount);
        }
        finally
        {
            pin.Free();
        }
    }

    public bool PushStateJson(string json)
    {
        if (!IsHealthy || string.IsNullOrEmpty(json)) return false;
        int maxBytes = Encoding.UTF8.GetMaxByteCount(json.Length);
        byte[] buf = EnsureScratchCapacity(maxBytes);
        int byteCount = Encoding.UTF8.GetBytes(json, 0, json.Length, buf, 0);
        GCHandle pin = GCHandle.Alloc(buf, GCHandleType.Pinned);
        try
        {
            return _pushStateJson(_handle, pin.AddrOfPinnedObject(), (UIntPtr)byteCount);
        }
        finally
        {
            pin.Free();
        }
    }

    public bool PushTelemetry(int fps, int gamePing, int voicePing, string clockText, string dateText)
    {
        if (!IsHealthy) return false;
        string c = clockText ?? string.Empty;
        string d = dateText ?? string.Empty;
        int maxC = Encoding.UTF8.GetMaxByteCount(c.Length);
        int maxD = Encoding.UTF8.GetMaxByteCount(d.Length);
        byte[] buf = EnsureScratchCapacity(maxC + maxD + 1);
        int cLen = c.Length > 0 ? Encoding.UTF8.GetBytes(c, 0, c.Length, buf, 0) : 0;
        int dLen = d.Length > 0 ? Encoding.UTF8.GetBytes(d, 0, d.Length, buf, cLen) : 0;
        GCHandle pin = GCHandle.Alloc(buf, GCHandleType.Pinned);
        try
        {
            IntPtr basePtr = pin.AddrOfPinnedObject();
            IntPtr cPtr = cLen > 0 ? basePtr : IntPtr.Zero;
            IntPtr dPtr = dLen > 0 ? IntPtr.Add(basePtr, cLen) : IntPtr.Zero;
            return _pushTelemetry(_handle, fps, gamePing, voicePing, cPtr, (UIntPtr)cLen, dPtr, (UIntPtr)dLen);
        }
        finally
        {
            pin.Free();
        }
    }

    public uint Step(uint surfaceId, float dtSecs, bool forceRedraw)
    {
        if (!IsHealthy) return StepFlagErrorOrPanic;
        return _step(_handle, surfaceId, dtSecs, forceRedraw);
    }

    public bool ReadPixels(uint surfaceId, IntPtr dstPtr, int dstLen)
    {
        if (!IsHealthy || dstPtr == IntPtr.Zero || dstLen <= 0) return false;
        return _readPixels(_handle, surfaceId, dstPtr, (UIntPtr)dstLen);
    }

    public string PollActionsJson()
    {
        if (!IsHealthy) return null;
        if (!_pollActionsJson(_handle, out IntPtr ptr, out UIntPtr len))
            return null;

        ulong byteLen = len.ToUInt64();
        if (ptr == IntPtr.Zero || byteLen == 0 || byteLen > 16 * 1024 * 1024)
            return null;

        int count = (int)byteLen;
        byte[] buf = EnsureScratchCapacity(count);
        Marshal.Copy(ptr, buf, 0, count);
        return Encoding.UTF8.GetString(buf, 0, count);
    }

    private void UnloadInternal()
    {
        if (_handle != IntPtr.Zero && _destroy != null)
        {
            try
            {
                _destroy(_handle);
            }
            catch (Exception ex)
            {
                MelonLogger.Error($"[LumenicMenu] Exception during lumenic_cvr_destroy: {ex.Message}");
            }
        }
        _handle = IntPtr.Zero;

        // Clear all managed delegates referencing the native DLL before calling FreeLibrary.
        _abiVersion = null;
        _lastError = null;
        _create = null;
        _destroy = null;
        _resize = null;
        _pointerMove = null;
        _pointerDown = null;
        _pointerUp = null;
        _pointerScroll = null;
        _pointerLeave = null;
        _pointerCancel = null;
        _keyDown = null;
        _textInput = null;
        _clearFocus = null;
        _commitVirtualKeyboard = null;
        _uploadImageBytes = null;
        _pushStateJson = null;
        _pushTelemetry = null;
        _step = null;
        _readPixels = null;
        _pollActionsJson = null;

        if (_module != IntPtr.Zero)
        {
            FreeLibrary(_module);
            _module = IntPtr.Zero;
        }
    }

    public void Dispose()
    {
        if (_disposed) return;
        _disposed = true;
        UnloadInternal();
    }
}
