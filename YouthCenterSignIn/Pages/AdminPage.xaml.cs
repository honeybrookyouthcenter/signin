using Windows.UI.Xaml.Controls;
using YouthCenterSignIn.Logic;
using YouthCenterSignIn.Logic.Data;

namespace YouthCenterSignIn.Pages
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
    }
}
