using Windows.UI.Xaml.Controls;
using YouthCenterSignIn.Logic;

// The Blank Page item template is documented at https://go.microsoft.com/fwlink/?LinkId=234238

namespace YouthCenterSignIn.Pages
{
    /// <summary>
    /// An empty page that can be used on its own or navigated to within a Frame.
    /// </summary>
    public sealed partial class AdminPage : Page
    {
        public Admin Admin { get; } = new Admin();

        public AdminPage()
        {
            this.InitializeComponent();
        }

        private void Back_Tapped(object sender, Windows.UI.Xaml.Input.TappedRoutedEventArgs e)
        {
            ((Frame)Parent).GoBack();
        }
    }
}
