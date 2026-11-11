using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Xaml.Navigation;

namespace Tether.WinUI;

internal sealed record FeaturePreviewInfo(string Title, string Subtitle, string Description);

public sealed partial class FeaturePreviewPage : Page
{
    public FeaturePreviewPage()
    {
        InitializeComponent();
    }

    protected override void OnNavigatedTo(NavigationEventArgs e)
    {
        base.OnNavigatedTo(e);
        if (e.Parameter is FeaturePreviewInfo preview)
        {
            PageTitle.Text = preview.Title;
            PageSubtitle.Text = preview.Subtitle;
            PageDescription.Text = preview.Description;
        }
    }
}
