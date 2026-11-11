using System.Text.Json.Serialization;

namespace Tether.WinUI;

internal sealed class FlowSnapshot
{
    [JsonPropertyName("state")]
    public string State { get; set; } = "idle";

    [JsonPropertyName("time")]
    public string Time { get; set; } = "00:00";

    [JsonPropertyName("elapsed_focus_secs")]
    public ulong ElapsedFocusSecs { get; set; }

    [JsonPropertyName("remaining_break_secs")]
    public ulong RemainingBreakSecs { get; set; }

    [JsonPropertyName("micro_remaining_secs")]
    public ulong MicroRemainingSecs { get; set; }

    [JsonPropertyName("break_secs_available")]
    public ulong BreakSecsAvailable { get; set; }

    [JsonPropertyName("focus_ratio")]
    public uint FocusRatio { get; set; }

    [JsonPropertyName("break_ratio")]
    public uint BreakRatio { get; set; }

    [JsonPropertyName("progress")]
    public float Progress { get; set; }
}
