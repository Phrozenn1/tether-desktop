using System.Text.Json;
using System.Text.Json.Nodes;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Xaml.Media;
using Microsoft.UI.Xaml.Navigation;

namespace Tether.WinUI;

internal sealed record FeaturePreviewInfo(string Route, string Title, string Subtitle, string Description);

public sealed partial class FeaturePreviewPage : Page
{
    private string _currentRoute = "ramdump";
    private int? _editingRamDumpIndex;

    public FeaturePreviewPage()
    {
        InitializeComponent();
    }

    protected override void OnNavigatedTo(NavigationEventArgs e)
    {
        base.OnNavigatedTo(e);
        if (e.Parameter is FeaturePreviewInfo preview)
        {
            _currentRoute = preview.Route;
            _editingRamDumpIndex = null;
            PageTitle.Text = preview.Title;
            PageSubtitle.Text = preview.Subtitle;
            PageDescription.Text = preview.Description;
            StatusBanner.Visibility = Visibility.Collapsed;
            RenderCurrentRoute();
        }
    }

    private void RenderCurrentRoute()
    {
        InteractivePanel.Children.Clear();
        try
        {
            switch (_currentRoute)
            {
                case "ramdump":
                    RenderRamDumpView();
                    break;
                case "impulses":
                    RenderImpulsesView();
                    break;
                case "momentum":
                    RenderMomentumView();
                    break;
                case "rewards":
                    RenderRewardsView();
                    break;
                case "settings":
                    RenderSettingsView();
                    break;
            }
        }
        catch (Exception ex)
        {
            SetStatus($"Core error: {ex.Message}");
        }
    }

    private void SetStatus(string message)
    {
        StatusBanner.Text = message;
        StatusBanner.Visibility = string.IsNullOrWhiteSpace(message)
            ? Visibility.Collapsed
            : Visibility.Visible;
    }

    private static Border CardBorder(UIElement child)
    {
        var brush = Application.Current.Resources["CardBackgroundFillColorDefaultBrush"] as Brush;
        return new Border
        {
            Padding = new Thickness(16),
            CornerRadius = new CornerRadius(12),
            Background = brush,
            Child = child,
        };
    }

    private static TextBox LabeledInput(string header, string placeholder, string value = "")
    {
        return new TextBox
        {
            Header = header,
            PlaceholderText = placeholder,
            Text = value,
            AcceptsReturn = false,
        };
    }

