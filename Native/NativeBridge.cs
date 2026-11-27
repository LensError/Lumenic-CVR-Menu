using System;
using System.IO;
using System.Reflection;
using System.Runtime.InteropServices;
using System.Security.Cryptography;
using System.Text;
using System.Threading;
using MelonLoader;
using MelonLoader.Utils;

namespace LumenicMenu.Native;

/// <summary>
/// Dynamic Win32 loader and FFI wrapper for lumenic_cvr_ui.dll.
/// Must be called strictly on Unity's main thread to match WGL context thread affinity,
/// except <see cref="TryDecodeImage"/> which is stateless and thread-safe.
/// </summary>
internal sealed class NativeBridge : IDisposable
{
    public const uint ExpectedAbiVersion = 8;
    public const uint SurfaceMain = 0;
    public const uint SurfaceQuick = 1;

    public const uint DomainCore = 0;
    public const uint DomainStaticCatalog = 1;
    public const uint DomainUiLib = 2;
    public const uint DomainContent = 3;
    public const uint DomainSocial = 4;
    public const uint DomainSettings = 5;
    public const uint DomainAvatarSettings = 6;
    public const uint DomainUix = 7;

    public const uint StepFlagRendered = 1u << 0;
    public const uint StepFlagTextFocused = 1u << 1;
    public const uint StepFlagPointerOverUi = 1u << 2;
    public const uint StepFlagHasActions = 1u << 3;
    public const uint StepFlagPixelsRead = 1u << 4;
    public const uint StepFlagErrorOrPanic = 1u << 31;

    [StructLayout(LayoutKind.Sequential)]
    private struct NativeRgbaUploadItem
    {
        public uint SurfaceId;
        public uint Width;
        public uint Height;
        public IntPtr KeyPtr;
        public UIntPtr KeyLen;
        public IntPtr RgbaPtr;
        public UIntPtr RgbaLen;
    }

    public readonly struct RgbaBatchItem
    {
        public readonly uint SurfaceId;
        public readonly string Key;
        public readonly uint Width;
        public readonly uint Height;
        public readonly byte[] RgbaBytes;

        public RgbaBatchItem(uint surfaceId, string key, uint width, uint height, byte[] rgbaBytes)
        {
            SurfaceId = surfaceId;
            Key = key;
            Width = width;
            Height = height;
            RgbaBytes = rgbaBytes;
        }
    }

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
    private delegate UIntPtr UploadRgbaBatchFn(
        IntPtr handle,
        IntPtr itemsPtr,
        UIntPtr count,
        IntPtr outResultsPtr);

