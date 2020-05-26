using System;
using System.ComponentModel;
using Windows.UI.Popups;
using Windows.UI.Xaml.Controls;
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

        async void BigIconButton_Tapped(object sender, Windows.UI.Xaml.Input.TappedRoutedEventArgs e)
        {
            string issues = "Enter a valid address.";
            if (!Person.Address.IsValid() || !Person.Guardian.IsValid(out issues))
            {
                await new MessageDialog(issues).ShowAsync();
                return;
            }

            if (await Person.Save(isUpdating: true))
                GoBack();
        }

        void GoBack() => ((Frame)Parent).GoBack(new SlideNavigationTransitionInfo());

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