    private void RenderRamDumpView()
    {
        var nowBox = LabeledInput("[NOW] What are you working on?", "Current task or bug...");
        var statusBox = LabeledInput("[STATUS] Working memory / blockers", "What needs remembering?");
        var nextBox = LabeledInput("[NEXT] First concrete micro-step", "Open file X and run Y...");

        var formActions = new StackPanel { Orientation = Orientation.Horizontal, Spacing = 8 };
        var saveBtn = new Button { Content = _editingRamDumpIndex.HasValue ? "Update RAM-dump" : "Save RAM-dump" };
        var suggestBtn = new Button { Content = "Auto-fill from Context / AI" };
        var cancelEditBtn = new Button
        {
            Content = "Cancel Edit",
            Visibility = _editingRamDumpIndex.HasValue ? Visibility.Visible : Visibility.Collapsed,
        };

        var dumps = CoreServices.Call("ramdump.list");
        if (_editingRamDumpIndex is int editIdx && dumps.ValueKind == JsonValueKind.Array && editIdx >= 0 && editIdx < dumps.GetArrayLength())
        {
            var entry = dumps[editIdx];
            nowBox.Text = entry.TryGetProperty("now", out var n) ? n.GetString() ?? "" : "";
            statusBox.Text = entry.TryGetProperty("status", out var s) ? s.GetString() ?? "" : "";
            nextBox.Text = entry.TryGetProperty("next_step", out var nx) ? nx.GetString() ?? "" : "";
        }

        suggestBtn.Click += (_, _) =>
        {
            try
            {
                var suggestion = CoreServices.Call("ramdump.suggest");
                nowBox.Text = suggestion.TryGetProperty("now", out var n) ? n.GetString() ?? "" : "";
                statusBox.Text = suggestion.TryGetProperty("status", out var s) ? s.GetString() ?? "" : "";
                nextBox.Text = suggestion.TryGetProperty("next_step", out var nx) ? nx.GetString() ?? "" : "";
                var src = suggestion.TryGetProperty("source_label", out var sl) ? sl.GetString() ?? "Context" : "Context";
                SetStatus($"Suggested from {src}.");
            }
            catch (Exception ex)
            {
                SetStatus(ex.Message);
            }
        };

        saveBtn.Click += (_, _) =>
        {
            try
            {
                if (_editingRamDumpIndex is int idx)
                {
                    CoreServices.Call("ramdump.update", new
                    {
                        index = idx,
                        now = nowBox.Text.Trim(),
                        status = statusBox.Text.Trim(),
                        next_step = nextBox.Text.Trim(),
                    });
                    _editingRamDumpIndex = null;
                    SetStatus("RAM-dump updated.");
                }
                else
                {
                    CoreServices.Call("ramdump.save", new
                    {
                        now = nowBox.Text.Trim(),
                        status = statusBox.Text.Trim(),
                        next_step = nextBox.Text.Trim(),
                    });
                    SetStatus("RAM-dump saved.");
                }
                RenderCurrentRoute();
            }
            catch (Exception ex)
            {
                SetStatus(ex.Message);
            }
        };

        cancelEditBtn.Click += (_, _) =>
        {
            _editingRamDumpIndex = null;
            RenderCurrentRoute();
        };

        formActions.Children.Add(saveBtn);
        formActions.Children.Add(suggestBtn);
        formActions.Children.Add(cancelEditBtn);

        var formStack = new StackPanel { Spacing = 10 };
        formStack.Children.Add(nowBox);
        formStack.Children.Add(statusBox);
        formStack.Children.Add(nextBox);
        formStack.Children.Add(formActions);
        InteractivePanel.Children.Add(CardBorder(formStack));

        // Evening Shutdown Ritual card
        var shutdownNowBox = LabeledInput("Shutdown summary", "Where did today end?");
        var shutdownNextBox = LabeledInput("Tomorrow's first step", "Exact first action tomorrow...");
        var shutdownBtn = new Button { Content = "Complete Evening Shutdown" };
        shutdownBtn.Click += (_, _) =>
        {
            try
            {
                var summary = shutdownNowBox.Text.Trim();
                var tomorrow = shutdownNextBox.Text.Trim();
                if (!string.IsNullOrEmpty(summary) || !string.IsNullOrEmpty(tomorrow))
                {
                    CoreServices.Call("ramdump.save", new
                    {
                        now = summary,
                        status = "Evening Shutdown (parked)",
                        next_step = tomorrow,
                    });
                }

                var openCount = 0;
                var imp = CoreServices.Call("impulses.list");
                if (imp.ValueKind == JsonValueKind.Array)
                {
                    foreach (var item in imp.EnumerateArray())
                    {
                        if (!item.TryGetProperty("done", out var done) || !done.GetBoolean())
                        {
                            openCount++;
                        }
                    }
                }

                CoreServices.Call("shutdown.save", new
                {
                    timestamp = DateTimeOffset.UtcNow.ToUnixTimeSeconds(),
                    date = DateTime.Now.ToString("yyyy-MM-dd"),
                    tomorrow_first_step = tomorrow,
                    summary_now = summary,
                    remaining_impulses = openCount,
                });
                SetStatus("Evening shutdown saved.");
                RenderCurrentRoute();
            }
            catch (Exception ex)
            {
                SetStatus(ex.Message);
            }
        };
        var shutdownStack = new StackPanel { Spacing = 10 };
        shutdownStack.Children.Add(new TextBlock { Text = "Evening Shutdown Ritual", FontSize = 15, FontWeight = Microsoft.UI.Text.FontWeights.SemiBold });
        shutdownStack.Children.Add(shutdownNowBox);
        shutdownStack.Children.Add(shutdownNextBox);
        shutdownStack.Children.Add(shutdownBtn);
        InteractivePanel.Children.Add(CardBorder(shutdownStack));

        // Saved RAM-dumps list
        if (dumps.ValueKind == JsonValueKind.Array && dumps.GetArrayLength() > 0)
        {
            InteractivePanel.Children.Add(new TextBlock
            {
                Text = $"Saved RAM-dumps ({dumps.GetArrayLength()})",
                FontSize = 15,
                FontWeight = Microsoft.UI.Text.FontWeights.SemiBold,
                Margin = new Thickness(0, 4, 0, 0),
            });

            for (var i = 0; i < dumps.GetArrayLength(); i++)
            {
                var entry = dumps[i];
                var idx = i;
                var ts = entry.TryGetProperty("timestamp", out var tEl) ? tEl.GetString() ?? "" : "";
                var n = entry.TryGetProperty("now", out var nEl) ? nEl.GetString() ?? "" : "";
                var s = entry.TryGetProperty("status", out var sEl) ? sEl.GetString() ?? "" : "";
                var nx = entry.TryGetProperty("next_step", out var nxEl) ? nxEl.GetString() ?? "" : "";

                var itemStack = new StackPanel { Spacing = 6 };
                itemStack.Children.Add(new TextBlock { Text = ts, FontSize = 11, Opacity = 0.65 });
                if (!string.IsNullOrEmpty(n)) itemStack.Children.Add(new TextBlock { Text = $"Now: {n}", TextWrapping = TextWrapping.Wrap });
                if (!string.IsNullOrEmpty(s)) itemStack.Children.Add(new TextBlock { Text = $"Status: {s}", TextWrapping = TextWrapping.Wrap, Opacity = 0.85 });
                if (!string.IsNullOrEmpty(nx)) itemStack.Children.Add(new TextBlock { Text = $"Next: {nx}", TextWrapping = TextWrapping.Wrap, FontWeight = Microsoft.UI.Text.FontWeights.SemiBold });

                var btnRow = new StackPanel { Orientation = Orientation.Horizontal, Spacing = 8 };
                var editBtn = new Button { Content = "Edit" };
                editBtn.Click += (_, _) =>
                {
                    _editingRamDumpIndex = idx;
                    RenderCurrentRoute();
                };
                var deleteBtn = new Button { Content = "Delete" };
                deleteBtn.Click += (_, _) =>
                {
                    CoreServices.Call("ramdump.delete", new { index = idx });
                    if (_editingRamDumpIndex == idx) _editingRamDumpIndex = null;
                    SetStatus("RAM-dump deleted.");
                    RenderCurrentRoute();
                };
                btnRow.Children.Add(editBtn);
                btnRow.Children.Add(deleteBtn);
                itemStack.Children.Add(btnRow);

                InteractivePanel.Children.Add(CardBorder(itemStack));
            }
        }
    }

