using Microsoft.UI.Dispatching;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Xaml.Navigation;

namespace Tether.WinUI;

public sealed partial class MainWindow : Window
{
    private readonly IntPtr _flow;
    private readonly DispatcherQueueTimer _timer;
    private DateTimeOffset _lastTick;
    private FlowSnapshot _snapshot = new();
    private FlowPage? _flowPage;

    public MainWindow()
    {
        InitializeComponent();
        Title = "Tether";

        var settings = CoreServices.Call("settings.load");
        var focusRatio = settings.GetProperty("ratio_focus").GetUInt32();
        var breakRatio = settings.GetProperty("ratio_break").GetUInt32();
        _flow = FlowNative.tether_flow_new();
        if (_flow == IntPtr.Zero)
        {
            throw new InvalidOperationException("The Tether core could not be initialized.");
        }

        if (!FlowNative.tether_flow_set_ratios(
                _flow,
                focusRatio,
                breakRatio))
        {
            FlowNative.tether_flow_free(_flow);
            throw new InvalidOperationException("The Tether core rejected the saved focus/break ratios.");
        }

        Closed += OnClosed;
        _timer = DispatcherQueue.GetForCurrentThread().CreateTimer();
        _timer.Interval = TimeSpan.FromMilliseconds(500);
        _timer.Tick += OnTimerTick;
        _lastTick = DateTimeOffset.UtcNow;
        _timer.Start();

        RootNavigation.SelectedItem = FlowItem;
        NavigateTo("flow");
        RefreshFlow();
    }

    private void RootNavigation_ItemInvoked(
        NavigationView sender,
        NavigationViewItemInvokedEventArgs args)
    {
        if (args.InvokedItemContainer is NavigationViewItem item && item.Tag is string route)
        {
            RootNavigation.SelectedItem = item;
            NavigateTo(route);
        }
    }

    private void NavigateTo(string route)
    {
        switch (route)
        {
            case "flow":
                ContentFrame.Navigate(typeof(FlowPage));
                break;
            case "ramdump":
                ShowPreview(
                    "RAM-dump",
                    "Get the open loops out of your head.",
                    "Capture what you’re doing, what’s on your mind, and the next step so you can return without rebuilding context.");
                break;
            case "impulses":
                ShowPreview(
                    "Impulses",
                    "Park a thought. Return to your task.",
                    "Capture small distractions, review them later, and keep your focus session intact.");
                break;
            case "momentum":
                ShowPreview(
                    "Momentum",
                    "Notice progress without streak pressure.",
                    "Your weekly focus, completed sessions, and parked impulses will live here.");
                break;
            case "rewards":
                ShowPreview(
                    "Rewards",
                    "Make breaks something to look forward to.",
                    "Use focus points to add, choose, and redeem rewards that work for you.");
                break;
            case "settings":
                ShowPreview(
                    "Settings",
                    "Make Tether fit the way you work.",
                    "Configure focus ratios, language, impulse files, soundscapes, and editor integrations.");
                break;
        }
    }

    private void ShowPreview(string title, string subtitle, string description)
    {
        ContentFrame.Navigate(
            typeof(FeaturePreviewPage),
            new FeaturePreviewInfo(title, subtitle, description));
    }

    private void ContentFrame_Navigated(object sender, NavigationEventArgs args)
    {
        if (_flowPage is not null)
        {
            _flowPage.ActionRequested -= OnFlowActionRequested;
        }

        _flowPage = args.Content as FlowPage;
        if (_flowPage is not null)
        {
            _flowPage.ActionRequested += OnFlowActionRequested;
            _flowPage.Present(_snapshot);
        }
    }

    private void OnFlowActionRequested(uint action)
    {
        if (!FlowNative.tether_flow_action(_flow, action))
        {
            throw new InvalidOperationException($"The Tether core rejected timer action {action}.");
        }

        RefreshFlow();
    }

    private void OnTimerTick(DispatcherQueueTimer sender, object args)
    {
        var now = DateTimeOffset.UtcNow;
        var elapsed = (ulong)Math.Max(0, (now - _lastTick).TotalMilliseconds);
        _lastTick = now;
        if (!FlowNative.tether_flow_tick(_flow, elapsed))
        {
            _timer.Stop();
            throw new InvalidOperationException("The Tether core stopped responding to timer updates.");
        }

        RefreshFlow();
    }

    private void RefreshFlow()
    {
        _snapshot = FlowNative.ReadSnapshot(_flow);
        _flowPage?.Present(_snapshot);
    }

    private void OnClosed(object sender, WindowEventArgs args)
    {
        _timer.Stop();
        FlowNative.tether_flow_free(_flow);
    }
}
