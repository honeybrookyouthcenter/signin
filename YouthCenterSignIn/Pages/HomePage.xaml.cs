using Windows.UI.Xaml;
using Windows.UI.Xaml.Controls;
using Windows.UI.Xaml.Controls.Primitives;
using YouthCenterSignIn.Logic;
using YouthCenterSignIn.Logic.Data;

// The Blank Page item template is documented at https://go.microsoft.com/fwlink/?LinkId=234238

namespace YouthCenterSignIn.Pages
{
    /// <summary>
    /// An empty page that can be used on its own or navigated to within a Frame.
    /// </summary>
    public sealed partial class HomePage : Page
    {
        public PersonSearch PersonSearch { get; } = new PersonSearch();

        public HomePage()
        {
            var getTask = Person.GetPeople();
            InitializeComponent();
        }

        private void TextBlock_Tapped(object sender, Windows.UI.Xaml.Input.TappedRoutedEventArgs e)
        {
            var textblock = ((TextBlock)sender);
            var person = (Person)textblock.DataContext;

            ((Frame)Parent).Navigate(typeof(PersonPage), person);
        }

        private void Border_DoubleTapped(object sender, Windows.UI.Xaml.Input.DoubleTappedRoutedEventArgs e)
        {
            FlyoutBase.ShowAttachedFlyout((FrameworkElement)sender);
        }

        private void uiPin_PasswordChanged(object sender, RoutedEventArgs e)
        {
            if (uiPin.Password.Length != 4)
                return;

            string pin = uiPin.Password;

            uiPinMessages.Text = "";
            uiPin.Password = "";

            if (Logic.Data.DataProvider.Current.AuthenticateAdmin(pin))
                ((Frame)Parent).Navigate(typeof(AdminPage));
            else
                uiPinMessages.Text = "Wrong pin!";
        }

        private void SignUp_Tapped(object sender, Windows.UI.Xaml.Input.TappedRoutedEventArgs e)
        {
            ((Frame)Parent).Navigate(typeof(NewPersonPage));
        }
    }
}