    private void RenderImpulsesView()
    {
        var input = LabeledInput("Park a thought / side-quest", "E.g. refactor module, reply to email...");
        var addBtn = new Button { Content = "Add Impulse" };
        addBtn.Click += (_, _) =>
        {
            var text = input.Text.Trim();
            if (string.IsNullOrEmpty(text)) return;
            CoreServices.Call("impulses.add", new { text });
            SetStatus("Impulse parked.");
            RenderCurrentRoute();
        };

        var addStack = new StackPanel { Spacing = 8 };
        addStack.Children.Add(input);
        addStack.Children.Add(addBtn);
        InteractivePanel.Children.Add(CardBorder(addStack));

        var list = CoreServices.Call("impulses.list");
        var itemsStack = new StackPanel { Spacing = 6 };
        if (list.ValueKind == JsonValueKind.Array && list.GetArrayLength() > 0)
        {
            for (var i = 0; i < list.GetArrayLength(); i++)
            {
                var item = list[i];
                var idx = i;
                var text = item.TryGetProperty("text", out var t) ? t.GetString() ?? "" : "";
                var done = item.TryGetProperty("done", out var d) && d.GetBoolean();
                var cb = new CheckBox
                {
                    Content = text,
                    IsChecked = done,
                };
                cb.Click += (_, _) =>
                {
                    var nextDone = cb.IsChecked == true;
                    CoreServices.Call("impulses.set_done", new { index = idx, done = nextDone });
                    RenderCurrentRoute();
                };
                itemsStack.Children.Add(cb);
            }
        }
        else
        {
            itemsStack.Children.Add(new TextBlock { Text = "No parked impulses yet.", Opacity = 0.65 });
        }

        InteractivePanel.Children.Add(CardBorder(itemsStack));
    }

    private void RenderMomentumView()
    {
        var stats = CoreServices.Call("stats.load");
        var tier = stats.TryGetProperty("tier_title", out var t) ? t.GetString() ?? "Spark" : "Spark";
        var xp = stats.TryGetProperty("xp", out var x) ? x.GetUInt64() : 0;
        var todayMins = stats.TryGetProperty("today_focus_mins", out var td) ? td.GetUInt64() : 0;
        var weekMins = stats.TryGetProperty("current_week_mins", out var wm) ? wm.GetUInt64() : 0;
        var totalMins = stats.TryGetProperty("total_focus_mins", out var tm) ? tm.GetUInt64() : 0;
        var sessions = stats.TryGetProperty("current_week_sessions", out var ws) ? ws.GetUInt32() : 0;
        var impulses = stats.TryGetProperty("current_week_impulses_captured", out var wi) ? wi.GetUInt32() : 0;

        var summary = new StackPanel { Spacing = 8 };
        summary.Children.Add(new TextBlock { Text = $"{tier} · {xp} XP", FontSize = 20, FontWeight = Microsoft.UI.Text.FontWeights.SemiBold });
        summary.Children.Add(new TextBlock { Text = $"Today's focus: {todayMins} min" });
        summary.Children.Add(new TextBlock { Text = $"Weekly focus: {weekMins / 60}h {weekMins % 60}m ({sessions} sessions)" });
        summary.Children.Add(new TextBlock { Text = $"All-time focus: {totalMins} min" });
        summary.Children.Add(new TextBlock { Text = $"Impulses parked this week: {impulses}" });
        InteractivePanel.Children.Add(CardBorder(summary));
    }

