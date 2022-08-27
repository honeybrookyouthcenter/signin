using SignIn.Logic.Data;
using System;
using Windows.UI.Popups;
using Windows.UI.ViewManagement;
using Windows.UI.Xaml;
using Windows.UI.Xaml.Controls;
using Windows.UI.Xaml.Input;
using Windows.UI.Xaml.Media.Animation;

// The Blank Page item template is documented at https://go.microsoft.com/fwlink/?LinkId=234238

namespace HBCCSignIn.Pages
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

            GoToState(PageState.Person);
        }

        public Person NewPerson { get; } = new Person() { Guardian = new Guardian() };

        private async void Cancel_Tapped(object sender, TappedRoutedEventArgs e)
        {
            MessageDialog message = new MessageDialog("Are you sure you want to cancel? You'll loose any information you entered.", "Cancel");
            message.Commands.Add(new UICommand("Yes", (_) => GoBack()));
            message.Commands.Add(new UICommand("No"));

            await message.ShowAsync();
        }

        async void Next_Tapped(object sender, TappedRoutedEventArgs e)
        {
            if (state == PageState.Person)
            {
                if (NewPerson.IsValid(out var issues))
                {
                    GoToState(PageState.Guardian);
                    uiScroller.ChangeView(0, 0, null);
                    uiGuardian.Focus();
                }
                else
                {
                    await new MessageDialog(issues).ShowAsync();
                }
            }
            else if (state == PageState.Guardian)
            {
                if (NewPerson.Guardian.IsValid(out var issues) && uiCovid.IsAgreed)
                {
                    GoToState(PageState.Rules);
                }
                else
                {
                    await new MessageDialog(issues ?? "Please review the agreement").ShowAsync();
                }
            }
        }

        private void Back_Tapped(object sender, TappedRoutedEventArgs e)
        {
            GoToState(state - 1);
        }

        async void Done_Tapped(object sender, TappedRoutedEventArgs e)
        {
            if (!IsEnabled)
                return;

            try
            {
                IsEnabled = false;

                if (await NewPerson.Save())
                {
                    await uiCovid.Save(NewPerson);

                    var parent = (Frame)Parent;
                    GoBack();
                    parent.Navigate(typeof(PersonPage), NewPerson);
                }
            }
            catch (Exception ex)
            {
                await DataProvider.Current.ShowMessage($"Oops, you found a bug", ex);
            }
            finally
            {
                IsEnabled = true;
            }
        }

        enum PageState { Person = 0, Guardian = 1, Rules = 2 }
        PageState state = PageState.Person;
        void GoToState(PageState state)
        {
            this.state = state;
            string visualState = "";
            switch (state)
            {
                case PageState.Person:
                    visualState = "PersonInfo";
                    break;
                case PageState.Guardian:
                    visualState = "GuardianInfo";
                    break;
                case PageState.Rules:
                    visualState = "Rules";
                    break;
            }

            VisualStateManager.GoToState(this, visualState, true);
        }

        void GoBack()
        {
            ((Frame)Parent).GoBack(new SlideNavigationTransitionInfo());
        }
    }
}
