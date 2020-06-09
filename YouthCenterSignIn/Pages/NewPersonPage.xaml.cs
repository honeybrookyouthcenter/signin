using System;
using Windows.UI.Popups;
using Windows.UI.ViewManagement;
using Windows.UI.Xaml;
using Windows.UI.Xaml.Controls;
using Windows.UI.Xaml.Input;
using Windows.UI.Xaml.Media.Animation;
using YouthCenterSignIn.Logic.Data;

// The Blank Page item template is documented at https://go.microsoft.com/fwlink/?LinkId=234238

namespace YouthCenterSignIn.Pages
{
    /// <summary>
    /// An empty page that can be used on its own or navigated to within a Frame.
    /// </summary>
    public sealed partial class NewPersonPage : Page
    {
        public NewPersonPage()
        {
            InitializeComponent();

            var inputPane = InputPane.GetForCurrentView();
            if (inputPane.Visible)
                ((FrameworkElement)Content).Margin = new Thickness(0, 0, 0, inputPane.OccludedRect.Height);

            VisualStateManager.GoToState(this, "PersonInfo", false);
        }

        public Person NewPerson { get; } = new Person() { Guardian = new Guardian() };

        private async void Cancel_Tapped(object sender, TappedRoutedEventArgs e)
        {
            MessageDialog message = new MessageDialog("Are you sure you want to cancel? You'll loose any information you entered.", "Cancel");
            message.Commands.Add(new UICommand("Yes", (_) => ((Frame)Parent).GoBack()));
            message.Commands.Add(new UICommand("No"));

            await message.ShowAsync();
        }

        async void Next_Tapped(object sender, TappedRoutedEventArgs e)
        {
            if (NewPerson.IsValid(out var issues))
            {
                VisualStateManager.GoToState(this, "GuardianInfo", true);
                uiScroller.ChangeView(0, 0, null);
                uiGuardian.Focus();
            }
            else
            {
                await new MessageDialog(issues).ShowAsync();
            }
        }

        private void Back_Tapped(object sender, TappedRoutedEventArgs e)
        {
            VisualStateManager.GoToState(this, "PersonInfo", true);
        }

        async void Done_Tapped(object sender, TappedRoutedEventArgs e)
        {
            if (!uiCovid.IsAgreed)
                return;

            if (!NewPerson.Guardian.IsValid(out var issues))
            {
                await new MessageDialog(issues).ShowAsync();
                return;
            }

            if (await NewPerson.Save())
            {
                await uiCovid.Save(NewPerson);

                var parent = (Frame)Parent;
                GoBack();
                parent.Navigate(typeof(PersonPage), NewPerson);
            }
        }

        void GoBack()
        {
            ((Frame)Parent).GoBack(new SlideNavigationTransitionInfo());
        }
    }
}
