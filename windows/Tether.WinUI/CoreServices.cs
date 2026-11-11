using System.Runtime.InteropServices;
using System.Text.Json;

namespace Tether.WinUI;

internal static class CoreServices
{
    private const string Library = "tether_core";

    [DllImport(Library, EntryPoint = "tether_core_request", CallingConvention = CallingConvention.Cdecl)]
    private static extern IntPtr Request(IntPtr requestJson);

    [DllImport(Library, EntryPoint = "tether_string_free", CallingConvention = CallingConvention.Cdecl)]
    private static extern void FreeResponse(IntPtr response);

    internal static JsonElement Call(string operation, object? payload = null)
    {
        var requestJson = Marshal.StringToCoTaskMemUTF8(JsonSerializer.Serialize(new
        {
            operation,
            payload
        }));
        try
        {
            var responsePointer = Request(requestJson);
            if (responsePointer == IntPtr.Zero)
            {
                throw new InvalidOperationException("The Tether core returned no service response.");
            }

            try
            {
                var responseJson = Marshal.PtrToStringUTF8(responsePointer)
                    ?? throw new InvalidOperationException("The Tether core returned invalid UTF-8 data.");
                using var response = JsonDocument.Parse(responseJson);
                var root = response.RootElement;
                if (!root.GetProperty("ok").GetBoolean())
                {
                    throw new InvalidOperationException(root.GetProperty("error").GetString()
                        ?? "The Tether core service failed.");
                }

                return root.GetProperty("data").Clone();
            }
            finally
            {
                FreeResponse(responsePointer);
            }
        }
        finally
        {
            Marshal.FreeCoTaskMem(requestJson);
        }
    }

    internal static void FreeString(IntPtr value)
    {
        if (value != IntPtr.Zero)
        {
            FreeResponse(value);
        }
    }
}