    private void RenderRewardsView()
    {
        var account = CoreServices.Call("dopamine.load");
        var balance = account.TryGetProperty("balance", out var b) ? b.GetUInt32() : 0;
        var earned = account.TryGetProperty("total_earned", out var te) ? te.GetUInt32() : 0;
        var spent = account.TryGetProperty("total_spent", out var ts) ? ts.GetUInt32() : 0;

        var header = new StackPanel { Spacing = 4 };
        header.Children.Add(new TextBlock { Text = $"Balance: {balance} DP", FontSize = 22, FontWeight = Microsoft.UI.Text.FontWeights.SemiBold });
        header.Children.Add(new TextBlock { Text = $"Earned: {earned} DP · Spent: {spent} DP", Opacity = 0.7 });
        InteractivePanel.Children.Add(CardBorder(header));

        var titleBox = LabeledInput("New custom reward", "E.g. Espresso & walk");
        var costBox = LabeledInput("Cost (DP)", "1", "1");
        var addRewardBtn = new Button { Content = "Add Custom Reward" };
        addRewardBtn.Click += (_, _) =>
        {
            var title = titleBox.Text.Trim();
            if (string.IsNullOrEmpty(title)) return;
            if (!uint.TryParse(costBox.Text.Trim(), out var cost) || cost == 0) cost = 1;
            CoreServices.Call("dopamine.reward.add", new
            {
                title,
                cost,
                category = "custom",
            });
            SetStatus($"Added reward '{title}'.");
            RenderCurrentRoute();
        };

        var createStack = new StackPanel { Spacing = 8 };
        createStack.Children.Add(titleBox);
        createStack.Children.Add(costBox);
        createStack.Children.Add(addRewardBtn);
        InteractivePanel.Children.Add(CardBorder(createStack));

        if (account.TryGetProperty("rewards", out var rewards) && rewards.ValueKind == JsonValueKind.Array)
        {
            foreach (var r in rewards.EnumerateArray())
            {
                var id = r.TryGetProperty("id", out var idEl) ? idEl.GetString() ?? "" : "";
                var title = r.TryGetProperty("title", out var tEl) ? tEl.GetString() ?? "" : "";
                var cost = r.TryGetProperty("cost", out var cEl) ? cEl.GetUInt32() : 0;
                var isCustom = r.TryGetProperty("is_custom", out var icEl) && icEl.GetBoolean();

                var row = new StackPanel { Spacing = 6 };
                row.Children.Add(new TextBlock { Text = $"{title} ({cost} DP)", FontWeight = Microsoft.UI.Text.FontWeights.SemiBold });
                var actions = new StackPanel { Orientation = Orientation.Horizontal, Spacing = 8 };
                var redeemBtn = new Button { Content = "Redeem", IsEnabled = balance >= cost };
                redeemBtn.Click += (_, _) =>
                {
                    try
                    {
                        CoreServices.Call("dopamine.redeem", new { reward_id = id });
                        SetStatus($"Redeemed '{title}'.");
                        RenderCurrentRoute();
                    }
                    catch (Exception ex)
                    {
                        SetStatus(ex.Message);
                    }
                };
                actions.Children.Add(redeemBtn);
                if (isCustom)
                {
                    var removeBtn = new Button { Content = "Remove" };
                    removeBtn.Click += (_, _) =>
                    {
                        CoreServices.Call("dopamine.reward.remove", new { reward_id = id });
                        SetStatus($"Removed '{title}'.");
                        RenderCurrentRoute();
                    };
                    actions.Children.Add(removeBtn);
                }
                row.Children.Add(actions);
                InteractivePanel.Children.Add(CardBorder(row));
            }
        }
    }

