using SignIn.Logic;
using SignIn.Logic.Data;
using Windows.UI.Xaml.Controls;

namespace HBCCSignIn.Pages
{
    public sealed partial class AdminPage : Page
    {
        public Admin Admin { get; } = new Admin();

        public AdminPage()
        {
            InitializeComponent();
        }

        void Back_Tapped(object sender, Windows.UI.Xaml.Input.TappedRoutedEventArgs e)
        {
            ((Frame)Parent).GoBack();
        }

        void ChangePin_Click(object sender, Windows.UI.Xaml.RoutedEventArgs e)
        {
            if (Admin.ChangeAdminPin(uiCurrentPin.Password, uiNewPin.Password, uiConfirmPin.Password))
                uiChangePinFlyout.Hide();
        }

        void AppBarButton_Click(object sender, Windows.UI.Xaml.RoutedEventArgs e)
        {
            Person.ClearPeopleCache();
        }

        void Refresh_Tapped(object sender, Windows.UI.Xaml.Input.TappedRoutedEventArgs e)
        {
            Admin.RefreshLogs();
        }
    }
}
