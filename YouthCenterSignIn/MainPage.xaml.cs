using System.Threading.Tasks;
using Windows.UI.Xaml.Controls;
using YouthCenterSignIn.Pages;

namespace YouthCenterSignIn
{
    public sealed partial class MainPage : Page
    {
        UwpDataProvider DataProvider => ((UwpDataProvider)Logic.Data.DataProvider.Current);

        public MainPage()
        {
            InitializeComponent();

            Loaded += MainPage_Loaded;
        }

        private async void MainPage_Loaded(object sender, Windows.UI.Xaml.RoutedEventArgs e)
        {
            if (await DataProvider.Graph.Login() && DataProvider.Graph.IsAuthenticated)
                uiFrame.Navigate(typeof(HomePage));
            else
                await DataProvider.ShowAuthenticationMessage();
        }
    }
}
