using System;
using System.ComponentModel;
using Windows.UI.Popups;
using Windows.UI.Xaml.Controls;
using Windows.UI.Xaml.Input;
using Windows.UI.Xaml.Media.Animation;
using Windows.UI.Xaml.Navigation;
using HBCCSignIn.Logic.Data;

namespace HBCCSignIn.Pages
{
    public sealed partial class UpdatePersonPage : Page, INotifyPropertyChanged
    {
        public UpdatePersonPage()
        {
            InitializeComponent();
        }

        protected override void OnNavigatedTo(NavigationEventArgs e)
        {
            Person = (Person)e.Parameter;
            Person.Clear();

            base.OnNavigatedTo(e);
        }

        Person person;
        public Person Person
        {
            get => person;
            set { person = value; OnPropertyChanged(); }
        }

        async void Done_Tapped(object sender, TappedRoutedEventArgs e)
        {
            if (!IsEnabled)
                return;

            try
            {
                IsEnabled = false;

                string issues = "Enter a valid address.";
                if (!Person.Address.IsValid() || !Person.Guardian.IsValid(out issues))
                {
                    await new MessageDialog(issues).ShowAsync();
                    return;
                }

                if (await Person.Save(isUpdating: true))
                    GoBack();
            }
            finally
            {
                IsEnabled = true;
            }
        }

        void Cancel_Tapped(object sender, TappedRoutedEventArgs e)
        {
            Person.Clear();
            GoBack();
        }

        void GoBack() => ((Frame)Parent)?.GoBack(new SlideNavigationTransitionInfo());

        async void NoInfo_Click(object sender, Windows.UI.Xaml.RoutedEventArgs e)
        {
            var question = new MessageDialog("If you don't have your info available right now, you can update it next time. Are you sure?")
            {
                Commands =
                {
                    new UICommand("Skip"),
                    new UICommand("Cancel")
                }
            };

            var answer = await question.ShowAsync();
            if (answer?.Label == "Skip")
            {
                Person.SkipNextExpire = true;
                Person.Clear();
                GoBack();
            }
        }

        #region Notify

        /// <summary>
        /// Property Changed event
        /// </summary>
        public event PropertyChangedEventHandler PropertyChanged;

        /// <summary>
        /// Fire the PropertyChanged event
        /// </summary>
        /// <param name="propertyName">Name of the property that changed (defaults from CallerMemberName)</param>
        void OnPropertyChanged([System.Runtime.CompilerServices.CallerMemberName] string propertyName = "")
        {
            PropertyChanged?.Invoke(this, new PropertyChangedEventArgs(propertyName));
        }

        #endregion
    }
}
