using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;

namespace Tether.WinUI;

public sealed partial class FlowPage : Page
{
    internal event Action<uint>? ActionRequested;

    public FlowPage()
    {
        InitializeComponent();
    }

    internal void Present(FlowSnapshot snapshot)
    {
        TimerText.Text = snapshot.Time;
        FlowProgress.Value = Math.Clamp(snapshot.Progress, 0, 1);
        StatusText.Text = snapshot.State switch
        {
            "micro_start" => "MICRO-START",
            "focus" => "FOCUS",
            "break" => "BREAK",
            _ => "READY"
        };
        ModeText.Text = snapshot.State switch
        {
            "micro_start" => "A small start is still a start.",
            "focus" => "You’re in focus.",
            "break" => "Take the time you’ve earned.",
            _ => "Ready when you are"
        };
        HelperText.Text = snapshot.State switch
        {
            "focus" => $"Earned break: {FormatTime(snapshot.BreakSecsAvailable)} · Ratio {snapshot.FocusRatio}:{snapshot.BreakRatio}",
            "break" => "Resume when you feel ready; unused break time is banked.",
            _ => "Work at your own pace. Breaks are earned as you go."
        };
        ToggleButton.Content = snapshot.State switch
        {
            "idle" => "Start focus",
            "micro_start" => "Cancel micro-start",
            "focus" => "Take a break",
            "break" => "Resume focus",
            _ => "Start focus"
        };
        BreakButton.Content = snapshot.State == "break" ? "Resume focus" : "Take a break";
        BreakButton.IsEnabled = snapshot.State is "focus" or "break";
        FlowProgress.Visibility = snapshot.State == "focus" ? Visibility.Collapsed : Visibility.Visible;
    }

    private void Toggle_Click(object sender, RoutedEventArgs e)
    {
        ActionRequested?.Invoke(2);
    }

    private void MicroStart_Click(object sender, RoutedEventArgs e)
    {
        ActionRequested?.Invoke(1);
    }

    private void Break_Click(object sender, RoutedEventArgs e)
    {
        ActionRequested?.Invoke(BreakButton.Content?.ToString() == "Resume focus" ? 4u : 3u);
    }

    private void Reset_Click(object sender, RoutedEventArgs e)
    {
        ActionRequested?.Invoke(5);
    }

    private static string FormatTime(ulong seconds)
    {
        var hours = seconds / 3600;
        var minutes = (seconds % 3600) / 60;
        var remaining = seconds % 60;
        return hours > 0
            ? $"{hours:00}:{minutes:00}:{remaining:00}"
            : $"{minutes:00}:{remaining:00}";
    }
}