    [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
    [return: MarshalAs(UnmanagedType.I1)]
    private delegate bool DecodeImageFn(
        IntPtr bytesPtr,
        UIntPtr bytesLen,
        out uint outWidth,
        out uint outHeight,
        out IntPtr outRgbaPtr,
        out UIntPtr outRgbaLen);

    [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
    private delegate void FreeDecodedFn(IntPtr ptr, UIntPtr len);

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
    private delegate bool PushDomainJsonFn(IntPtr handle, uint domainId, IntPtr utf8Ptr, UIntPtr len);

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
    private delegate uint StepAndReadFn(
        IntPtr handle,
        uint surfaceId,
        float dtSecs,
        [MarshalAs(UnmanagedType.I1)] bool forceRedraw,
        IntPtr dstPtr,
        UIntPtr dstLen);

    [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
    [return: MarshalAs(UnmanagedType.I1)]
    private delegate bool ReadPixelsFn(IntPtr handle, uint surfaceId, IntPtr dstPtr, UIntPtr dstLen);

    [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
    [return: MarshalAs(UnmanagedType.I1)]
    private delegate bool PollActionsJsonFn(IntPtr handle, out IntPtr outPtr, out UIntPtr outLen);

    private readonly ReaderWriterLockSlim _nativeLock = new ReaderWriterLockSlim();
    private IntPtr _module = IntPtr.Zero;
    private IntPtr _handle = IntPtr.Zero;
    private volatile bool _disposed;

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
    private UploadRgbaBatchFn _uploadRgbaBatch;
    private DecodeImageFn _decodeImage;
    private FreeDecodedFn _freeDecoded;
    private RegisterFontFn _registerFont;
    private SetDefaultFontFamilyFn _setDefaultFontFamily;
    private PushStateJsonFn _pushStateJson;
    private PushDomainJsonFn _pushDomainJson;
    private PushStateJsonFn _pushExtensionsJson;
    private PushTelemetryFn _pushTelemetry;
    private StepFn _step;
    private StepAndReadFn _stepAndRead;
    private ReadPixelsFn _readPixels;
    private PollActionsJsonFn _pollActionsJson;

    private byte[] _utf8Scratch = new byte[4096];

    public bool IsHealthy => !_disposed && _module != IntPtr.Zero && _handle != IntPtr.Zero;
    public string LastFailureReason { get; private set; } = string.Empty;

    public static string UserDataDir => Path.Combine(MelonEnvironment.UserDataDirectory, "LumenicMenu");
    public static string DefaultDllPath => Path.Combine(UserDataDir, "lumenic_cvr_ui.dll");

    public bool Load(uint mmW, uint mmH, uint qmW, uint qmH, float scale)
    {
        if (!EnsureEmbeddedDllExtracted(out string extractError))
        {
            LastFailureReason = extractError;
            MelonLogger.Error($"[LumenicMenu] {LastFailureReason}");
            return false;
        }
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

    private static bool EnsureEmbeddedDllExtracted(out string error)
    {
        const string resourceName = "lumenic_cvr_ui.dll";
        error = string.Empty;
        try
        {
            Assembly asm = typeof(NativeBridge).Assembly;
            using Stream resStream = asm.GetManifestResourceStream(resourceName);
            if (resStream == null)
            {
                error = $"Embedded resource '{resourceName}' not found in assembly.";
                return false;
            }

            byte[] embeddedHash;
            using (var sha = SHA256.Create())
            {
                embeddedHash = sha.ComputeHash(resStream);
            }

            if (VerifyFileHash(DefaultDllPath, embeddedHash))
                return true;

            resStream.Position = 0;
            Directory.CreateDirectory(UserDataDir);
            string tmpPath = DefaultDllPath + "." + Guid.NewGuid().ToString("N") + ".tmp";
            try
            {
                using (var fs = new FileStream(tmpPath, FileMode.CreateNew, FileAccess.Write, FileShare.None))
                {
                    resStream.CopyTo(fs);
                }
                if (File.Exists(DefaultDllPath))
                    File.Delete(DefaultDllPath);
                File.Move(tmpPath, DefaultDllPath);
            }
            finally
            {
                if (File.Exists(tmpPath))
                {
                    try { File.Delete(tmpPath); } catch { }
                }
            }

            if (!VerifyFileHash(DefaultDllPath, embeddedHash))
            {
                error = $"SHA-256 verification failed after extracting '{DefaultDllPath}'.";
                return false;
            }

            MelonLogger.Msg("[LumenicMenu] Extracted embedded lumenic_cvr_ui.dll.");
            return true;
        }
        catch (Exception ex)
        {
            error = $"Failed to extract/verify embedded native DLL: {ex.Message}";
            return false;
        }
    }

    private static bool VerifyFileHash(string path, byte[] expectedHash)
    {
        if (!File.Exists(path))
            return false;
        try
        {
            using var existingStream = File.OpenRead(path);
            using var sha = SHA256.Create();
            byte[] existingHash = sha.ComputeHash(existingStream);
            return HashesEqual(expectedHash, existingHash);
        }
        catch
        {
            return false;
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
        _uploadRgbaBatch = Bind<UploadRgbaBatchFn>("lumenic_cvr_upload_rgba_batch");
        _decodeImage = Bind<DecodeImageFn>("lumenic_cvr_decode_image");
        _freeDecoded = Bind<FreeDecodedFn>("lumenic_cvr_free_decoded");
        _registerFont = Bind<RegisterFontFn>("lumenic_cvr_register_font");
        _setDefaultFontFamily = Bind<SetDefaultFontFamilyFn>("lumenic_cvr_set_default_font_family");
        _pushStateJson = Bind<PushStateJsonFn>("lumenic_cvr_push_state_json");
        _pushDomainJson = Bind<PushDomainJsonFn>("lumenic_cvr_push_domain_json");
        _pushExtensionsJson = Bind<PushStateJsonFn>("lumenic_cvr_push_extensions_json");
        _pushTelemetry = Bind<PushTelemetryFn>("lumenic_cvr_push_telemetry");
        _step = Bind<StepFn>("lumenic_cvr_step");
        _stepAndRead = Bind<StepAndReadFn>("lumenic_cvr_step_and_read");
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
            && _uploadRgbaBatch != null
            && _decodeImage != null
            && _freeDecoded != null
            && _registerFont != null
            && _setDefaultFontFamily != null
            && _pushStateJson != null
            && _pushDomainJson != null
            && _pushTelemetry != null
            && _step != null
            && _stepAndRead != null
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

        byte[] buf = EnsureScratchCapacity((int)needed);
        unsafe
        {
            fixed (byte* ptr = buf)
            {
                _lastError((IntPtr)ptr, (UIntPtr)needed);
            }
        }
        return Encoding.UTF8.GetString(buf, 0, (int)needed);
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
        int maxBytes = Encoding.UTF8.GetMaxByteCount(text.Length);
        byte[] buf = EnsureScratchCapacity(maxBytes);
        int byteCount = Encoding.UTF8.GetBytes(text, 0, text.Length, buf, 0);
        unsafe
        {
            fixed (byte* ptr = buf)
            {
                _textInput(_handle, surfaceId, (IntPtr)ptr, (UIntPtr)byteCount);
            }
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
        string t = text ?? string.Empty;
        int maxField = Encoding.UTF8.GetMaxByteCount(fieldId.Length);
        int maxText = Encoding.UTF8.GetMaxByteCount(t.Length);
        byte[] buf = EnsureScratchCapacity(maxField + maxText + 1);
        int fieldLen = Encoding.UTF8.GetBytes(fieldId, 0, fieldId.Length, buf, 0);
        int textLen = t.Length > 0 ? Encoding.UTF8.GetBytes(t, 0, t.Length, buf, fieldLen) : 0;
        unsafe
        {
            fixed (byte* basePtr = buf)
            {
                IntPtr textPtr = textLen > 0 ? (IntPtr)(basePtr + fieldLen) : IntPtr.Zero;
                return _commitVirtualKeyboard(
                    _handle,
                    surfaceId,
                    (IntPtr)basePtr,
                    (UIntPtr)fieldLen,
                    textPtr,
                    (UIntPtr)textLen);
            }
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
        bool hasName = nameUtf8 != null && nameUtf8.Length > 0;
        unsafe
        {
            fixed (byte* namePtr = nameUtf8)
            fixed (byte* bytesPtr = fontBytes)
            {
                return _registerFont(
                    hasName ? (IntPtr)namePtr : IntPtr.Zero,
                    hasName ? (UIntPtr)nameUtf8.Length : UIntPtr.Zero,
                    (IntPtr)bytesPtr,
                    (UIntPtr)fontBytes.Length);
            }
        }
    }

    public bool SetDefaultFontFamily(string familyName)
    {
        if (_setDefaultFontFamily == null || string.IsNullOrEmpty(familyName))
            return false;

        byte[] nameUtf8 = Encoding.UTF8.GetBytes(familyName);
        unsafe
        {
            fixed (byte* namePtr = nameUtf8)
            {
                return _setDefaultFontFamily((IntPtr)namePtr, (UIntPtr)nameUtf8.Length);
            }
        }
    }

    /// <summary>
    /// Thread-safe, stateless image decode callable from background worker threads.
    /// Synchronized against DLL unload via <see cref="_nativeLock"/>.
    /// </summary>
    public bool TryDecodeImage(byte[] encodedBytes, out uint width, out uint height, out byte[] rgbaBytes)
    {
        width = 0;
        height = 0;
        rgbaBytes = null;
        if (_disposed || encodedBytes == null || encodedBytes.Length == 0)
            return false;

        _nativeLock.EnterReadLock();
        try
        {
            if (_disposed || _module == IntPtr.Zero || _decodeImage == null || _freeDecoded == null)
                return false;

            IntPtr rgbaPtr = IntPtr.Zero;
            UIntPtr rgbaLen = UIntPtr.Zero;
            bool ok;
            unsafe
            {
                fixed (byte* srcPtr = encodedBytes)
                {
                    ok = _decodeImage(
                        (IntPtr)srcPtr,
                        (UIntPtr)encodedBytes.Length,
                        out width,
                        out height,
                        out rgbaPtr,
                        out rgbaLen);
                }
            }

            if (!ok || rgbaPtr == IntPtr.Zero)
                return false;

            try
            {
                ulong byteLen = rgbaLen.ToUInt64();
                if (byteLen == 0 || byteLen > 64 * 1024 * 1024)
                    return false;

                rgbaBytes = new byte[(int)byteLen];
                Marshal.Copy(rgbaPtr, rgbaBytes, 0, (int)byteLen);
                return true;
            }
            finally
            {
                _freeDecoded(rgbaPtr, rgbaLen);
            }
        }
        finally
        {
            _nativeLock.ExitReadLock();
        }
    }

    public bool UploadImageBytes(uint surfaceId, string key, byte[] imageBytes)
    {
        if (!IsHealthy || string.IsNullOrEmpty(key) || imageBytes == null || imageBytes.Length == 0)
            return false;

        int maxKey = Encoding.UTF8.GetMaxByteCount(key.Length);
        byte[] keyBuf = EnsureScratchCapacity(maxKey);
        int keyLen = Encoding.UTF8.GetBytes(key, 0, key.Length, keyBuf, 0);
        unsafe
        {
            fixed (byte* keyPtr = keyBuf)
            fixed (byte* bytesPtr = imageBytes)
            {
                return _uploadImageBytes(
                    _handle,
                    surfaceId,
                    (IntPtr)keyPtr,
                    (UIntPtr)keyLen,
                    (IntPtr)bytesPtr,
                    (UIntPtr)imageBytes.Length);
            }
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

        int maxKey = Encoding.UTF8.GetMaxByteCount(key.Length);
        byte[] keyBuf = EnsureScratchCapacity(maxKey);
        int keyLen = Encoding.UTF8.GetBytes(key, 0, key.Length, keyBuf, 0);
        unsafe
        {
            fixed (byte* keyPtr = keyBuf)
            fixed (byte* rgbaPtr = rgbaBytes)
            {
                return _uploadRgbaTexture(
                    _handle,
                    surfaceId,
                    (IntPtr)keyPtr,
                    (UIntPtr)keyLen,
                    width,
                    height,
                    (IntPtr)rgbaPtr,
                    (UIntPtr)expected);
            }
        }
    }

    public int UploadRgbaBatch(RgbaBatchItem[] items, int count, byte[] outResults)
    {
        if (!IsHealthy || _uploadRgbaBatch == null || items == null || count <= 0)
            return 0;

        const int MaxStackBatch = 64;
        int totalCount = Math.Min(count, items.Length);
        int totalUploaded = 0;
        for (int offset = 0; offset < totalCount; offset += MaxStackBatch)
        {
            int chunkCount = Math.Min(MaxStackBatch, totalCount - offset);
            totalUploaded += UploadRgbaBatchChunk(items, offset, chunkCount, outResults);
        }
        return totalUploaded;
    }

    private int UploadRgbaBatchChunk(RgbaBatchItem[] items, int offset, int count, byte[] outResults)
    {
        int totalKeyBytes = 0;
        for (int i = 0; i < count; i++)
        {
            totalKeyBytes += Encoding.UTF8.GetMaxByteCount(items[offset + i].Key?.Length ?? 0);
        }
        byte[] keyBuf = EnsureScratchCapacity(Math.Max(1, totalKeyBytes));

        GCHandle[] rgbaPins = new GCHandle[count];
        int pinnedCount = 0;
        try
        {
            unsafe
            {
                NativeRgbaUploadItem* nativeItems = stackalloc NativeRgbaUploadItem[count];
                fixed (byte* keyBase = keyBuf)
                fixed (byte* resultsPtr = outResults)
                {
                    int keyOffset = 0;
                    for (int i = 0; i < count; i++)
                    {
                        ref readonly RgbaBatchItem item = ref items[offset + i];
                        string k = item.Key ?? string.Empty;
                        int kLen = k.Length > 0 ? Encoding.UTF8.GetBytes(k, 0, k.Length, keyBuf, keyOffset) : 0;

                        IntPtr rgbaPtr = IntPtr.Zero;
                        int rgbaLen = 0;
                        if (item.RgbaBytes != null && item.RgbaBytes.Length > 0)
                        {
                            GCHandle pin = GCHandle.Alloc(item.RgbaBytes, GCHandleType.Pinned);
                            rgbaPins[pinnedCount++] = pin;
                            rgbaPtr = pin.AddrOfPinnedObject();
                            rgbaLen = item.RgbaBytes.Length;
                        }

                        nativeItems[i] = new NativeRgbaUploadItem
                        {
                            SurfaceId = item.SurfaceId,
                            Width = item.Width,
                            Height = item.Height,
                            KeyPtr = kLen > 0 ? (IntPtr)(keyBase + keyOffset) : IntPtr.Zero,
                            KeyLen = (UIntPtr)kLen,
                            RgbaPtr = rgbaPtr,
                            RgbaLen = (UIntPtr)rgbaLen
                        };
                        keyOffset += kLen;
                    }

                    IntPtr chunkResultsPtr = outResults != null && outResults.Length >= offset + count
                        ? (IntPtr)(resultsPtr + offset)
                        : IntPtr.Zero;
                    return (int)_uploadRgbaBatch(
                        _handle,
                        (IntPtr)nativeItems,
                        (UIntPtr)count,
                        chunkResultsPtr).ToUInt64();
                }
            }
        }
        finally
        {
            for (int i = 0; i < pinnedCount; i++)
            {
                rgbaPins[i].Free();
            }
        }
    }

    public bool PushExtensionsJson(string json)
    {
        if (!IsHealthy || _pushExtensionsJson == null || string.IsNullOrEmpty(json)) return false;
        int maxBytes = Encoding.UTF8.GetMaxByteCount(json.Length);
        byte[] buf = EnsureScratchCapacity(maxBytes);
        int byteCount = Encoding.UTF8.GetBytes(json, 0, json.Length, buf, 0);
        unsafe
        {
            fixed (byte* ptr = buf)
            {
                return _pushExtensionsJson(_handle, (IntPtr)ptr, (UIntPtr)byteCount);
            }
        }
    }

    public bool PushStateJson(string json)
    {
        if (!IsHealthy || string.IsNullOrEmpty(json)) return false;
        int maxBytes = Encoding.UTF8.GetMaxByteCount(json.Length);
        byte[] buf = EnsureScratchCapacity(maxBytes);
        int byteCount = Encoding.UTF8.GetBytes(json, 0, json.Length, buf, 0);
        unsafe
        {
            fixed (byte* ptr = buf)
            {
                return _pushStateJson(_handle, (IntPtr)ptr, (UIntPtr)byteCount);
            }
        }
    }

    public bool PushDomainJson(uint domainId, string json)
    {
        if (!IsHealthy || _pushDomainJson == null || string.IsNullOrEmpty(json)) return false;
        int maxBytes = Encoding.UTF8.GetMaxByteCount(json.Length);
        byte[] buf = EnsureScratchCapacity(maxBytes);
        int byteCount = Encoding.UTF8.GetBytes(json, 0, json.Length, buf, 0);
        unsafe
        {
            fixed (byte* ptr = buf)
            {
                return _pushDomainJson(_handle, domainId, (IntPtr)ptr, (UIntPtr)byteCount);
            }
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
        unsafe
        {
            fixed (byte* basePtr = buf)
            {
                IntPtr cPtr = cLen > 0 ? (IntPtr)basePtr : IntPtr.Zero;
                IntPtr dPtr = dLen > 0 ? (IntPtr)(basePtr + cLen) : IntPtr.Zero;
                return _pushTelemetry(_handle, fps, gamePing, voicePing, cPtr, (UIntPtr)cLen, dPtr, (UIntPtr)dLen);
            }
        }
    }

    public uint Step(uint surfaceId, float dtSecs, bool forceRedraw)
    {
        if (!IsHealthy) return StepFlagErrorOrPanic;
        return _step(_handle, surfaceId, dtSecs, forceRedraw);
    }

    public uint StepAndRead(uint surfaceId, float dtSecs, bool forceRedraw, IntPtr dstPtr, int dstLen)
    {
        if (!IsHealthy) return StepFlagErrorOrPanic;
        return _stepAndRead(_handle, surfaceId, dtSecs, forceRedraw, dstPtr, dstLen > 0 ? (UIntPtr)dstLen : UIntPtr.Zero);
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
        _nativeLock.EnterWriteLock();
        try
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
            _uploadRgbaTexture = null;
            _uploadRgbaBatch = null;
            _decodeImage = null;
            _freeDecoded = null;
            _registerFont = null;
            _setDefaultFontFamily = null;
            _pushStateJson = null;
            _pushDomainJson = null;
            _pushExtensionsJson = null;
            _pushTelemetry = null;
            _step = null;
            _stepAndRead = null;
            _readPixels = null;
            _pollActionsJson = null;

            if (_module != IntPtr.Zero)
            {
                FreeLibrary(_module);
                _module = IntPtr.Zero;
            }
        }
        finally
        {
            _nativeLock.ExitWriteLock();
        }
    }

    public void Dispose()
    {
        if (_disposed) return;
        _disposed = true;
        UnloadInternal();
        _nativeLock.Dispose();
    }
}
