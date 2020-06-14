using System;
using System.ComponentModel;
using Windows.UI.Popups;
using Windows.UI.Xaml.Controls;
using Windows.UI.Xaml.Input;
using Windows.UI.Xaml.Media.Animation;
using Windows.UI.Xaml.Navigation;
using YouthCenterSignIn.Logic.Data;

namespace YouthCenterSignIn.Pages
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
            if (!uiCovid.IsAgreed || !IsEnabled)
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
                {
                    await uiCovid.Save(Person);
                    GoBack();
                }
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
