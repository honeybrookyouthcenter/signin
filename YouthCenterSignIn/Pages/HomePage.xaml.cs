using Windows.UI.ViewManagement;
using Windows.UI.Xaml;
using Windows.UI.Xaml.Controls;
using YouthCenterSignIn.Logic;
using YouthCenterSignIn.Logic.Data;

namespace YouthCenterSignIn.Pages
{
    public sealed partial class HomePage : Page
    {
        public PersonSearch PersonSearch { get; } = new PersonSearch();

        public HomePage()
        {
            var getTask = Person.GetPeople();
            InitializeComponent();

            Loaded += HomePage_Loaded;
        }

        private void HomePage_Loaded(object sender, RoutedEventArgs e)
        {
            uiName.Focus(FocusState.Keyboard);
            InputPane.GetForCurrentView().TryShow();
        }

        private void uiPin_PasswordChanged(object sender, RoutedEventArgs e)
        {
            if (uiPin.Password.Length != 6)
                return;

            string pin = uiPin.Password;

            uiPinMessages.Text = "";
            uiPin.Password = "";

            if (DataProvider.Current.AuthenticateAdmin(pin))
                ((Frame)Parent).Navigate(typeof(AdminPage));
            else
                uiPinMessages.Text = "Wrong pin!";
        }

        private void SignUp_Tapped(object sender, RoutedEventArgs e)
        {
            ((Frame)Parent).Navigate(typeof(NewPersonPage));
        }

        private void UiName_SuggestionChosen(AutoSuggestBox sender, AutoSuggestBoxSuggestionChosenEventArgs args)
        {
            var person = (Person)args.SelectedItem;

            ((Frame)Parent).Navigate(typeof(PersonPage), person);
        }
    }
}
