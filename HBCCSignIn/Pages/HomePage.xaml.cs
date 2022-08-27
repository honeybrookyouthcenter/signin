using Windows.UI.ViewManagement;
using Windows.UI.Xaml;
using Windows.UI.Xaml.Controls;
using Windows.UI.Xaml.Documents;
using Windows.UI.Xaml.Input;
using HBCCSignIn.Controls;
using HBCCSignIn.Logic;
using HBCCSignIn.Logic.Data;

namespace HBCCSignIn.Pages
{
    public sealed partial class HomePage : Page
    {
        public PersonSearch PersonSearch { get; } = new PersonSearch();

        public HomePage()
        {
            var getTask = Person.GetPeople(alwaysUseCache: false);
            InitializeComponent();

            Loaded += HomePage_Loaded;
        }

        void HomePage_Loaded(object sender, RoutedEventArgs e)
        {
            uiName.Focus(FocusState.Keyboard);
            InputPane.GetForCurrentView().TryShow();
        }

        void uiPin_PasswordChanged(object sender, RoutedEventArgs e)
        {
            if (uiPin.Password.Length != 6)
                return;

            string pin = uiPin.Password;

            uiPinMessages.Text = "";
            uiPin.Password = "";

            if (new Admin().Authenticate(pin))
                ((Frame)Parent).Navigate(typeof(AdminPage));
            else
                uiPinMessages.Text = "Wrong pin!";
        }

        void UiName_SuggestionChosen(AutoSuggestBox sender, AutoSuggestBoxSuggestionChosenEventArgs args)
        {
            var person = (Person)args.SelectedItem;

            ((Frame)Parent).Navigate(typeof(PersonPage), person);
        }

        void SignUp_Click(object sender, RoutedEventArgs e)
        {
            ((Frame)Parent).Navigate(typeof(NewPersonPage));
        }

        void Admin_Opened(object sender, object e)
        {
            InputPane.GetForCurrentView().TryShow();
        }

        void Rules_Click(Hyperlink sender, HyperlinkClickEventArgs args)
        {
            var dialog = new ContentDialog()
            {
                CloseButtonCommand = new StandardUICommand(StandardUICommandKind.Close),
                Content = new RulesControl(),
            };
            _ = dialog.ShowAsync();
        }
    }
}
