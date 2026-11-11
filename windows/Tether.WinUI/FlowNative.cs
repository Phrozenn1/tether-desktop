using System.Runtime.InteropServices;
using System.Text.Json;

namespace Tether.WinUI;

internal static class FlowNative
{
    private const string Library = "tether_core";

    [DllImport(Library, CallingConvention = CallingConvention.Cdecl)]
    internal static extern IntPtr tether_flow_new();

    [DllImport(Library, CallingConvention = CallingConvention.Cdecl)]
    [return: MarshalAs(UnmanagedType.I1)]
    internal static extern bool tether_flow_tick(IntPtr flow, ulong elapsedMillis);

    [DllImport(Library, CallingConvention = CallingConvention.Cdecl)]
    [return: MarshalAs(UnmanagedType.I1)]
    internal static extern bool tether_flow_action(IntPtr flow, uint action);

    [DllImport(Library, CallingConvention = CallingConvention.Cdecl)]
    [return: MarshalAs(UnmanagedType.I1)]
    internal static extern bool tether_flow_set_ratios(IntPtr flow, uint focusRatio, uint breakRatio);

    [DllImport(Library, CallingConvention = CallingConvention.Cdecl)]
    internal static extern IntPtr tether_flow_snapshot(IntPtr flow);

    [DllImport(Library, CallingConvention = CallingConvention.Cdecl)]
    internal static extern void tether_flow_free(IntPtr flow);

    internal static FlowSnapshot ReadSnapshot(IntPtr flow)
    {
        var jsonPointer = tether_flow_snapshot(flow);
        if (jsonPointer == IntPtr.Zero)
        {
            throw new InvalidOperationException("The Tether core could not provide timer state.");
        }

        try
        {
            var json = Marshal.PtrToStringUTF8(jsonPointer)
                ?? throw new InvalidOperationException("The Tether core returned invalid UTF-8 data.");
            return JsonSerializer.Deserialize<FlowSnapshot>(json)
                ?? throw new InvalidOperationException("The Tether core returned an empty timer state.");
        }
        finally
        {
            CoreServices.FreeString(jsonPointer);
        }
    }
}