    private void RenderSettingsView()
    {
        var loaded = CoreServices.Call("settings.load");
        var focusRatio = loaded.TryGetProperty("ratio_focus", out var rf) ? rf.GetUInt32() : 45;
        var breakRatio = loaded.TryGetProperty("ratio_break", out var rb) ? rb.GetUInt32() : 15;
        var cueMins = loaded.TryGetProperty("time_cue_interval_mins", out var tc) ? tc.GetUInt32() : 30;
        var hardStopEnabled = loaded.TryGetProperty("hard_stop_enabled", out var hs) && hs.GetBoolean();
        var hardStopTime = loaded.TryGetProperty("hard_stop_time", out var hst) ? hst.GetString() ?? "17:00" : "17:00";
        var hardStopLabel = loaded.TryGetProperty("hard_stop_label", out var hsl) ? hsl.GetString() ?? "" : "";
        var checkUpdatesAuto = !loaded.TryGetProperty("check_updates_automatically", out var cua) || cua.GetBoolean();

        var focusBox = LabeledInput("Focus ratio (minutes)", "45", focusRatio.ToString());
        var breakBox = LabeledInput("Break ratio (minutes)", "15", breakRatio.ToString());
        var cueBox = LabeledInput("Time-anchor interval (minutes, 0 = off)", "30", cueMins.ToString());
        var hardStopCheck = new CheckBox { Content = "Enable daily Hard-Stop reminder", IsChecked = hardStopEnabled };
        var hardStopTimeBox = LabeledInput("Hard-Stop time (HH:MM)", "17:00", hardStopTime);
        var hardStopLabelBox = LabeledInput("Hard-Stop label", "Stop work / dinner", hardStopLabel);
        var checkUpdatesBox = new CheckBox { Content = "Check for updates automatically on GitHub", IsChecked = checkUpdatesAuto };

        var saveBtn = new Button { Content = "Save Settings" };
        saveBtn.Click += (_, _) =>
        {
            try
            {
                var node = JsonNode.Parse(loaded.GetRawText())?.AsObject() ?? new JsonObject();
                if (!uint.TryParse(focusBox.Text.Trim(), out var nextFocus) || nextFocus == 0) nextFocus = 45;
                if (!uint.TryParse(breakBox.Text.Trim(), out var nextBreak) || nextBreak == 0) nextBreak = 15;
                if (!uint.TryParse(cueBox.Text.Trim(), out var nextCue)) nextCue = 30;

                node["ratio_focus"] = nextFocus;
                node["ratio_break"] = nextBreak;
                node["time_cue_interval_mins"] = nextCue;
                node["hard_stop_enabled"] = hardStopCheck.IsChecked == true;
                node["hard_stop_time"] = hardStopTimeBox.Text.Trim();
                node["hard_stop_label"] = hardStopLabelBox.Text.Trim();
                node["check_updates_automatically"] = checkUpdatesBox.IsChecked == true;

                CoreServices.Call("settings.save", node);
                SetStatus("Settings saved.");
                RenderCurrentRoute();
            }
            catch (Exception ex)
            {
                SetStatus(ex.Message);
            }
        };

        var checkUpdateNowBtn = new Button { Content = "Check for updates now" };
        checkUpdateNowBtn.Click += (_, _) =>
        {
            try
            {
                var info = CoreServices.Call("update.check", new { force = true });
                var available = info.TryGetProperty("update_available", out var ua) && ua.GetBoolean();
                var current = info.TryGetProperty("current_version", out var cv) ? cv.GetString() ?? "" : "";
                var latest = info.TryGetProperty("latest_version", out var lv) ? lv.GetString() ?? "" : "";
                var url = info.TryGetProperty("html_url", out var urlEl) ? urlEl.GetString() ?? "" : "";
                if (available)
                {
                    SetStatus($"Update available: v{latest} (you have v{current}). {url}");
                }
                else
                {
                    SetStatus($"Tether v{current} is up to date.");
                }
            }
            catch (Exception ex)
            {
                SetStatus($"Update check failed: {ex.Message}");
            }
        };

        var actionsRow = new StackPanel { Orientation = Orientation.Horizontal, Spacing = 8 };
        actionsRow.Children.Add(saveBtn);
        actionsRow.Children.Add(checkUpdateNowBtn);

        var stack = new StackPanel { Spacing = 10 };
        stack.Children.Add(focusBox);
        stack.Children.Add(breakBox);
        stack.Children.Add(cueBox);
        stack.Children.Add(hardStopCheck);
        stack.Children.Add(hardStopTimeBox);
        stack.Children.Add(hardStopLabelBox);
        stack.Children.Add(checkUpdatesBox);
        stack.Children.Add(actionsRow);
        InteractivePanel.Children.Add(CardBorder(stack));
    }
}
