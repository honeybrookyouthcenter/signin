using System;
using Windows.UI.Xaml.Controls;
using Windows.UI.Xaml.Navigation;
using YouthCenterSignIn.Logic.Data;

namespace YouthCenterSignIn.Pages
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

        private async void SignInOut_Tapped(object sender, Windows.UI.Xaml.Input.TappedRoutedEventArgs e)
        {
            if (await Person.SignInOut() == SignInOutResult.InfoExpired)
                AskForInfo();
            else
                Close();
        }

        void AskForInfo() => ((Frame)Parent).Navigate(typeof(UpdatePersonPage), Person);

        void Close() => ((Frame)Parent).GoBack();

        private void Cancel_Tapped(object sender, Windows.UI.Xaml.Input.TappedRoutedEventArgs e)
        {
            Close();
        }
    }
}
