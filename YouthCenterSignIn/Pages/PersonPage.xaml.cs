using System;
using Windows.UI.Xaml.Controls;
using Windows.UI.Xaml.Navigation;
using SignIn.Logic.Data;
using SignIn.Uwp.Pages;
using Windows.UI.Xaml.Input;

namespace SignIn.Pages
{
    public sealed partial class PersonPage : Page
    {
        public PersonPage()
        {
            InitializeComponent();
        }

        protected override async void OnNavigatedTo(NavigationEventArgs e)
        {
            Person = (Person)e.Parameter;
            await Person.RefreshSignedIn();

            base.OnNavigatedTo(e);
        }

        public Person Person { get; private set; }

        async void SignInOut_Tapped(object sender, Windows.UI.Xaml.Input.TappedRoutedEventArgs e)
        {
            try
            {
                uiSignInOut.IsEnabled = false;

                if (await Person.SignInOut() == SignInOutResult.InfoExpired)
                    AskForInfo();
                else
                    Close();
            }
            finally
            {
                uiSignInOut.IsEnabled = true;
            }
        }

        void AskForInfo() => ((Frame)Parent).Navigate(typeof(UpdatePersonPage), Person);

        void Close() => ((Frame)Parent)?.GoBack();

        void Cancel_Tapped(object sender, TappedRoutedEventArgs e) => Close();
    }
}
