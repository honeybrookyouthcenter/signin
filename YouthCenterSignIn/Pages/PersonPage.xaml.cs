using Windows.UI.Xaml.Controls;
using Windows.UI.Xaml.Navigation;
using YouthCenterSignIn.Logic.Data;

// The Blank Page item template is documented at https://go.microsoft.com/fwlink/?LinkId=234238

namespace YouthCenterSignIn.Pages
{
    /// <summary>
    /// An empty page that can be used on its own or navigated to within a Frame.
    /// </summary>
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
            await Person.SignInOut();
            Close();
        }

        void Close() => ((Frame)Parent).GoBack();

        private void Cancel_Tapped(object sender, Windows.UI.Xaml.Input.TappedRoutedEventArgs e)
        {
            Close();
        }
    }
}
